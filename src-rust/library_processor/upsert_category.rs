use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tracing::warn;

use crate::{
    app_state::AppState,
    config::CATEGORY_INFO_FILENAME,
    library_processor::blurhash::encode,
    types::{absolute_path::AbsolutePath, category_info::CategoryInfo, CategoryID},
};

/// Try to get the category id from the hashmap by its path first, then
/// upsert it to the database if not found, finally return the category id.
///
/// If the category path is `None`, immediately return `None` (syntax sugar for
/// the callers).
pub async fn upsert_category<'e>(
    app_state: &Arc<AppState>,
    category_path: &Option<PathBuf>,
    category_path_to_id: &Arc<RwLock<HashMap<AbsolutePath, i64>>>,
    conn: impl sqlx::Executor<'e, Database = sqlx::Postgres> + 'e,
) -> Result<Option<CategoryID>> {
    let category_path = match category_path {
        Some(category_path) => {
            AbsolutePath::from(category_path, None).context("can't convert path to absolute")?
        }
        None => return Ok(None),
    };

    if let Some(category_id) = (*category_path_to_id.read().await).get(&category_path) {
        return Ok(Some(*category_id));
    };

    let category_path_string = category_path.to_string_lossy().to_string();

    let category_info_path = category_path.as_ref().join(CATEGORY_INFO_FILENAME);
    let mut category_info = if category_info_path.exists() {
        CategoryInfo::from_str(
            &std::fs::read_to_string(&category_info_path)
                .context(format!("can't read {CATEGORY_INFO_FILENAME}"))?,
        )
        .context(format!("can't parse {CATEGORY_INFO_FILENAME}"))?
    } else {
        CategoryInfo::default()
    };

    let mut configured_cover_path: Option<String> = None;
    let mut cover_blurhash: Option<String> = None;
    let mut cover_width: Option<i32> = None;
    let mut cover_height: Option<i32> = None;

    'cover_finder: {
        let cover = match category_info.cover.as_mut() {
            Some(cover) => cover,
            None => break 'cover_finder,
        };

        let cover_path = match cover.path {
            Some(ref p) => p,
            None => break 'cover_finder,
        };

        let cover_path =
            match AbsolutePath::from(&PathBuf::from(cover_path), Some(category_path.as_ref())) {
                Ok(p) => p,
                Err(e) => {
                    warn!(
                        "can't convert cover path `{}` to absolute: {e:?}",
                        cover_path
                    );
                    break 'cover_finder;
                }
            };

        let real_modified_date: DateTime<Utc> = match cover_path
            .as_ref()
            .metadata()
            .and_then(|m| m.modified())
            .map(|d| d.into())
        {
            Ok(d) => d,
            Err(e) => {
                warn!("can't get modified date of cover file for `{category_path_string}`: {e:?}");
                break 'cover_finder;
            }
        };

        if let (Some(blurhash), Some(modified_date_at_encode)) =
            (cover.blurhash.as_ref(), cover.modified_date_at_encode)
        {
            let valid_dimensions = cover.width != 0 && cover.height != 0;
            let unmodified = modified_date_at_encode == real_modified_date;
            if valid_dimensions && unmodified {
                cover_blurhash = Some(blurhash.clone());
                cover_width = Some(cover.width);
                cover_height = Some(cover.height);
            }
        }

        let blurhash_result = std::fs::read(cover_path.as_ref())
            .context("can't read cover file")
            .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
            .and_then(|img| encode(&img).context("can't encode image to blurhash"));

        match blurhash_result {
            Ok(blurhash_result) => {
                cover.blurhash = Some(blurhash_result.blurhash.clone());
                cover.width = blurhash_result.width;
                cover.height = blurhash_result.height;
                cover.modified_date_at_encode = Some(real_modified_date);

                cover_blurhash = Some(blurhash_result.blurhash);
                cover_width = Some(blurhash_result.width);
                cover_height = Some(blurhash_result.height);

                break 'cover_finder;
            }
            Err(e) => {
                configured_cover_path = None;
                warn!("can't encode the configured cover of `{category_path_string}`: {e:?}");
            }
        }
    };

    '_save_category_info: {
        std::fs::write(&category_info_path, category_info.to_pretty_string()?)
            .context(format!("can't write to {CATEGORY_INFO_FILENAME}"))?;
    }

    let category_id = sqlx::query!(
        "INSERT INTO categories
            (id, name, description, path,
            cover_path, cover_blurhash,
            cover_width, cover_height)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (path) DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            cover_path = EXCLUDED.cover_path,
            cover_blurhash = EXCLUDED.cover_blurhash,
            cover_width = EXCLUDED.cover_width,
            cover_height = EXCLUDED.cover_height
        RETURNING id",
        app_state
            .id_generator
            .snowflake()
            .await
            .context("can't generate category id")?,
        category_info.name.clone().unwrap_or("Untitled".to_string()),
        category_info.description.as_ref(),
        &category_path_string,
        configured_cover_path,
        cover_blurhash,
        cover_width,
        cover_height
    )
    .fetch_one(conn)
    .await
    .context(format!(
        "can't upsert category `{}` to database",
        category_path.as_ref().display()
    ))?
    .id;

    category_path_to_id
        .write()
        .await
        .insert(category_path, category_id);

    Ok(Some(category_id))
}
