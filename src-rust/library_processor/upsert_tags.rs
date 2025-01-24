use futures_util::future::join_all;

use crate::{app_state::AppState, utils::id_generator::GenerateIDErr};

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
    tags: &[String],
    title_id: &i64,
    conn: impl sqlx::Executor<'e, Database = sqlx::Postgres> + 'e,
) -> Result<(), UpsertTagsErr> {
    if tags.is_empty() {
        sqlx::query!(
            "DELETE FROM titles_tags
            WHERE title_id = $1",
            title_id
        )
        .execute(conn)
        .await
        .map_err(UpsertTagsErr::CleanupTags)?;
        return Ok(());
    }

    let tag_ids = join_all((0..tags.len()).map(|_| app_state.id_generator.snowflake()))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    sqlx::query!(
        "WITH tag_upsert AS (
            INSERT INTO tags (id, name)
            SELECT id,
                name
            FROM UNNEST($1::bigint [], $2::text []) AS t(id, name) ON CONFLICT (name) DO
            UPDATE
            SET name = EXCLUDED.name
            RETURNING id
        ) -- clean up old tags
        ,
        cleanup AS (
            DELETE FROM titles_tags
            WHERE title_id = $3
        ) -- insert new relationships
        INSERT INTO titles_tags (title_id, tag_id)
        SELECT $3,
            id
        FROM tag_upsert ON CONFLICT DO NOTHING",
        &tag_ids,
        tags,
        title_id,
    )
    .execute(conn)
    .await
    .map_err(UpsertTagsErr::UpsertTags)?;

    Ok(())
}
