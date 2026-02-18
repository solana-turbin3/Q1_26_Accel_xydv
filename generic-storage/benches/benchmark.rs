use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use generic_storage::{Borsh, Json, Person, Serializer, Wincode};

fn serialize_benchmark(c: &mut Criterion) {
    let person = Person {
        name: "aditya".to_string(),
        age: 21,
        gender: true,
    };

    let mut group = c.benchmark_group("serialize");

    group.bench_function("borsh", |b| b.iter(|| Borsh.to_bytes(black_box(&person))));

    group.bench_function("wincode", |b| {
        b.iter(|| Wincode.to_bytes(black_box(&person)))
    });

    group.bench_function("json", |b| b.iter(|| Json.to_bytes(black_box(&person))));
    group.finish();
}

criterion_group!(benches, serialize_benchmark);
criterion_main!(benches);
