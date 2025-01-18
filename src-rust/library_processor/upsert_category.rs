use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Context;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::{
    app_state::AppState,
    config::CATEGORY_INFO_FILENAME,
    id_generator::GenerateIDErr,
    library_processor::blurhash_encode::encode_blurhash,
    traits::{PathBufUtils, WarnResultThenOk},
    types::{
        absolute_path::{AbsolutePath, AbsolutePathErr},
        category_info::CategoryInfo,
        CategoryID,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum UpsertCategoryErr {
    #[error("can't convert category path to absolute")]
    ConvertToAbsolute(AbsolutePathErr),

    #[error("can't read CategoryInfo.xml: {0:?}")]
    CategoryInfoRead(std::io::Error),
    #[error("can't deserialize CategoryInfo.xml: {0:?}")]
    CategoryInfoParse(quick_xml::DeError),
    #[error("can't serialize CategoryInfo.xml: {0:?}")]
    CategoryInfoSerialize(quick_xml::DeError),
    #[error("can't write CategoryInfo.xml: {0:?}")]
    CategoryInfoWrite(std::io::Error),

    #[error("can't generate an ID for the category: {0:?}")]
    GenerateID(GenerateIDErr),

    #[error("can't upsert category to database: {0:?}")]
    Upsert(sqlx::Error),
}

/// Try to get the category id from the hashmap by its path first, then
/// upsert it to the database if not found, finally return the category id.
///
/// If the category path is `None`, immediately return `None` (syntax sugar for
/// the callers).
pub async fn upsert_category<'e>(
    app_state: &Arc<AppState>,
    category_path: Option<&PathBuf>,
    category_path_to_id: &Arc<RwLock<HashMap<AbsolutePath, i64>>>,
    conn: impl sqlx::Executor<'e, Database = sqlx::Postgres> + 'e,
) -> Result<Option<CategoryID>, UpsertCategoryErr> {
    let category_path = match category_path {
        Some(category_path) => {
            AbsolutePath::from(category_path, None).map_err(UpsertCategoryErr::ConvertToAbsolute)?
        }
        None => return Ok(None),
    };

    #[allow(clippy::significant_drop_in_scrutinee)]
    if let Some(category_id) = category_path_to_id.read().await.get(&category_path) {
        return Ok(Some(*category_id));
    };

    let category_info_path = category_path.as_ref().join(CATEGORY_INFO_FILENAME);
    let (original_category_info, mut category_info) = {
        if category_info_path.exists() {
            let s = std::fs::read_to_string(&category_info_path)
                .map_err(UpsertCategoryErr::CategoryInfoRead)?;
            let tmp = CategoryInfo::from_str(&s).map_err(UpsertCategoryErr::CategoryInfoParse)?;
            (tmp.clone(), tmp)
        } else {
            (CategoryInfo::default(), CategoryInfo::default())
        }
    };

    let mut configured_cover_path: Option<String> = None;
    let mut cover_blurhash: Option<String> = None;
    let mut cover_width: Option<i32> = None;
    let mut cover_height: Option<i32> = None;

    'cover_finder: {
        let Some(cover) = category_info.cover.as_mut() else {
            break 'cover_finder;
        };

        let Some(ref cover_path) = cover.path else {
            break 'cover_finder;
        };

        let Some(cover_path) =
            AbsolutePath::from(&PathBuf::from(cover_path), Some(category_path.as_ref()))
                .okay(|e| warn!("can't convert cover path to absolute: {e:?}"))
        else {
            break 'cover_finder;
        };

        let Some(real_modified_date) = cover_path
            .last_modified()
            .okay(|e| warn!("can't get modified date of cover file: {e:?}"))
        else {
            break 'cover_finder;
        };

        if let Some((blurhash, modified_date_at_encode)) =
            cover.blurhash.as_ref().zip(cover.modified_date_at_encode)
        {
            let valid_dimensions = cover.width != 0 && cover.height != 0;
            let unmodified = modified_date_at_encode == real_modified_date;
            if valid_dimensions && unmodified {
                configured_cover_path = cover.path.clone();
                cover_blurhash = Some(blurhash.clone());
                cover_width = Some(cover.width);
                cover_height = Some(cover.height);
                break 'cover_finder;
            }
        }

        if let Some(blurhash_result) = std::fs::read(cover_path.as_ref())
            .context("can't read image")
            .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
            .and_then(|img| encode_blurhash(&img).context("can't encode image to blurhash"))
            .okay(format!("can't encode `{category_path}` cover"))
        {
            configured_cover_path = cover.path.clone();
            cover.blurhash = Some(blurhash_result.blurhash.clone());
            cover.width = blurhash_result.width;
            cover.height = blurhash_result.height;
            cover.modified_date_at_encode = Some(real_modified_date);

            cover_blurhash = Some(blurhash_result.blurhash);
            cover_width = Some(blurhash_result.width);
            cover_height = Some(blurhash_result.height);
        }
    };

    if original_category_info != category_info {
        let s = category_info
            .to_pretty_string()
            .map_err(UpsertCategoryErr::CategoryInfoSerialize)?;
        std::fs::write(&category_info_path, &s).map_err(UpsertCategoryErr::CategoryInfoWrite)?;
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
            .map_err(UpsertCategoryErr::GenerateID)?,
        category_info.name.clone().unwrap_or("Untitled".to_string()),
        category_info.description.as_ref(),
        category_path.to_string_lossy(),
        configured_cover_path,
        cover_blurhash,
        cover_width,
        cover_height
    )
    .fetch_one(conn)
    .await
    .map_err(UpsertCategoryErr::Upsert)?
    .id;

    category_path_to_id
        .write()
        .await
        .insert(category_path, category_id);

    Ok(Some(category_id))
}
