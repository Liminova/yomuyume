mod batch_upsert;
mod mut_or_map_unzip;
mod nanoid_gen;
use criterion::{criterion_group, criterion_main};

criterion_group!(
    benches,
    batch_upsert::main,
    nanoid_gen::main,
    mut_or_map_unzip::main
);
criterion_main!(benches);
