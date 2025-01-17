use futures_util::future::join_all;

use crate::{app_state::AppState, id_generator::GenerateIDErr, types::comic_info::ComicInfo};

#[derive(Debug, thiserror::Error)]
pub enum UpsertTagsErr {
    #[error("can't generate enough tag ids: {0:?}")]
    NotEnoughTagIDs(#[from] GenerateIDErr),

    #[error("can't delete old tags of title: {0:?}")]
    CleanupTags(sqlx::Error),

    #[error("can't upsert tags to database: {0:?}")]
    UpsertTags(sqlx::Error),
}

pub async fn upsert_tags<'e>(
    app_state: &AppState,
    comicinfo: &ComicInfo,
    title_id: &i64,
    conn: impl sqlx::Executor<'e, Database = sqlx::Postgres> + 'e,
) -> Result<(), UpsertTagsErr> {
    if comicinfo.tags.is_empty() {
        sqlx::query!("DELETE FROM titles_tags WHERE title_id = $1", title_id)
            .execute(conn)
            .await
            .map_err(UpsertTagsErr::CleanupTags)?;
        return Ok(());
    }

    let tag_ids = join_all((0..comicinfo.tags.len()).map(|_| app_state.id_generator.snowflake()))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

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
    .map_err(UpsertTagsErr::UpsertTags)?;

    Ok(())
}
