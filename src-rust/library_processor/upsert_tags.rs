use anyhow::{Context, Result};
use futures_util::future::join_all;

use crate::{app_state::AppState, types::comic_info::ComicInfo};

pub async fn upsert_tags<'e>(
    app_state: &AppState,
    comicinfo: &ComicInfo,
    title_id: &i64,
    conn: impl sqlx::Executor<'e, Database = sqlx::Postgres> + 'e,
) -> Result<()> {
    if comicinfo.tags.is_empty() {
        sqlx::query!("DELETE FROM titles_tags WHERE title_id = $1", title_id)
            .execute(conn)
            .await
            .context("can't delete tags")?;
        return Ok(());
    }

    let tag_ids: Result<Vec<i64>> =
        join_all((0..comicinfo.tags.len()).map(|_| app_state.id_generator.snowflake()))
            .await
            .into_iter()
            .collect();
    let tag_ids = tag_ids.context("can't generate enough tag ids")?;

    sqlx::query!(
        "WITH tag_ids AS (
            INSERT INTO tags (id, name)
            SELECT id, name
                FROM UNNEST($1::bigint[], $2::text[])
                AS t(id, name)
            ON CONFLICT (name) DO UPDATE SET
                name = EXCLUDED.name WHERE FALSE
            RETURNING id
        )
        INSERT INTO titles_tags (title_id, tag_id)
            SELECT $3, id
            FROM tag_ids
        ON CONFLICT DO NOTHING",
        &tag_ids,
        &comicinfo.tags,
        title_id,
    )
    .execute(conn)
    .await
    .context("can't upsert tags to database")
    .map(|_| ())
}
