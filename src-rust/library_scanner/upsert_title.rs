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
    content_file_path: &PathBuf,
) -> Result<(TitleID, ComicInfo)> {
    '_pre_checks: {
        if !content_file_path.exists() {
            return Err(anyhow!("content file not exists"));
        }
        if !content_file_path.is_file() {
            return Err(anyhow!("content file is not a file"));
        }
        if !content_file_path
            .extension()
            .map(|s| s.to_string_lossy() == "zip")
            .unwrap_or(false)
        {
            return Err(anyhow!("content file is not a zip file"));
        }
    }

    // micro optimization, this value is used frequently
    let content_file_path_string = content_file_path.to_string_lossy().to_string();

    let mut archive_file = ArchiveFile::from(content_file_path.clone())
        .context("can't create ArchiveFile from content file")?;

    let mut comic_info = archive_file
        .get_file("CategoryInfo.xml")
        .context("can't get CategoryInfo.xml in content file")
        .and_then(|b| String::from_utf8(b).context("can't decode CategoryInfo.xml in content file"))
        .and_then(|s| ComicInfo::from_str(&s))?;

    let current_content_file_hash = {
        let title_file = tokio::fs::read(&content_file_path).await?;
        match murmur3_32(&mut &title_file[..], 0) {
            Ok(hash) => hash,
            Err(e) => return Err(anyhow!("can't hash content file: {}", e)),
        }
    };
    let title_model = Titles::find()
        .filter(titles::Column::Path.eq(&content_file_path_string))
        .one(&app_state.db)
        .await
        .context("can't find existing title by hash")?;

    if Titles::find()
        .filter(titles::Column::ContentFileHash.eq(current_content_file_hash))
        .count(&app_state.db)
        .await
        .context("can't count existing title by hash")?
        > 1
    {
        warn!("another title is having the same hash as {content_file_path_string}");
    }

    let title_id = title_model
        .as_ref()
        .map(|m| m.id.clone())
        .unwrap_or(TitleID::new());

    let mut active_title_model: titles::ActiveModel = title_model
        .clone()
        .map(|m| m.into())
        .unwrap_or_else(|| titles::ActiveModel {
            id: Set(title_id.clone()),
            title: Set(comic_info.title.to_string()),
            category_id: Set(category_id.clone()),
            author: Set(comic_info.penciller.clone()),
            description: Set(comic_info.summary.clone()),
            // TOOD: put into warning list instead of .ok()
            release: Set(comic_info.get_release().ok()),
            path: Set(content_file_path_string.clone()),

            cover_path: Set(None),
            cover_blurhash: Set(None),
            blurhash_width: Set(None),
            blurhash_height: Set(None),

            content_file_hash: Set(current_content_file_hash),
            // TODO: handle this properly
            comic_info_pages_field_hash: Set(comic_info.get_pages_field_hash().ok().unwrap_or(0)),
            date_added: Set(chrono::Utc::now()),
            date_updated: Set(None),
        });

    if active_title_model.is_changed() {
        debug!("title metadata CHANGED for {content_file_path_string}")
    } else {
        debug!("title metadata NOT CHANGED for {content_file_path_string}")
    }

    if let Some(ref title_model) = title_model {
        if title_model.path != content_file_path_string {
            active_title_model.path = Set(content_file_path_string.clone());
        }
        if title_model.title != comic_info.title.to_string() {
            active_title_model.title = Set(comic_info.title.to_string());
        }
        if title_model.category_id != category_id {
            active_title_model.category_id = Set(category_id.clone());
        }
        if title_model.author != comic_info.penciller.clone() {
            active_title_model.author = Set(comic_info.penciller.clone());
        }
        if title_model.description != comic_info.summary {
            active_title_model.description = Set(comic_info.summary.clone());
        }
        // TOOD: put into warning list instead of .ok()
        if title_model.release != comic_info.get_release().ok() {
            active_title_model.release = Set(comic_info.get_release().ok());
        }
        if title_model.content_file_hash != current_content_file_hash {
            active_title_model.content_file_hash = Set(current_content_file_hash);
        }
    }

    let need_scan_content_file = {
        let content_file_hash_changed = title_model
            .as_ref()
            .map(|m| m.content_file_hash != current_content_file_hash)
            .unwrap_or(true);
        let comic_info_pages_field_hash_changed = title_model
            .as_ref()
            .map(|m| {
                // TODO: handle this properly
                m.comic_info_pages_field_hash != comic_info.get_pages_field_hash().ok().unwrap_or(0)
            })
            .unwrap_or(true);
        content_file_hash_changed || comic_info_pages_field_hash_changed
    };

    let txn = app_state.clone().db.begin().await?;

    if !need_scan_content_file {
        if !active_title_model.is_changed() {
            return Ok((title_id, comic_info));
        }
        active_title_model.date_updated = Set(Some(chrono::Utc::now()));
        if title_model.is_some() {
            active_title_model
                .update(&txn)
                .await
                .context("can't early update title in database")?;
        } else {
            active_title_model
                .insert(&txn)
                .await
                .context("can't early insert title to database")?;
        }
        // TODO: write ComicInfo.xml to content file
        txn.commit()
            .await
            .context("can't commit transaction early")?;
        return Ok((title_id, comic_info));
    }

    let pages_in_content_file = archive_file
        .list_files()
        .await
        .context("can't list files in content file")?
        .into_iter()
        .filter(|p| SUPPORTED_IMAGE_FORMATS.contains_key(p.split('.').last().unwrap_or_default()))
        .collect::<Vec<_>>();

    '_validate_cover_page: {
        // try encode-to-blurhash the configured FrontCover in ComicInfo.toml
        let mut cover_page_blurhash = comic_info
            .pages
            .pages
            .iter()
            .find(|p| p.page_type == ComicPageType::FrontCover)
            .ok_or_else(|| anyhow!("not configured yet"))
            .and_then(|p| {
                p.image_path
                    .as_ref()
                    .ok_or_else(|| anyhow!("ImagePath attribute is empty"))
            })
            .and_then(|path| {
                archive_file
                    .get_file(path)
                    .context("can't extract from content file")
                    .map(|buf| (buf, path))
            })
            .and_then(|(buf, path)| {
                image::load_from_memory(&buf)
                    .context("can't decode")
                    .map(|img| (img, path))
            })
            .and_then(|(img, path)| {
                encode(&img)
                    .context("can't encode to blurhash")
                    .map(|blurhash_result| (blurhash_result, path))
            });

        // try everything else
        if let Err(ref e) = cover_page_blurhash {
            warn!("there's a problem with FrontCover page in ComicInfo.xml for {content_file_path_string}: {e:?}");

            cover_page_blurhash = pages_in_content_file.iter().try_find_map(|page_file_name| {
                archive_file
                    .get_file(page_file_name)
                    .context("can't extract from content file")
                    .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                    .and_then(|img| encode(&img).context("can't encode image to blurhash"))
                    .map(|blurhash_result| (blurhash_result, page_file_name))
            });
        }

        match cover_page_blurhash {
            Ok((blurhash_result, path)) => {
                if let Some(ref title_model) = title_model {
                    if title_model.cover_blurhash.as_ref() != Some(&blurhash_result.blurhash) {
                        active_title_model.cover_blurhash = Set(Some(blurhash_result.blurhash));
                    }
                    if title_model.blurhash_width != Some(blurhash_result.small_width) {
                        active_title_model.blurhash_width = Set(Some(blurhash_result.small_width));
                    }
                    if title_model.blurhash_height != Some(blurhash_result.small_height) {
                        active_title_model.blurhash_height =
                            Set(Some(blurhash_result.small_height));
                    }
                    if title_model.cover_path != Some(path.to_string()) {
                        active_title_model.cover_path = Set(Some(path.to_string()));
                    }
                } else {
                    active_title_model.cover_blurhash = Set(Some(blurhash_result.blurhash));
                    active_title_model.blurhash_width = Set(Some(blurhash_result.small_width));
                    active_title_model.blurhash_height = Set(Some(blurhash_result.small_height));
                    active_title_model.cover_path = Set(Some(path.to_string()));
                }
            }
            Err(ref e) => {
                if let Some(ref title_model) = title_model {
                    if title_model.cover_blurhash.is_some() {
                        active_title_model.cover_blurhash = Set(None);
                    }
                    if title_model.blurhash_width.is_some() {
                        active_title_model.blurhash_width = Set(None);
                    }
                    if title_model.blurhash_height.is_some() {
                        active_title_model.blurhash_height = Set(None);
                    }
                    if title_model.cover_path.is_some() {
                        active_title_model.cover_path = Set(None);
                    }
                } else {
                    active_title_model.cover_blurhash = Set(None);
                    active_title_model.blurhash_width = Set(None);
                    active_title_model.blurhash_height = Set(None);
                    active_title_model.cover_path = Set(None);
                }
                warn!("no cover file for title {content_file_path_string}: {e:?}");
            }
        }
    }

    '_upsert_title_active_and_save_metadata: {
        if active_title_model.is_changed() {
            active_title_model.date_updated = Set(Some(chrono::Utc::now()));
        }
        if title_model.is_some() {
            active_title_model
                .update(&txn)
                .await
                .context("can't update title in database")?;
        } else {
            active_title_model
                .insert(&txn)
                .await
                .context("can't insert title to database")?;
        }
        // TODO: write back metadata to content file
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
