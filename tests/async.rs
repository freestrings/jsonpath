extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate serde_json;

use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use common::{read_json, setup};
use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    task::Waker,
    Future, SinkExt, StreamExt,
};
use jsonpath::{JsonSelector, MultiJsonSelectorMut, PathParser};
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

    fn new_field(&self, input: Value) -> CryptoField {
        let bag = CryptoField::new(input);
        self.bags.lock().unwrap().push(bag.clone());
        bag
    }

    async fn send_request(&self) {
        let mut bags = self.bags.lock().unwrap();
        let inputs = bags
            .iter_mut()
            .filter_map(|bag| bag.input.take())
            .collect::<Vec<_>>();
        // let _ = reqwest::Client::new()
        //     .post("https://blackhole.posterior.io/vr5kvy")
        //     .body(serde_json::to_string(&inputs).unwrap())
        //     .send()
        //     .await
        //     .unwrap();
        for bag in bags.iter_mut() {
            bag.value.set_value(serde_json::json!(42));
        }
    }
}

#[derive(Clone)]
struct CryptoField {
    input: Option<Value>,
    value: ValueFuture<Value>,
}

impl CryptoField {
    fn new(input: Value) -> Self {
        Self {
            input: Some(input),
            value: ValueFuture::new(),
        }
    }

    pub fn value(self) -> ValueFuture<Value> {
        self.value
    }
}

#[tokio::test]
async fn selector_mut() {
    setup();

    let parser = PathParser::compile("$.store..price").unwrap();
    let parser_two = PathParser::compile("$.store..author").unwrap();
    let mut selector_mut =
        MultiJsonSelectorMut::new_multi_parser(vec![parser.into(), parser_two.into()]);

    let crypto_request = Arc::new(CryptoRequest::new());

    let result_futures = selector_mut
        .value(read_json("./benchmark/example.json"))
        .replace_with_async(|v| {
            let bag: CryptoField = crypto_request.new_field(v);

            Box::pin(async move {
                let val = bag.value().await;
                Some(val)
            })
        })
        .unwrap();

    crypto_request.send_request().await;

    let result = result_futures.await.unwrap().take().unwrap();

    let parser = PathParser::compile("$.store..price").unwrap();
    let mut selector = JsonSelector::new(parser);
    let result = selector.value(&result).select().unwrap();

    assert_eq!(
        vec![&json!(42), &json!(42), &json!(42), &json!(42), &json!(42)],
        result
    );
}
