use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

use super::upsert_title::COMICINFO_FILENAME;
use crate::{
    library_processor::blurhash::encode,
    types::{category_info::CategoryInfo, CategoryID},
    AppState,
};

const CATEGORY_INFO_FILENAME: &str = "CategoryInfo.xml";

pub(super) enum UpsertCategoryError {
    DirIsInLibraryRoot,
    DirIsTitle,
    Other(anyhow::Error),
}

impl From<anyhow::Error> for UpsertCategoryError {
    fn from(e: anyhow::Error) -> Self {
        Self::Other(e)
    }
}

/// Decide if a given title path is in any subdirectory of the library path,
/// and if so, upsert the category to the database and return its ID.
pub async fn upsert_category<'e>(
    app_state: Arc<AppState>,
    title_path: &Path,
    conn: impl sqlx::Executor<'e, Database = sqlx::Postgres> + 'e,
) -> Result<CategoryID, UpsertCategoryError> {
    let category_path = match title_path
        .parent()
        .filter(|p| {
            p.strip_prefix(app_state.config.library_path.clone())
                .map(|p| !p.to_string_lossy().to_string().is_empty())
                .unwrap_or(false)
        })
        .map(PathBuf::from)
    {
        Some(category_path) => category_path,
        None => return Err(UpsertCategoryError::DirIsInLibraryRoot),
    };

    // TODO: support directory-as-title
    if category_path.join(COMICINFO_FILENAME).exists() {
        return Err(UpsertCategoryError::DirIsTitle);
    }

    let category_path_string = category_path.to_string_lossy().to_string();

    let category_info_path = category_path.join(CATEGORY_INFO_FILENAME);
    let mut category_info = CategoryInfo::from_str(
        &std::fs::read_to_string(&category_info_path)
            .context(format!("can't read {CATEGORY_INFO_FILENAME}"))
            .map_err(UpsertCategoryError::Other)?,
    )
    .context(format!("can't parse {CATEGORY_INFO_FILENAME}"))
    .map_err(UpsertCategoryError::Other)?;

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
            tracing::warn!(
                "can't get modified date of cover file for {category_path_string}: {e:#?}"
            );
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
                tracing::warn!(
                    "can't encode the configured cover of {category_path_string}: {e:#?}"
                );
            }
        }

        (None, None, None, None)
    };

    '_save_category_info: {
        std::fs::write(
            &category_info_path,
            category_info
                .to_pretty_string()
                .map_err(UpsertCategoryError::Other)?,
        )
        .context(format!("can't write to {CATEGORY_INFO_FILENAME}"))
        .map_err(UpsertCategoryError::Other)?;
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
            .context("can't generate category id")
            .map_err(UpsertCategoryError::Other)?,
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
    .context("can't upsert category to database")
    .map_err(UpsertCategoryError::Other)?
    .id;

    Ok(category_id)
}
