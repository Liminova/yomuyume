use nanoid::nanoid;
use rayon::prelude::*;

pub fn main(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("nanoid_gen");

    group.bench_function("nanoid", |b| {
        b.iter(|| {
            for _ in 0..100 {
                nanoid!();
            }
        })
    });

    group.bench_function("nanoid_parallel", |b| {
        b.iter(|| {
            (0..100).into_par_iter().for_each(|_| {
                nanoid!();
            })
        })
    });

    group.finish();
}
