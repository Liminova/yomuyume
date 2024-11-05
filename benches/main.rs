mod batch_upsert;
use criterion::{criterion_group, criterion_main};

criterion_group!(benches, batch_upsert::main);
criterion_main!(benches);
