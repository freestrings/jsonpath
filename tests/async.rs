extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate serde_json;

use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use common::{read_json, setup};
use futures::Future;
use jsonpath::{
    JsonSelector, MultiJsonSelectorMutWithMetadata, PathParser, PathParserWithMetadata,
};
use serde_json::Value;

mod common;

#[derive(Clone)]
struct ValueFuture<T> {
    inner: Arc<Mutex<Option<T>>>,
}

impl<T> ValueFuture<T> {
    fn new() -> Self {
        ValueFuture {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    fn set_value(&self, value: T) {
        let mut inner = self.inner.lock().unwrap();
        *inner = Some(value);
    }
}

impl<T: Clone> Future for ValueFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = self.inner.lock().unwrap();
        if let Some(value) = inner.as_ref() {
            Poll::Ready(value.clone())
        } else {
            // This future isn't ready yet, so we'll notify the context when it is.
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

struct CryptoRequest {
    bags: Mutex<Vec<CryptoField>>,
}

impl CryptoRequest {
    fn new() -> Self {
        Self {
            bags: Mutex::new(Vec::new()),
        }
    }

    fn new_field(&self, metadata: String) -> CryptoField {
        let bag = CryptoField::new(metadata);
        self.bags.lock().unwrap().push(bag.clone());
        bag
    }

    async fn send_request(&self) {
        let mut bags = self.bags.lock().unwrap();
        for bag in bags.iter_mut() {
            bag.value.set_value(serde_json::Value::String(bag.metadata.clone()));
        }
    }
}

#[derive(Clone)]
struct CryptoField {
    metadata: String,
    value: ValueFuture<Value>,
}

impl CryptoField {
    fn new(metadata: String) -> Self {
        Self {
            metadata: metadata,
            value: ValueFuture::new(),
        }
    }

    pub fn value(self) -> ValueFuture<Value> {
        self.value
    }
}

#[tokio::test]
async fn async_selector_mut() {
    setup();

    let parser = PathParserWithMetadata::compile("$.store..price", "price-metadata").unwrap();
    let parser_two = PathParserWithMetadata::compile("$.store..author", "author-metadata").unwrap();
    let mut selector_mut =
        MultiJsonSelectorMutWithMetadata::new_multi_parser(vec![parser, parser_two]);

    let crypto_request = Arc::new(CryptoRequest::new());

    let result_futures = selector_mut
        .replace_with_async(read_json("./benchmark/example.json"), |_, m| {
            let bag: CryptoField = crypto_request.new_field(m.to_string());

            Box::pin(async move {
                let val = bag.value().await;
                Some(val)
            })
        })
        .unwrap();

    crypto_request.send_request().await;

    let root_result = result_futures.await.unwrap();

    // Check that it replaced $.store..price with 42
    let parser = PathParser::compile("$.store..price").unwrap();
    let mut selector = JsonSelector::new(parser);
    let result = selector.value(&root_result).select().unwrap();

    assert_eq!(
        vec![&json!("price-metadata"), &json!("price-metadata"), &json!("price-metadata"), &json!("price-metadata"), &json!("price-metadata")],
        result
    );

    // Check that it replaced $.store..author with 42
    let parser = PathParser::compile("$.store..author").unwrap();
    let mut selector = JsonSelector::new(parser);
    let result = selector.value(&root_result).select().unwrap();

    assert_eq!(
        vec![&json!("author-metadata"), &json!("author-metadata"), &json!("author-metadata"), &json!("author-metadata")],
        result
    );
}
