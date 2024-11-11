use std::collections::HashMap;
use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Timelike, Utc};
use futures_util::future::join_all;
use tokio::sync::Mutex;
use tracing::warn;

use crate::library_processor::upsert_category::upsert_category;
use crate::ItemInArchive;
use crate::{
    library_processor::blurhash::encode,
    types::{
        comic_info::{ComicInfo, ComicPageInfo, ComicPageType},
        TitleID,
    },
    AppState, ArchiveFile, IteratorExt, COMICINFO_FILENAME, SUPPORTED_IMAGE_FORMATS,
};

/// ONLY add errors that would need to handle differently, e.g. [`IsIgnored`]
/// would tell the caller the function "failed" because the file is ignored,
/// not something wrong happened.
///
/// [`IsIgnored`]: UpsertOneshotErr::IsIgnored
#[derive(Debug, thiserror::Error)]
pub enum UpsertOneshotErr {
    #[error("content file is ignored")]
    IsIgnored,
    #[error("content file is empty")]
    IsEmpty,
    #[error("other error: {0:?}")]
    Other(anyhow::Error),
}

impl From<anyhow::Error> for UpsertOneshotErr {
    fn from(e: anyhow::Error) -> Self {
        Self::Other(e)
    }
}

#[derive(Debug)]
pub enum OneshotType {
    InArchive(PathBuf),
    InDirectory(Vec<PathBuf>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct PageInTitle {
    path: String,
    filesize: Option<i64>,
    last_modified: DateTime<Utc>,
}

impl From<ItemInArchive> for PageInTitle {
    fn from(item: ItemInArchive) -> Self {
        PageInTitle {
            path: item.path,
            filesize: item.size,
            last_modified: item.last_modified,
        }
    }
}

impl Ord for PageInTitle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for PageInTitle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Upsert a title to the database and return its ID and ComicInfo.
pub async fn upsert_oneshot(
    app_state: Arc<AppState>,
    title_path: PathBuf,
    oneshot_type: OneshotType,
    parent_path: Option<PathBuf>,
    category_path_to_id: Arc<Mutex<HashMap<PathBuf, i64>>>,
    nomedia_support: bool,
) -> Result<TitleID, UpsertOneshotErr> {
    tracing::debug!("processing {title_path:?}");

    let mut txn = app_state
        .pool
        .begin()
        .await
        .context("can't begin transaction")?;

    // get from provided hashmap if possible,
    // else upsert to db and insert to hashmap
    let category_id: Option<i64> = 'scoped: {
        let parent_path = match parent_path {
            Some(parent_path) => parent_path,
            None => break 'scoped None,
        };
        if let Some(category_id) = category_path_to_id.lock().await.get(&parent_path) {
            break 'scoped Some(*category_id);
        }
        let category_id = upsert_category(app_state.clone(), &parent_path, &mut *txn)
            .await
            .context(format!(
                "can't upsert category {} to database",
                parent_path.display()
            ))?;
        category_path_to_id
            .lock()
            .await
            .insert(parent_path.to_path_buf(), category_id);
        Some(category_id)
    };

    // micro DX optimization, this value is used frequently
    let title_path_string = title_path.to_string_lossy().to_string();

    // - archive: already have filesize & modified date -> filter out non-image files
    // - directory: already have list of images -> get filesize & modified date
    let pages_in_title = match oneshot_type {
        OneshotType::InArchive(ref archive_file) => {
            let files_in_archive = archive_file
                .list_files()
                .context("can't list files in archive")?
                .into_iter()
                .filter_map(|item: ItemInArchive| {
                    match SUPPORTED_IMAGE_FORMATS.contains(
                        &item
                            .path
                            .split('.')
                            .last()
                            .unwrap_or_default()
                            .to_ascii_lowercase()
                            .as_str(),
                    ) {
                        true => Some(item.into()),
                        false => None,
                    }
                })
                .collect::<Vec<_>>();
            if nomedia_support
                && files_in_archive
                    .iter()
                    .any(|item: &PageInTitle| item.path == ".nomedia")
            {
                return Err(UpsertOneshotErr::IsIgnored);
            }
            files_in_archive
        }
        OneshotType::InDirectory(ref files) => {
            let mut files = files
                .iter()
                .try_fold(vec![], |mut acc, item| {
                    let metadata = item.metadata().context("can't get metadata of page file")?;
                    acc.push(PageInTitle {
                        path: item.to_string_lossy().to_string(),
                        filesize: Some(metadata.len() as i64),
                        last_modified: metadata
                            .modified()
                            .context("can't get modified date of page file")?
                            .into(),
                    });
                    Ok::<_, anyhow::Error>(acc)
                })
                .context("can't map page paths to PageInTitle")?;
            files.sort();
            files
        }
    };

    // ignore if is empty
    if pages_in_title.is_empty() {
        return Err(UpsertOneshotErr::IsEmpty);
    }

    // ComicInfo.xml
    let mut comic_info = match oneshot_type {
        OneshotType::InArchive(ref archive_file) => archive_file
            .read_file(COMICINFO_FILENAME)
            .context(format!("can't get {COMICINFO_FILENAME} in content file"))
            .and_then(|b| {
                String::from_utf8(b)
                    .context(format!("can't decode {COMICINFO_FILENAME} in content file"))
            })
            .and_then(|s| ComicInfo::from_str(&s))?,
        OneshotType::InDirectory(_) => {
            let comic_info_path = title_path.join(COMICINFO_FILENAME);
            if !comic_info_path.exists() {
                std::fs::write(&comic_info_path, COMICINFO_FILENAME).context(format!(
                    "can't write to {COMICINFO_FILENAME} in content file"
                ))?;
            }
            ComicInfo::from_str(
                &std::fs::read_to_string(&comic_info_path)
                    .context(format!("can't read {COMICINFO_FILENAME} in content file"))?,
            )?
        }
    };

    let (cover_path, cover_blurhash, cover_width, cover_height): (
        Option<String>,
        Option<String>,
        Option<i32>,
        Option<i32>,
    ) = 'scoped: {
        // if found a FrontCover page in ComicInfo.xml
        if let Some((cover_page_info, cover_path_string)) = comic_info
            .pages_mut()
            .iter_mut()
            .rev()
            .filter(|p| p.page_type == ComicPageType::FrontCover)
            .filter_map(|p| p.image_path.clone().map(|path| (p, path)))
            .find(|(_, page_path)| match &oneshot_type {
                OneshotType::InArchive(_) => pages_in_title.iter().any(|i| &i.path == page_path),
                OneshotType::InDirectory(pages_paths) => {
                    pages_paths.contains(&PathBuf::from(&page_path))
                }
            })
        {
            let real_modified_date = pages_in_title
                .iter()
                .find(|i| i.path == cover_path_string)
                .ok_or_else(|| anyhow!("file doesn't exist in archive"))
                .and_then(|i| {
                    i.last_modified
                        .with_nanosecond(0)
                        .ok_or_else(|| anyhow!("can't round the modified date {}", i.last_modified))
                });

            if let Err(ref e) = real_modified_date {
                warn!("can't get modified date for {cover_path_string} inside {title_path_string}: {e:?}")
            }

            // and all the following fields are valid
            if let (Some(blurhash), Some(modified_date_at_encode), Ok(real_modified_date)) = (
                cover_page_info.blurhash.as_ref(),
                cover_page_info.modified_date_at_encode.as_ref(),
                real_modified_date.as_ref(),
            ) {
                let valid_dimension =
                    cover_page_info.image_width > 0 && cover_page_info.image_height > 0;
                let unmodified = modified_date_at_encode == real_modified_date;
                // then use them
                if valid_dimension && unmodified {
                    break 'scoped (
                        Some(cover_path_string),
                        Some(blurhash.clone()),
                        Some(cover_page_info.image_width),
                        Some(cover_page_info.image_height),
                    );
                }
            }

            // else re-encode the page
            let blurhash_result = match oneshot_type {
                OneshotType::InArchive(ref archive_file) => archive_file
                    .read_file(&cover_path_string)
                    .context("can't get cover file"),
                OneshotType::InDirectory(_) => {
                    std::fs::read(&cover_path_string).context("can't get cover file")
                }
            };

            match blurhash_result
                .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                .and_then(|img| encode(&img).context("can't encode image to blurhash"))
            {
                Ok(blurhash_result) => {
                    cover_page_info.blurhash = Some(blurhash_result.blurhash.clone());
                    cover_page_info.image_width = blurhash_result.width;
                    cover_page_info.image_height = blurhash_result.height;
                    cover_page_info.modified_date_at_encode = real_modified_date.ok();
                    break 'scoped (
                        Some(cover_path_string),
                        Some(blurhash_result.blurhash),
                        Some(cover_page_info.image_width),
                        Some(cover_page_info.image_height),
                    );
                }
                Err(e) => {
                    warn!("can't encode {cover_path_string} in {title_path_string} to blurhash: {e:?}");
                }
            }
        }

        // else try every single pages
        let result = match oneshot_type {
            OneshotType::InArchive(ref archive_file) => {
                pages_in_title.iter().try_find_map(|item| {
                    archive_file
                        .read_file(&item.path)
                        .context("can't get file in archive")
                        .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                        .and_then(|img| encode(&img).context("can't encode image to blurhash"))
                        .map(|blurhash_result| {
                            (item.path.clone(), item.last_modified, blurhash_result)
                        })
                })
            }
            OneshotType::InDirectory(_) => pages_in_title.iter().try_find_map(|page| {
                let page_last_modified = std::fs::metadata(page.path.clone())
                    .and_then(|m| m.modified().map(DateTime::<Utc>::from))
                    .context("can't get modified date of page file")?;

                std::fs::read(&page.path)
                    .context("can't get file in directory")
                    .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                    .and_then(|img| encode(&img).context("can't encode image to blurhash"))
                    .map(|blurhash_result| (page.path.clone(), page_last_modified, blurhash_result))
            }),
        };
        match result {
            Ok((page_path, page_last_modified, blurhash_result)) => {
                comic_info.pages_mut().push(ComicPageInfo {
                    page_type: ComicPageType::FrontCover,
                    blurhash: Some(blurhash_result.blurhash.clone()),
                    image_path: Some(page_path.clone()),
                    image_width: blurhash_result.width,
                    image_height: blurhash_result.height,
                    modified_date_at_encode: Some(page_last_modified),
                    ..Default::default()
                });
                (
                    Some(page_path),
                    Some(blurhash_result.blurhash),
                    Some(blurhash_result.width),
                    Some(blurhash_result.height),
                )
            }
            Err(e) => {
                warn!(
                    "there's no file in {title_path_string} that can be encoded to blurhash: {e:?}"
                );
                (None, None, None, None)
            }
        }
    };

    // no longer need to modify anything in the ComicInfo.xml
    // beside the cover page, so we can write it back here
    match oneshot_type {
        OneshotType::InArchive(ref archive_file) => {
            archive_file
                .upsert_file(
                    COMICINFO_FILENAME,
                    Arc::new(comic_info.to_pretty_string()?.as_bytes().to_vec()),
                )
                .context("can't write back metadata to content file")?;
        }
        OneshotType::InDirectory(_) => {
            std::fs::write(
                title_path.join(COMICINFO_FILENAME),
                comic_info.to_pretty_string()?.as_bytes(),
            )
            .context("can't write back metadata to content file")?;
        }
    }

    // upsert title and get its id
    let title_id = sqlx::query!(
        "INSERT INTO titles
            (id, title, category_id, author, description, release,
            path, is_dir, cover_path, cover_blurhash, cover_width,
            cover_height, date_updated)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ON CONFLICT (path)
        DO UPDATE SET
            title = EXCLUDED.title,
            category_id = EXCLUDED.category_id,
            author = EXCLUDED.author,
            description = EXCLUDED.description,
            release = EXCLUDED.release,
            path = EXCLUDED.path,
            is_dir = EXCLUDED.is_dir,
            cover_path = EXCLUDED.cover_path,
            cover_blurhash = EXCLUDED.cover_blurhash,
            cover_width = EXCLUDED.cover_width,
            cover_height = EXCLUDED.cover_height,
            date_updated = EXCLUDED.date_updated
        RETURNING id",
        app_state.id_generator.snowflake().await?,
        comic_info.title,
        category_id,
        comic_info.penciller.as_ref(),
        comic_info.summary.as_ref(),
        comic_info.get_release(),
        title_path_string.to_string(),
        match oneshot_type {
            OneshotType::InArchive(_) => false,
            OneshotType::InDirectory(_) => true,
        },
        cover_path,
        cover_blurhash,
        cover_width,
        cover_height,
        std::fs::metadata(&title_path)
            .and_then(|m| { m.modified().map(|d| DateTime::<Utc>::from(d)) })
            .map_err(|e| tracing::warn!("can't get modified date of {title_path_string}: {e:?}"))
            .ok()
            .unwrap_or_default(),
    )
    .fetch_one(&mut *txn)
    .await
    .context("can't upsert title model to DB")?
    .id;

    '_upsert_pages: {
        let page_ids: Result<Vec<i64>> =
            join_all((0..pages_in_title.len()).map(|_| app_state.id_generator.snowflake()))
                .await
                .into_iter()
                .collect();
        let page_ids = page_ids.context("can't generate enough page ids")?;

        let mut page_paths = Vec::with_capacity(pages_in_title.len());
        let mut page_filesizes = Vec::with_capacity(pages_in_title.len());
        let mut page_descriptions = Vec::with_capacity(pages_in_title.len());

        for item in pages_in_title.iter() {
            page_paths.push(item.path.clone());
            page_filesizes.push(item.filesize.unwrap_or_default());
            page_descriptions.push(
                comic_info
                    .get_page_description(&item.path)
                    .unwrap_or_default(),
            );
        }

        sqlx::query!(
            "WITH _ AS (
            INSERT INTO pages (id, title_id, path, filesize, description)
            SELECT id, $1, path, NULLIF(filesize, 0), NULLIF(description, '')
                FROM UNNEST($2::bigint[], $3::text[], $4::bigint[], $5::text[])
                AS t(id, path, filesize, description)
            ON CONFLICT (title_id, path) DO UPDATE
                SET description = EXCLUDED.description
            )
            DELETE FROM pages WHERE title_id = $1
                AND path NOT IN (SELECT UNNEST($3::text[]))",
            title_id,
            &page_ids,
            &page_paths,
            &page_filesizes,
            &page_descriptions
        )
        .execute(&mut *txn)
        .await
        .context("can't upsert new pages")?;
    }

    'upsert_tags: {
        if comic_info.tags.is_empty() {
            sqlx::query!("DELETE FROM titles_tags WHERE title_id = $1", title_id)
                .execute(&mut *txn)
                .await
                .context("can't delete tags")?;
            break 'upsert_tags;
        }

        let tag_ids: Result<Vec<i64>> =
            join_all((0..comic_info.tags.len()).map(|_| app_state.id_generator.snowflake()))
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
            &comic_info.tags,
            title_id,
        )
        .execute(&mut *txn)
        .await
        .context("can't insert tags to database")?;
    }

    txn.commit().await.context("can't commit transaction")?;

    Ok(title_id)
}
