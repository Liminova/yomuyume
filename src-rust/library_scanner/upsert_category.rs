use std::{path::Path, sync::Arc};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use tracing::warn;

use crate::{
    library_scanner::blurhash::encode,
    types::{category_info::CategoryInfo, CategoryID},
    AppState,
};

const CATEGORY_INFO_FILENAME: &str = "CategoryInfo.xml";

/// Upsert a category to the database and return its ID.
pub async fn upsert_category(
    app_state: Arc<AppState>,
    category_dir_path: &Path,
) -> Result<CategoryID> {
    '_pre_checks: {
        if !category_dir_path.exists() {
            return Err(anyhow!("category dir not found"));
        }
        if !category_dir_path.is_dir() {
            return Err(anyhow!("category dir is not a directory"));
        }
    }

    let category_dir_path_string = category_dir_path.to_string_lossy().to_string();

    let category_info_path = category_dir_path.join(CATEGORY_INFO_FILENAME);
    let mut category_info = CategoryInfo::from_str(
        &std::fs::read_to_string(&category_info_path)
            .context(format!("can't read {CATEGORY_INFO_FILENAME}"))?,
    )
    .context(format!("can't parse {CATEGORY_INFO_FILENAME}"))?;

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
            warn!("can't get modified date of cover file for {category_dir_path_string}: {e:#}");
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

        let blurhash_result = tokio::fs::read(&path)
            .await
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
                warn!("can't encode the configured cover of {category_dir_path_string}: {e:#}");
            }
        }

        (None, None, None, None)
    };

    let category_id = sqlx::query!(
        r#"
        INSERT INTO "categories"
            ("id", "name", "description",
            "cover_path", "cover_blurhash",
            "cover_width", "cover_height")
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT ("id") DO UPDATE SET
            "name" = EXCLUDED.name,
            "description" = EXCLUDED.description,
            "cover_path" = EXCLUDED.cover_path,
            "cover_blurhash" = EXCLUDED.cover_blurhash,
            "cover_width" = EXCLUDED.cover_width,
            "cover_height" = EXCLUDED.cover_height
        RETURNING id
    "#,
        category_info.id.as_ref(),
        category_info.name.clone().unwrap_or("Untitled".to_string()),
        category_info.description.as_ref(),
        cover_path,
        cover_blurhash,
        cover_width.map(|w| w as i32),
        cover_height.map(|h| h as i32)
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't upsert category to database")?
    .id;

    category_info.id = CategoryID::from(category_id)
        .context("can't convert back category ID got from DB back to CategoryID")?;

    '_save_category_info: {
        std::fs::write(&category_info_path, category_info.to_pretty_string()?)
            .context("can't write to CategoryInfo.xml")?;
    }

    Ok(category_info.id)
}
