mod batch_upsert;
mod vec_rev_or_deque;
use criterion::{criterion_group, criterion_main};

criterion_group!(benches, batch_upsert::main, vec_rev_or_deque::main);
criterion_main!(benches);
