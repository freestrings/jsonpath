extern crate jsonpath_lib as jsonpath;
extern crate serde_json;

use common::read_json;
use criterion::{criterion_group, criterion_main, BenchmarkId};

use jsonpath::{JsonSelectorMut, PathParser};
use serde_json::Value;

mod common;

fn selector_mut(mut selector_mut: JsonSelectorMut, json: Value) -> Value {
    let mut nums = Vec::new();
    let result = selector_mut
        .value(json)
        .replace_with(&mut |v| {
            if let Value::Number(n) = v {
                nums.push(n.as_f64().unwrap());
            }
            Ok(Some(Value::String("a".to_string())))
        })
        .unwrap()
        .take()
        .unwrap();

    result
}

fn setup_async_benchmark(c: &mut criterion::Criterion) {
    let t1_json = read_json("./benchmark/example.json");
    let t1_parser = PathParser::compile("$.store..price").unwrap();
    let t1_selector_mut = JsonSelectorMut::new(t1_parser.clone());
    let t1_selector_mut_two = JsonSelectorMut::new(t1_parser);

    let t2_json = read_json("./benchmark/big_example.json");
    let t2_parser = PathParser::compile("$.store.book[*].author").unwrap();
    let t2_parser_two = PathParser::compile("$.store.author").unwrap();
    let t2_selector_mut = JsonSelectorMut::new(t2_parser);
    let t2_selector_mut_two = JsonSelectorMut::new(t2_parser_two);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    c.bench_with_input(
        BenchmarkId::new("selector_mut", "Json"),
        &(t1_selector_mut.clone(), t1_json.clone()),
        |b, (s, v)| {
            // Insert a call to `to_async` to convert the bencher to async mode.
            // The timing loops are the same as with the normal bencher.
            b.to_async(&runtime).iter_batched(
                || (s.clone(), v.clone()),
                |(s, v)| async {
                    selector_mut(s, v);
                },
                criterion::BatchSize::SmallInput,
            );
        },
    );

    c.bench_with_input(
        BenchmarkId::new("selector_mut", "BigJson"),
        &(t2_selector_mut.clone(), t2_json.clone()),
        |b, (s, v)| {
            // Insert a call to `to_async` to convert the bencher to async mode.
            // The timing loops are the same as with the normal bencher.
            b.to_async(&runtime).iter_batched(
                || (s.clone(), v.clone()),
                |(s, v)| async {
                    selector_mut(s, v);
                },
                criterion::BatchSize::LargeInput,
            );
        },
    );

    c.bench_with_input(
        BenchmarkId::new("double_selector_mut", "Json"),
        &(t1_selector_mut, t1_selector_mut_two, t1_json),
        |b, (s, s2, v)| {
            // Insert a call to `to_async` to convert the bencher to async mode.
            // The timing loops are the same as with the normal bencher.
            b.to_async(&runtime).iter_batched(
                || (s.clone(), s2.clone(), v.clone()),
                |(s, s2, v)| async {
                    let v = selector_mut(s, v);
                    let _ = selector_mut(s2, v);
                },
                criterion::BatchSize::SmallInput,
            );
        },
    );

    c.bench_with_input(
        BenchmarkId::new("double_selector_mut", "BigJson"),
        &(t2_selector_mut, t2_selector_mut_two, t2_json),
        |b, (s, s2, v)| {
            // Insert a call to `to_async` to convert the bencher to async mode.
            // The timing loops are the same as with the normal bencher.
            b.to_async(&runtime).iter_batched(
                || (s.clone(), s2.clone(), v.clone()),
                |(s, s2, v)| async {
                    let v = selector_mut(s, v);
                    let _ = selector_mut(s2, v);
                },
                criterion::BatchSize::LargeInput,
            );
        },
    );
}

criterion_group!(benches, setup_async_benchmark);
criterion_main!(benches);
