use std::sync::Arc;

use criterion::{black_box, BenchmarkId, Criterion};
use nanoid::nanoid;
use sqlx::{PgPool, Pool, Postgres};
use tokio::runtime::Runtime;

#[derive(serde::Serialize, serde::Deserialize)]
struct Page {
    id: String,
    path: String,
    description: Option<String>,
}

async fn format_method(
    pool: Arc<Pool<Postgres>>,
    pages: Arc<Vec<Page>>,
    title_id: Arc<String>,
) -> Result<(), String> {
    let raw = format!(
        r#"
        INSERT INTO pages_benchmark (id, title_id, path, description)
            VALUES {}
        ON CONFLICT (title_id, path) DO UPDATE
            SET description = EXCLUDED.description"#,
        (0..pages.len())
            .collect::<Vec<_>>()
            .iter()
            .map(|i| {
                let base = ((i + 1) * 3) + 1;
                format!("(${}, $1, ${}, NULLIF(${}, ''))", base - 2, base - 1, base)
            })
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut query = sqlx::query(&raw);
    query = query.bind(title_id.as_ref());
    for item in pages.iter() {
        query = query.bind(item.id.clone());
        query = query.bind(item.path.clone());
        query = query.bind(item.description.clone().unwrap_or_default());
    }
    query
        .execute(pool.as_ref())
        .await
        .map_err(|e| {
            println!("{e:?}");
            e.to_string()
        })
        .map(|_| ())
}

async fn unnest_method(
    pool: Arc<Pool<Postgres>>,
    pages: Arc<Vec<Page>>,
    title_id: Arc<String>,
) -> Result<(), String> {
    let mut page_ids = Vec::with_capacity(pages.len());
    let mut page_paths = Vec::with_capacity(pages.len());
    let mut page_descriptions = Vec::with_capacity(pages.len());

    pages.iter().for_each(|item| {
        page_ids.push(item.id.clone());
        page_paths.push(item.path.clone());
        page_descriptions.push(item.description.clone().unwrap_or_default());
    });

    sqlx::query!(
        r#"
        INSERT INTO pages_benchmark (id, title_id, path, description)
            SELECT id, $1, path, NULLIF(description, '')
            FROM UNNEST($2::text[], $3::text[], $4::text[])
            AS t(id, path, description)
        ON CONFLICT (title_id, path) DO UPDATE
            SET description = EXCLUDED.description
        "#,
        title_id.as_ref(),
        &page_ids,
        &page_paths,
        &page_descriptions
    )
    .execute(pool.as_ref())
    .await
    .map_err(|e| {
        println!("{e:?}");
        e.to_string()
    })
    .map(|_| ())
}

async fn json_to_recordset_method(
    pool: Arc<Pool<Postgres>>,
    pages: Arc<Vec<Page>>,
    title_id: Arc<String>,
) -> Result<(), String> {
    sqlx::query!(
        r#"
        INSERT INTO pages_benchmark (id, title_id, path, description)
            SELECT id, $1, path, NULLIF(description, '')
            FROM json_to_recordset($2::json)
            AS t(id text, path text, description text)
        ON CONFLICT (title_id, path) DO UPDATE
            SET description = EXCLUDED.description;
        "#,
        title_id.as_ref(),
        serde_json::to_value(pages.as_ref()).unwrap()
    )
    .execute(pool.as_ref())
    .await
    .map_err(|e| {
        println!("{e:?}");
        e.to_string()
    })
    .map(|_| ())
}

async fn json_table_method(
    pool: Arc<Pool<Postgres>>,
    pages: Arc<Vec<Page>>,
    title_id: Arc<String>,
) -> Result<(), String> {
    sqlx::query!(
        r#"
        INSERT INTO pages_benchmark (id, title_id, path, description)
            SELECT id, $1, path, NULLIF(description, '')
            FROM json_table($2, '$[*]' COLUMNS (
                id TEXT PATH '$.id',
                path TEXT PATH '$.path',
                description TEXT PATH '$.description'
            ))
        ON CONFLICT (title_id, path) DO UPDATE
            SET description = EXCLUDED.description;
        "#,
        title_id.as_ref(),
        serde_json::to_string(pages.as_ref()).unwrap()
    )
    .execute(pool.as_ref())
    .await
    .map_err(|e| {
        println!("{e:?}");
        e.to_string()
    })
    .map(|_| ())
}

pub fn main(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_upsert");

    let pages = Arc::new({
        let mut pages = Vec::with_capacity(200);
        for _ in 0..100 {
            pages.push(Page {
                id: nanoid!(),
                path: nanoid!(),
                description: Some(nanoid!()),
            });
        }
        for _ in 100..101 {
            pages.push(Page {
                id: nanoid!(),
                path: nanoid!(),
                description: None,
            });
        }
        pages
    });
    let title_id = Arc::new(nanoid!());

    let rt = Runtime::new().unwrap();
    let pool = Arc::new(rt.block_on(async {
        PgPool::connect("postgres://postgres:postgres@localhost:5432/yomuyume")
            .await
            .unwrap()
    }));

    group.bench_with_input(
        BenchmarkId::new("format_method", "pool, pages, title_id"),
        &(pool.clone(), pages.clone(), title_id.clone()),
        |b, param| {
            b.to_async(&rt).iter(|| {
                rt.spawn(format_method(
                    black_box(param.0.clone()),
                    black_box(param.1.clone()),
                    black_box(param.2.clone()),
                ))
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("unnest_method", "pool, pages, title_id"),
        &(pool.clone(), pages.clone(), title_id.clone()),
        |b, param| {
            b.to_async(&rt).iter(|| {
                rt.spawn(unnest_method(
                    black_box(param.0.clone()),
                    black_box(param.1.clone()),
                    black_box(param.2.clone()),
                ))
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("json_to_recordset_method", "pool, pages, title_id"),
        &(pool.clone(), pages.clone(), title_id.clone()),
        |b, param| {
            b.to_async(&rt).iter(|| {
                rt.spawn(json_to_recordset_method(
                    black_box(param.0.clone()),
                    black_box(param.1.clone()),
                    black_box(param.2.clone()),
                ))
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("json_table_method", "pool, pages, title_id"),
        &(pool.clone(), pages.clone(), title_id.clone()),
        |b, param| {
            b.to_async(&rt).iter(|| {
                rt.spawn(json_table_method(
                    black_box(param.0.clone()),
                    black_box(param.1.clone()),
                    black_box(param.2.clone()),
                ))
            })
        },
    );

    group.finish();
}
