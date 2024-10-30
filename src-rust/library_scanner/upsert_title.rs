// NEW AND IMPROVED

use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context, Result};
use murmur3::murmur3_32;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, Set,
    TransactionTrait,
};
use tracing::{debug, warn};
// use zip::ZipArchive;

use crate::{
    library_scanner::{blurhash::encode, comic_info::ComicInfo},
    models::prelude::*,
    types::custom_id::CustomID,
    AppState, ArchiveFile, IteratorExt, SUPPORTED_IMAGE_FORMATS,
};

use super::comic_info::ComicPageType;

/// Upsert a title to the database and return its ID and ComicInfo.
pub async fn upsert_title(
    app_state: Arc<AppState>,
    category_id: Option<CustomID>,
    title_file_path: &PathBuf,
) -> Result<(TitleID, ComicInfo)> {
    '_pre_checks: {
        if !title_file_path.exists() {
            return Err(anyhow!("content file not exists"));
        }
        if !title_file_path.is_file() {
            return Err(anyhow!("content file is not a file"));
        }
        if !title_file_path
            .extension()
            .map(|s| s.to_string_lossy() == "zip")
            .unwrap_or(false)
        {
            return Err(anyhow!("content file is not a zip file"));
        }
    }

    // micro optimization, this value is used frequently
    let title_file_path_string = title_file_path.to_string_lossy().to_string();

    let mut archive_file = ArchiveFile::from(title_file_path.clone())
        .context("can't create ArchiveFile from content file")?;
    let pages_in_archive = archive_file
        .list_files()
        .await
        .context("can't list files in archive")?
        .into_iter()
        .filter(|item| {
            SUPPORTED_IMAGE_FORMATS.contains_key(
                &item
                    .path
                    .split('.')
                    .last()
                    .unwrap_or_default()
                    .to_ascii_lowercase(),
            )
        })
        .collect::<Vec<_>>();

    let mut comic_info = archive_file
        .get_file("ComicInfo.xml")
        .context("can't get ComicInfo.xml in content file")
        .and_then(|b| String::from_utf8(b).context("can't decode ComicInfo.xml in content file"))
        .and_then(|s| ComicInfo::from_str(&s))?;

    let (cover_path, cover_blurhash, cover_width, cover_height): (
        Option<String>,
        Option<String>,
        Option<u32>,
        Option<u32>,
    ) = 'scoped: {
        // if found a FrontCover page in ComicInfo.xml
        if let Some((cover, cover_path)) = comic_info
            .pages_mut()
            .iter_mut()
            .filter(|p| p.page_type == ComicPageType::FrontCover)
            .next()
            .and_then(|p| match p.image_path.clone() {
                Some(path) => Some((p, path)),
                None => None,
            })
        {
            let real_modified_date = pages_in_archive
                .iter()
                .filter(|i| i.path == *cover_path)
                .next()
                .map(|i| i.last_modified);

            if real_modified_date.is_none() {
                warn!("can't get modified date for files inside {title_file_path_string}")
            }

            // and all the following fields are valid
            if let (Some(blurhash), Some(modified_date_at_encode), Some(ref real_modified_date)) = (
                cover.blurhash.as_ref(),
                cover.modified_date_at_encode,
                real_modified_date,
            ) {
                let valid_dimension = cover.image_width > 0 && cover.image_height > 0;
                let modified_date_at_encode = modified_date_at_encode.to_rfc3339();
                let real_modified_date = real_modified_date.to_rfc3339();
                let unmodified = modified_date_at_encode[..27.min(modified_date_at_encode.len())]
                    == real_modified_date[..27.min(real_modified_date.len())];
                // then use them
                if valid_dimension && unmodified {
                    break 'scoped (
                        Some(cover_path.clone()),
                        Some(blurhash.clone()),
                        Some(cover.image_width as u32),
                        Some(cover.image_height as u32),
                    );
                }
            }

            // else re-encode the page
            let blurhash_result = archive_file
                .get_file(&cover_path)
                .context("can't get cover file")
                .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                .and_then(|img| encode(&img).context("can't encode image to blurhash"));
            match blurhash_result {
                Ok(blurhash_result) => {
                    cover.blurhash = Some(blurhash_result.blurhash.clone());
                    cover.image_width = blurhash_result.width as i32;
                    cover.image_height = blurhash_result.height as i32;
                    cover.modified_date_at_encode = real_modified_date;
                    break 'scoped (
                        Some(cover_path.clone()),
                        Some(blurhash_result.blurhash),
                        Some(cover.image_width as u32),
                        Some(cover.image_height as u32),
                    );
                }
                Err(e) => {
                    warn!("can't encode the configured cover of {title_file_path_string}: {e:#}");
                }
            }
        }

        // else try every single pages
        match pages_in_archive.iter().try_find_map(|item| {
            archive_file
                .get_file(&item.path)
                .context("can't get file in archive")
                .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                .and_then(|img| encode(&img).context("can't encode image to blurhash"))
                .map(|blurhash_result| (item, blurhash_result))
        }) {
            Ok((item, blurhash_result)) => {
                comic_info.pages_mut().push(ComicPageInfo {
                    page_type: ComicPageType::FrontCover,
                    blurhash: Some(blurhash_result.blurhash.clone()),
                    image_path: Some(item.path.clone()),
                    image_width: blurhash_result.width as i32,
                    image_height: blurhash_result.height as i32,
                    modified_date_at_encode: Some(item.last_modified),
                    ..Default::default()
                });
                break 'scoped (
                    Some(item.path.clone()),
                    Some(blurhash_result.blurhash),
                    Some(blurhash_result.width),
                    Some(blurhash_result.height),
                );
            }
            Err(e) => {
                warn!("there's no file in {title_file_path_string} that can be encoded to blurhash: {e:#}");
            }
        };

        (None, None, None, None)
    };

    let txn = app_state.clone().db.begin().await?;

    let title_id = Titles::insert(titles::ActiveModel {
        id: Set(TitleID::new()),
        title: Set(comic_info.title.to_string()),
        category_id: Set(category_id.clone()),
        author: Set(comic_info.penciller.clone()),
        description: Set(comic_info.summary.clone()),
        release: Set(comic_info.get_release()),
        path: Set(title_file_path_string.clone()),
        cover_path: Set(cover_path),
        cover_blurhash: Set(cover_blurhash),
        cover_width: Set(cover_width),
        cover_height: Set(cover_height),
        file_hash: Set({
            let title_file = tokio::fs::read(&title_file_path).await?;
            match murmur3_32(&mut &title_file[..], 0) {
                Ok(hash) => hash,
                Err(e) => return Err(anyhow!("can't hash content file: {}", e)),
            }
        }),
        date_added: Set(chrono::Utc::now()),
        date_updated: Set(match tokio::fs::metadata(&title_file_path).await {
            Ok(metadata) => Some(metadata.modified()?.into()),
            Err(e) => {
                warn!("can't get last modified date of {title_file_path_string}: {e:?}");
                None
            }
        }),
    })
    .on_conflict(
        OnConflict::column(titles::Column::Path)
            .update_columns([
                titles::Column::Title,
                titles::Column::CategoryId,
                titles::Column::Author,
                titles::Column::Description,
                titles::Column::Release,
                titles::Column::Path,
                titles::Column::CoverPath,
                titles::Column::CoverBlurhash,
                titles::Column::CoverWidth,
                titles::Column::CoverHeight,
                titles::Column::FileHash,
                titles::Column::DateUpdated,
            ])
            .to_owned(),
    )
    .exec(&txn)
    .await
    .context("can't insert title model to DB")?
    .last_insert_id;

    '_save_comic_info: {
        archive_file
            .upsert_file(
                "CategoryInfo.xml",
                Arc::new(comic_info.to_pretty_string()?.as_bytes().to_vec()),
            )
            .context("can't write back metadata to content file")?;
    }

    '_upsert_pages: {
        debug!("upserting pages");
        let pages_in_content_file_active: Vec<pages::ActiveModel> = pages_in_content_file
            .iter()
            .map(|p| pages::ActiveModel {
                id: Set(PageID::new()),
                title_id: Set(title_id.clone()),
                path: Set(p.clone()),
                description: Set(comic_info.get_page_description(p)),
            })
            .collect();
        Pages::insert_many(pages_in_content_file_active)
            // .on_conflict(
            //     OnConflict::column(pages::Column::Path)
            //         .do_nothing()
            //         .to_owned(),
            // )
            // .on_conflict(
            //     OnConflict::column(pages::Column::TitleId)
            //         .do_nothing()
            //         .to_owned(),
            // )
            .exec(&txn)
            .await
            .context("can't insert pages to DB")?;
        Pages::delete_many()
            .filter(
                Condition::all()
                    .add(pages::Column::TitleId.eq(&title_id))
                    .add(pages::Column::Path.is_not_in(&pages_in_content_file)),
            )
            .exec(&txn)
            .await
            .context("can't delete pages in DB")?;
    }

    // TODO: handle tags

    txn.commit().await.context("can't commit transaction")?;

    Ok((title_id, comic_info))
}

#[cfg(test)]
mod tests {
    use sea_orm::{
        ActiveModelTrait,
        ActiveValue::{NotSet, Set},
        ColumnTrait, Database, EntityTrait, QueryFilter,
    };
    use sea_orm_migration::MigratorTrait;

    use crate::{migrator::Migrator, models::prelude::*};

    #[tokio::test]
    async fn seaorm_set_always_change_active_model() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();

        Tags::insert(tags::ActiveModel {
            id: NotSet,
            name: Set("favorite".to_string()),
        })
        .exec(&db)
        .await
        .unwrap();

        let mut active_model: tags::ActiveModel = Tags::find()
            .filter(tags::Column::Name.eq("favorite"))
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .into();

        assert!(!(active_model.is_changed()));

        active_model.name = Set("favorite".to_string());

        assert!(active_model.is_changed());
    }
}
