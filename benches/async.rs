extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate serde_json;

use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use common::read_json;
use criterion::{criterion_group, criterion_main, BenchmarkId};
use futures::Future;
use jsonpath::{MultiJsonSelectorMutWithMetadata, PathParserWithMetadata};
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

struct MutationRequest {
    bags: Mutex<Vec<Field>>,
}

impl MutationRequest {
    fn new() -> Self {
        Self {
            bags: Mutex::new(Vec::new()),
        }
    }

    fn new_field(&self, input: Value) -> Field {
        let bag = Field::new(input);
        self.bags.lock().unwrap().push(bag.clone());
        bag
    }

    async fn send_request(&self) {
        let mut bags = self.bags.lock().unwrap();
        for bag in bags.iter_mut() {
            bag.value.set_value(bag.input.take().unwrap());
        }
    }
}

#[derive(Clone)]
struct Field {
    input: Option<Value>,
    value: ValueFuture<Value>,
}

impl Field {
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

async fn async_run(mut selector_mut: MultiJsonSelectorMutWithMetadata<'_, &str>, json: Value) {
    let mut_request = Arc::new(MutationRequest::new());

    let result_futures = selector_mut
        .replace_with_async(json, |v, _| {
            let bag: Field = mut_request.new_field(v);

            Box::pin(async move {
                let val = bag.value().await;
                Some(val)
            })
        })
        .unwrap();

    mut_request.send_request().await;

    let _result = result_futures.await.unwrap();
}

fn setup_async_benchmark(c: &mut criterion::Criterion) {
    let t1_json = read_json("./benchmark/example.json");
    let t1_parser = PathParserWithMetadata::compile("$.store..price", "one").unwrap();
    let t1_parser_two = PathParserWithMetadata::compile("$.store..author", "two").unwrap();
    let t1_selector_mut =
        MultiJsonSelectorMutWithMetadata::new_multi_parser(vec![t1_parser, t1_parser_two]);

    // let big_array = read_json("./benchmark/big_array.json");
    let t2_json = read_json("./benchmark/big_example.json");
    let t2_parser = PathParserWithMetadata::compile("$.store.book[*].author", "one").unwrap();
    let t2_parser_two = PathParserWithMetadata::compile("$.store.author", "two").unwrap();
    let t2_selector_mut =
        MultiJsonSelectorMutWithMetadata::new_multi_parser(vec![t2_parser, t2_parser_two]);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    c.bench_with_input(
        BenchmarkId::new("async_selector_mut", "Json"),
        &(t1_selector_mut, t1_json),
        |b, (s, v)| {
            // Insert a call to `to_async` to convert the bencher to async mode.
            // The timing loops are the same as with the normal bencher.
            b.to_async(&runtime).iter_batched(
                || (s.clone(), v.clone()),
                |(s, v)| async {
                    async_run(s, v).await;
                },
                criterion::BatchSize::SmallInput,
            );
        },
    );

    c.bench_with_input(
        BenchmarkId::new("async_selector_mut", "BigJson"),
        &(t2_selector_mut, t2_json),
        |b, (s, v)| {
            // Insert a call to `to_async` to convert the bencher to async mode.
            // The timing loops are the same as with the normal bencher.
            b.to_async(&runtime).iter_batched(
                || (s.clone(), v.clone()),
                |(s, v)| async {
                    async_run(s, v).await;
                },
                criterion::BatchSize::LargeInput,
            );
        },
    );
}

criterion_group!(benches, setup_async_benchmark);
criterion_main!(benches);
