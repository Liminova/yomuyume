use nanoid::nanoid;
use rayon::prelude::*;

pub fn main(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("mut_or_map_unzip");

    let ids_and_descs: Vec<(String, String)> = (0..100)
        .into_par_iter()
        .map(|_| (nanoid!(), nanoid!()))
        .collect();

    group.bench_function("for_loop_mut_var", |b| {
        b.iter(|| {
            let _page_ids = (0..ids_and_descs.len()).into_par_iter().map(|_| nanoid!());
            let mut page_paths = Vec::with_capacity(ids_and_descs.len());
            let mut page_descriptions = Vec::with_capacity(ids_and_descs.len());

            for (id, desc) in ids_and_descs.iter() {
                page_paths.push(id.clone());
                page_descriptions.push(desc.clone());
            }
        })
    });

    group.bench_function("par_iter_and_unzip", |b| {
        b.iter(|| {
            let (_page_ids, _page_paths, _page_descriptions): (Vec<_>, Vec<_>, Vec<_>) =
                itertools::multiunzip(
                    ids_and_descs
                        .par_iter()
                        .map(|(id, desc)| (nanoid!(), id.clone(), desc.clone()))
                        .collect::<Vec<_>>(),
                );
        })
    });

    group.finish();
}
