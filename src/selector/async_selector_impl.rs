use std::collections::HashSet;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use futures::future::BoxFuture;
use futures::stream::FuturesOrdered;
use futures::StreamExt;
use serde_json::map::Entry;
use serde_json::Value;

use crate::paths::PathParserWithMetadata;
use crate::{JsonPathError, JsonSelector};

type FutureValue = Pin<Box<dyn Future<Output = Option<Value>> + Send>>;

struct JsonPointerWithMetadata<'a, T: Debug + Send + Sync> {
    pointer: String,
    metadata: &'a T,
}

impl<'a, T: Debug + Send + Sync> From<(Vec<String>, &'a T)> for JsonPointerWithMetadata<'a, T> {
    fn from((pointer, metadata): (Vec<String>, &'a T)) -> Self {
        let pointer = "/".to_owned() + &pointer.join("/");

        JsonPointerWithMetadata { pointer, metadata }
    }
}

#[derive(Default, Clone)]
pub struct MultiJsonSelectorMutWithMetadata<'a, T: Debug + Send + Sync> {
    parser: Option<Vec<PathParserWithMetadata<'a, T>>>,
}

impl<'a, T: Debug + Send + Sync + 'a> MultiJsonSelectorMutWithMetadata<'a, T> {
    pub fn new(parser: PathParserWithMetadata<'a, T>) -> Self {
        Self::new_ref(vec![parser])
    }

    pub fn new_multi_parser(parsers: Vec<PathParserWithMetadata<'a, T>>) -> Self {
        Self::new_ref(parsers)
    }

    pub fn new_ref(parser: Vec<PathParserWithMetadata<'a, T>>) -> Self {
        MultiJsonSelectorMutWithMetadata {
            parser: Some(parser),
        }
    }

    pub fn reset_parser(&mut self, parser: PathParserWithMetadata<'a, T>) -> &mut Self {
        self.parser = Some(vec![parser]);
        self
    }

    pub fn reset_parser_ref(&mut self, parser: Vec<PathParserWithMetadata<'a, T>>) -> &mut Self {
        self.parser = Some(parser);
        self
    }

    pub fn delete(&mut self, value: Value) -> Result<&mut Self, JsonPathError> {
        self.replace_with(value, &mut |_| Ok(Some(Value::Null)))
    }

    pub fn remove(&mut self, value: Value) -> Result<&mut Self, JsonPathError> {
        self.replace_with(value, &mut |_| Ok(None))
    }

    fn select_with_parser<'b>(
        &self,
        value: &'b Value,
        parser: &PathParserWithMetadata<'b, T>,
    ) -> Result<Vec<&'b Value>, JsonPathError> {
        let mut selector = JsonSelector::default();

        selector.reset_parser_ref(parser.parser());

        selector.value(value);

        selector.select()
    }

    fn select<'b>(&self, value: &'b Value) -> Result<Vec<&'b Value>, JsonPathError>
    where
        'a: 'b,
    {
        let res: Vec<Result<Vec<&Value>, JsonPathError>> =
            if let Some(parser) = self.parser.as_ref() {
                parser
                    .iter()
                    .map(|p| {
                        let mut selector = JsonSelector::default();
                        selector.reset_parser_ref(p.parser());
                        selector.value(value);
                        selector.select()
                    })
                    .collect()
            } else {
                return Err(JsonPathError::EmptyPath);
            };

        Ok(res.into_iter().flatten().flatten().collect())
    }

    pub fn replace_with<F>(
        &mut self,
        mut value: Value,
        fun: &mut F,
    ) -> Result<&mut Self, JsonPathError>
    where
        F: FnMut(Value) -> Result<Option<Value>, JsonPathError>,
    {
        let result = self.select(&value)?;
        let result = result
            .into_iter()
            .filter(|v| !v.is_object() && !v.is_array())
            .collect();
        let paths = self.compute_paths(&value, result);

        for tokens in paths {
            Self::replace_value(tokens, &mut value, fun)?;
        }

        Ok(self)
    }

    fn get_json_pointers(
        &'a self,
        value: &Value,
    ) -> Result<Vec<JsonPointerWithMetadata<'a, T>>, JsonPathError> {
        let Some(parsers) = &self.parser else {
            return Err(JsonPathError::EmptyPath);
        };

        let paths = parsers
            .iter()
            .map(|p| {
                let selections = self.select_with_parser(value, p)?;
                let selections = selections
                    .into_iter()
                    .filter(|v| !v.is_object() && !v.is_array())
                    .collect();
                let paths = self.compute_paths(value, selections);

                let paths_with_metadata: Vec<JsonPointerWithMetadata<'a, T>> = paths
                    .into_iter()
                    .map(|pointer| JsonPointerWithMetadata::from((pointer, p.metadata())))
                    .collect();

                Ok(paths_with_metadata)
            })
            .collect::<Result<Vec<Vec<JsonPointerWithMetadata<'a, T>>>, JsonPathError>>()?;

        let pointers: Vec<JsonPointerWithMetadata<'a, T>> = paths.into_iter().flatten().collect();

        Ok(pointers)
    }

    /// Replace the value at the given path with the result of some asynchronous computation.
    /// The function provided is called with the current value and the metadata associated with the path,
    /// and should return a Future that resolves to an Option<Value>. This value will replace the current value.
    pub fn replace_with_async<F>(
        &mut self,
        mut value: Value,
        fun: F,
    ) -> Result<BoxFuture<Result<Value, JsonPathError>>, JsonPathError>
    where
        F: Fn(Value, &T) -> FutureValue,
    {
        let mut futures = FuturesOrdered::new();

        let json_pointers = self.get_json_pointers(&value)?;

        for pointer in json_pointers.iter() {
            let target = value
                .pointer_mut(&pointer.pointer)
                .ok_or(JsonPathError::EmptyValue)?;
            let future = fun(std::mem::replace(target, Value::Null), pointer.metadata);
            futures.push_back(future);
        }

        let result_future = Box::pin(async move {
            // FuturesOrdered maintains a strict FIFO order, so we can use the index to get the pointer
            let mut i = 0;
            while let Some(res) = futures.next().await {
                // Get the pointer for this value
                let pointer = json_pointers.get(i).ok_or(JsonPathError::EmptyValue)?;

                if let Some(v) = res {
                    let target = value
                        .pointer_mut(&pointer.pointer)
                        .ok_or(JsonPathError::EmptyValue)?;
                    *target = v;
                } else {
                    // If None is returned then delete the value
                    value.as_object_mut().unwrap().remove(&pointer.pointer);
                }
                i += 1;
            }

            Ok::<_, JsonPathError>(value)
        });

        Ok(result_future)
    }

    fn replace_value<F>(
        mut tokens: Vec<String>,
        value: &mut Value,
        fun: &mut F,
    ) -> Result<(), JsonPathError>
    where
        F: FnMut(Value) -> Result<Option<Value>, JsonPathError>,
    {
        let mut target = value;

        let last_index = tokens.len().saturating_sub(1);
        for (i, token) in tokens.drain(..).enumerate() {
            let target_once = target;
            let is_last = i == last_index;
            let target_opt = match *target_once {
                Value::Object(ref mut map) => {
                    if is_last {
                        if let Entry::Occupied(mut e) = map.entry(token) {
                            let v = e.insert(Value::Null);
                            if let Some(res) = fun(v)? {
                                e.insert(res);
                            } else {
                                e.remove();
                            }
                        }
                        return Ok(());
                    }
                    map.get_mut(&token)
                }
                Value::Array(ref mut vec) => {
                    if let Ok(x) = token.parse::<usize>() {
                        if is_last {
                            if x < vec.len() {
                                let v = std::mem::replace(&mut vec[x], Value::Null);
                                if let Some(res) = fun(v)? {
                                    vec[x] = res;
                                } else {
                                    vec.remove(x);
                                }
                            }
                            return Ok(());
                        }
                        vec.get_mut(x)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(t) = target_opt {
                target = t;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn compute_paths(&self, origin: &Value, mut result: Vec<&Value>) -> Vec<Vec<String>> {
        let mut visited = HashSet::new();
        let mut visited_order = Vec::new();

        let mut tokens = Vec::new();
        Self::walk(
            origin,
            &mut result,
            &mut tokens,
            &mut visited,
            &mut visited_order,
        );

        visited_order
    }

    fn walk(
        origin: &Value,
        target: &mut Vec<&Value>,
        tokens: &mut Vec<String>,
        visited: &mut HashSet<*const Value>,
        visited_order: &mut Vec<Vec<String>>,
    ) -> bool {
        trace!("{:?}, {:?}", target, tokens);

        if target.is_empty() {
            return true;
        }

        target.retain(|t| {
            if std::ptr::eq(origin, *t) {
                if visited.insert(*t) {
                    visited_order.push(tokens.to_vec());
                }
                false
            } else {
                true
            }
        });

        match origin {
            Value::Array(vec) => {
                for (i, v) in vec.iter().enumerate() {
                    tokens.push(i.to_string());
                    if Self::walk(v, target, tokens, visited, visited_order) {
                        return true;
                    }
                    tokens.pop();
                }
            }
            Value::Object(map) => {
                for (k, v) in map {
                    tokens.push(k.clone());
                    if Self::walk(v, target, tokens, visited, visited_order) {
                        return true;
                    }
                    tokens.pop();
                }
            }
            _ => {}
        }

        false
    }
}
