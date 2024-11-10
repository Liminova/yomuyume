use std::{path::Path, sync::Arc};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use tracing::warn;

use crate::{
    library_processor::blurhash::encode,
    types::{category_info::CategoryInfo, CategoryID},
    AppState, CATEGORY_INFO_FILENAME,
};

pub(super) async fn upsert_category<'e>(
    app_state: Arc<AppState>,
    category_path: &Path,
    conn: impl sqlx::Executor<'e, Database = sqlx::Postgres> + 'e,
) -> Result<CategoryID> {
    let category_path_string = category_path.to_string_lossy().to_string();

    let category_info_path = category_path.join(CATEGORY_INFO_FILENAME);
    let mut category_info = if category_info_path.exists() {
        CategoryInfo::from_str(
            &std::fs::read_to_string(&category_info_path)
                .context(format!("can't read {CATEGORY_INFO_FILENAME}"))?,
        )
        .context(format!("can't parse {CATEGORY_INFO_FILENAME}"))?
    } else {
        CategoryInfo::default()
    };

    let (cover_path, cover_blurhash, cover_width, cover_height) = 'scoped: {
        let cover = match category_info.cover.as_mut() {
            Some(cover) => cover,
            None => break 'scoped (None, None, None, None),
        };

        let path = match cover.path.as_ref() {
            Some(path) => path,
            None => break 'scoped (None, None, None, None),
        };

        let real_modified_date: Result<DateTime<Utc>> = path
            .metadata()
            .context("can't get metadata of cover file")
            .and_then(|m| {
                m.modified()
                    .context("can't get modified date of cover file")
            })
            .map(|d| d.into());
        if let Err(ref e) = real_modified_date {
            warn!("can't get modified date of cover file for {category_path_string}: {e:?}");
        }

        if let (Some(blurhash), Some(modified_date_at_encode), Ok(real_modified_date)) = (
            cover.blurhash.as_ref(),
            cover.modified_date_at_encode,
            real_modified_date.as_ref(),
        ) {
            let valid_dimensions = cover.width != 0 && cover.height != 0;
            let unmodified = modified_date_at_encode == *real_modified_date;
            if valid_dimensions && unmodified {
                break 'scoped (
                    Some(path.to_string_lossy().to_string()),
                    Some(blurhash.clone()),
                    Some(cover.width),
                    Some(cover.height),
                );
            }
        }

        let blurhash_result = std::fs::read(path)
            .context("can't read cover file")
            .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
            .and_then(|img| encode(&img).context("can't encode image to blurhash"));

        match blurhash_result {
            Ok(blurhash_result) => {
                cover.blurhash = Some(blurhash_result.blurhash.clone());
                cover.width = blurhash_result.width;
                cover.height = blurhash_result.height;
                cover.modified_date_at_encode = real_modified_date.ok();
                break 'scoped (
                    Some(path.to_string_lossy().to_string()),
                    Some(blurhash_result.blurhash.clone()),
                    Some(blurhash_result.width),
                    Some(blurhash_result.height),
                );
            }
            Err(e) => {
                warn!("can't encode the configured cover of {category_path_string}: {e:?}");
            }
        }

        (None, None, None, None)
    };

    '_save_category_info: {
        std::fs::write(&category_info_path, category_info.to_pretty_string()?)
            .context(format!("can't write to {CATEGORY_INFO_FILENAME}"))?;
    }

    let category_id = sqlx::query!(
        r#"
        INSERT INTO "categories"
            ("id", "name", "description", "path",
            "cover_path", "cover_blurhash",
            "cover_width", "cover_height")
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT ("path") DO UPDATE SET
            "name" = EXCLUDED.name,
            "description" = EXCLUDED.description,
            "cover_path" = EXCLUDED.cover_path,
            "cover_blurhash" = EXCLUDED.cover_blurhash,
            "cover_width" = EXCLUDED.cover_width,
            "cover_height" = EXCLUDED.cover_height
        RETURNING id
    "#,
        app_state
            .id_generator
            .snowflake()
            .await
            .context("can't generate category id")?,
        category_info.name.clone().unwrap_or("Untitled".to_string()),
        category_info.description.as_ref(),
        &category_path_string,
        cover_path,
        cover_blurhash,
        cover_width,
        cover_height
    )
    .fetch_one(conn)
    .await
    .context("can't upsert category to database")?
    .id;

    Ok(category_id)
}
