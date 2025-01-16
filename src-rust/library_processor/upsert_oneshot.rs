use std::collections::HashMap;
use std::fs::DirEntry;
use std::os::unix::fs::MetadataExt;
use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use futures_util::future::join_all;
use tokio::sync::RwLock;

use crate::app_state::AppState;
use crate::archive_file::{ArchiveFile, ItemInArchive, ItemsInArchiveUtils};
use crate::config::COMICINFO_FILENAME;
use crate::macros::bail_if_empty;
use crate::traits::{IteratorExt, PathBufUtils, WarnResultThenOk};
use crate::types::absolute_path::AbsolutePath;
use crate::{
    library_processor::{
        blurhash::encode, upsert_category::upsert_category, upsert_tags::upsert_tags, PageInTitle,
        UpsertTitleError,
    },
    types::{
        comic_info::{ComicInfo, ComicPageInfo, ComicPageType},
        TitleID,
    },
};

#[derive(Debug)]
pub enum OneshotType {
    Directory(Vec<DirEntry>),
    Archive(Vec<ItemInArchive>),
}

/// upsert a oneshot to the database and return its ID
pub async fn upsert_oneshot(
    app_state: Arc<AppState>,
    title_path: PathBuf,
    oneshot_type: OneshotType,
    parent_path: Option<PathBuf>,
    category_path_to_id: Arc<RwLock<HashMap<AbsolutePath, i64>>>,
    nomedia_support: bool,
) -> Result<TitleID, UpsertTitleError> {
    let title_path = AbsolutePath::from(&title_path, None).context(format!(
        "can't convert `{}` to absolute",
        title_path.display()
    ))?;

    // for DX reasons
    let title_path_str = title_path.to_string_lossy().to_string();

    let mut comicinfo_path = PathBuf::from("");
    let mut comicinfo_path_str = String::from("");
    let mut comicinfo = match oneshot_type {
        OneshotType::Archive(_) => title_path
            .as_ref()
            .read_file_from_archive(COMICINFO_FILENAME)
            .context("can't extract file from archive")
            .and_then(|b| String::from_utf8(b).context("can't decode content to string"))
            .and_then(|s| ComicInfo::from_str(&s))
            .context("can't parse ComicInfo.xml from archive")?,

        OneshotType::Directory(_) => {
            comicinfo_path = title_path.as_ref().join(COMICINFO_FILENAME);
            comicinfo_path_str = comicinfo_path.to_string_lossy().to_string();

            if !comicinfo_path.exists() {
                std::fs::write(&comicinfo_path, COMICINFO_FILENAME)
                    .context(format!("can't write to `{comicinfo_path_str}`"))?;
            }
            ComicInfo::from_str(
                &std::fs::read_to_string(&comicinfo_path)
                    .context(format!("can't parse `{comicinfo_path_str}`"))?,
            )?
        }
    };

    let mut pages_in_title: Vec<PageInTitle> = Vec::with_capacity(match oneshot_type {
        OneshotType::Archive(ref files_in_archive) => files_in_archive.len(),
        OneshotType::Directory(ref files) => files.len(),
    });

    let mut title_last_modified = None;

    let mut cover_path = None;
    let mut cover_blurhash = None;
    let mut cover_width = None;
    let mut cover_height = None;

    'handle_as_archive: {
        let files_in_archive = match oneshot_type {
            OneshotType::Archive(ref files_in_archive) => files_in_archive,
            OneshotType::Directory(_) => break 'handle_as_archive,
        };

        title_last_modified = title_path
            .as_ref()
            .last_modified()
            .okay(format!("can't get last modified date of {title_path_str}"));

        let pages_in_archive = files_in_archive.keep_images(nomedia_support);

        bail_if_empty!(pages_in_archive, Err(UpsertTitleError::IsEmpty));

        'cover_finder: {
            // check if there's a configured cover
            let mut page_modified = None;
            let page_config_and_path = comicinfo
                .pages_mut()
                .iter_mut()
                .rev()
                .filter(|p| p.page_type == ComicPageType::FrontCover)
                .filter_map(|pc| {
                    pc.image_path
                        .clone()
                        .map(|page_config_path| (pc, page_config_path))
                })
                .find(|(_, page_config_path)| {
                    pages_in_archive
                        .iter()
                        .any(|p| match p.path == *page_config_path {
                            true => {
                                page_modified = Some(p.last_modified);
                                true
                            }
                            false => false,
                        })
                });

            // and use it if the fields are valid or encode-able to blurhash
            '_use_configured_one: {
                let (modified, (page_config, page_path)) =
                    match page_modified.zip(page_config_and_path) {
                        Some((a, b)) => (a, b),
                        _ => break '_use_configured_one,
                    };

                if page_config
                    .blurhash
                    .as_ref()
                    .zip(page_config.modified_date_at_encode.as_ref())
                    .filter(|(_, m)| {
                        let valid_dimension =
                            page_config.image_width > 0 && page_config.image_height > 0;
                        let unmodified = **m == modified;

                        valid_dimension && unmodified
                    })
                    .map(|(bh, _)| {
                        cover_path = Some(page_path.clone());
                        cover_blurhash = Some(bh.clone());
                        cover_width = Some(page_config.image_width);
                        cover_height = Some(page_config.image_height);
                    })
                    .is_some()
                {
                    break 'cover_finder;
                };

                // try re-encode if something went wrong
                if title_path
                    .as_ref()
                    .read_file_from_archive(&page_path)
                    .context("can't extract configured cover file from archive")
                    .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                    .and_then(|img| encode(&img).context("can't encode image to blurhash"))
                    .okay(format!(
                        "can't use `{page_path}` as cover for `{title_path_str}`"
                    ))
                    .map(|blurhash_result| {
                        page_config.blurhash = Some(blurhash_result.blurhash.clone());
                        page_config.image_width = blurhash_result.width;
                        page_config.image_height = blurhash_result.height;
                        page_config.modified_date_at_encode = Some(modified);

                        cover_path = Some(page_path);
                        cover_blurhash = Some(blurhash_result.blurhash);
                        cover_width = Some(page_config.image_width);
                        cover_height = Some(page_config.image_height);
                    })
                    .is_some()
                {
                    break 'cover_finder;
                }
            }

            // else try every single pages
            pages_in_archive
                .iter()
                .try_find_map(|item| {
                    title_path
                        .as_ref()
                        .read_file_from_archive(&item.path)
                        .context("can't get file in archive")
                        .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                        .and_then(|img| encode(&img).context("can't encode image to blurhash"))
                        .map(|blurhash_result| {
                            (item.path.clone(), item.last_modified, blurhash_result)
                        })
                })
                .map(|(path, modified, blurhash)| {
                    comicinfo.pages_mut().push(ComicPageInfo {
                        page_type: ComicPageType::FrontCover,
                        blurhash: Some(blurhash.blurhash.clone()),
                        image_path: Some(path.clone()),
                        image_width: blurhash.width,
                        image_height: blurhash.height,
                        modified_date_at_encode: Some(modified),
                        ..Default::default()
                    });
                    cover_path = Some(path);
                    cover_blurhash = Some(blurhash.blurhash);
                    cover_width = Some(blurhash.width);
                    cover_height = Some(blurhash.height);
                })
                .okay(format!(
                    "no file in `{title_path_str}` can be encoded to blurhash"
                ));
        };

        // save ComicInfo.xml
        comicinfo
            .to_pretty_string()
            .context(format!("can't serialize {COMICINFO_FILENAME}"))
            .and_then(|s| {
                title_path
                    .as_ref()
                    .upsert_file_to_archive(COMICINFO_FILENAME, Arc::new(s.as_bytes().to_vec()))
                    .map_err(|e| anyhow!("{e:?}"))
            })
            .okay(format!(
                "can't write `{COMICINFO_FILENAME}` back to title `{title_path_str}`"
            ));

        pages_in_title = pages_in_archive.iter().cloned().cloned().collect();
    }

    'handle_as_directory: {
        let sub_entries = match oneshot_type {
            OneshotType::Archive(_) => break 'handle_as_directory,
            OneshotType::Directory(ref sub_entries) => sub_entries,
        };

        title_last_modified = comicinfo_path
            .last_modified()
            .okay(format!("can't get modified date of {comicinfo_path_str}"));

        type PageLastModified = DateTime<Utc>;
        type PageFileSize = i64;

        // **ABSOLUTE** image paths
        let mut pages_in_dir: Vec<(AbsolutePath, PageLastModified, PageFileSize)> = vec![];

        // this entire for loop is just for mapping the list of sub-entries of
        // the title's directory to the list of page paths along with their last
        // modified date and file size
        sub_entries
            .iter()
            .filter_map(|e| {
                AbsolutePath::from(&e.path(), None).okay(format!(
                    "can't convert `{}` to absolute",
                    e.path().display()
                ))
            })
            .filter_map(|p| {
                p.as_ref()
                    .metadata()
                    .okay(format!("can't get metadata of `{p}`"))
                    .map(|m| (p, m))
            })
            .filter_map(|(p, m)| {
                p.as_ref()
                    .last_modified()
                    .okay(format!("can't get last modified date of `{p}`"))
                    .map(|d| (p, d, m.size() as i64))
            })
            .for_each(|(path, modified, size)| {
                if path.as_ref().is_dir() {
                    if path.as_ref().has_image_ext() {
                        pages_in_dir.push((path, modified, size));
                    }
                    return;
                }
                pages_in_dir.extend(
                    path.as_ref()
                        .scan_dir_recursively_for_image(nomedia_support)
                        .iter()
                        .filter_map(|p| {
                            AbsolutePath::from(p, None)
                                .okay(format!("can't convert `{}` to absolute", p.display()))
                        })
                        .filter_map(|p| {
                            p.as_ref()
                                .metadata()
                                .okay(format!("can't get metadata of `{p}`"))
                                .map(|m| (p, m))
                        })
                        .filter_map(|(p, m)| {
                            p.as_ref()
                                .last_modified()
                                .okay(format!("can't get last modified date of `{p}`"))
                                .map(|d| (p, d, m.size() as i64))
                        }),
                );
            });

        bail_if_empty!(pages_in_dir, Err(UpsertTitleError::IsEmpty));

        if let Some(m) = pages_in_dir
            .iter()
            .map(|(_, last_modified, _)| last_modified)
            .max()
            .cloned()
        {
            title_last_modified = Some(m);
        };

        'cover_finder: {
            // check if there's a configured cover
            let mut page_abs_path = None;
            let mut page_modified = None;
            let page_config_and_path = comicinfo
                .pages_mut()
                .iter_mut()
                .rev()
                .filter(|p| p.page_type == ComicPageType::FrontCover)
                .filter_map(|page_config| {
                    page_config
                        .image_path
                        .clone()
                        .map(|page_config_path| (page_config, page_config_path))
                })
                .find(|(_, page_config_path)| {
                    pages_in_dir
                        .iter()
                        .filter_map(|(ap, m, _)| {
                            ap.to_relative(Some(&title_path))
                                .okay(format!("can't convert `{ap}` to relative to title's path"))
                                .map(|p| (ap, p.to_string_lossy().to_string(), m))
                        })
                        .any(|(ap, p, m)| match p == *page_config_path {
                            true => {
                                page_abs_path = Some(ap.clone());
                                page_modified = Some(m);
                                true
                            }
                            false => false,
                        })
                });

            // and use it if the fields are valid or encode-able to blurhash
            'use_configured_one: {
                let ((page_config, page_config_path), (page_abs_path, page_modified)) =
                    match page_config_and_path.zip(page_abs_path.zip(page_modified)) {
                        Some(p) => p,
                        _ => break 'use_configured_one,
                    };

                // use the configured page if lgtm
                if page_config
                    .blurhash
                    .as_ref()
                    .zip(page_config.modified_date_at_encode.as_ref())
                    .filter(|(_, m)| {
                        let valid_dimension =
                            page_config.image_width > 0 && page_config.image_height > 0;
                        let unmodified = **m == *page_modified;

                        valid_dimension && unmodified
                    })
                    .map(|(bh, _)| {
                        cover_path = Some(page_config_path.clone());
                        cover_blurhash = Some(bh.clone());
                        cover_width = Some(page_config.image_width);
                        cover_height = Some(page_config.image_height);
                    })
                    .is_some()
                {
                    break 'cover_finder;
                }

                // try re-encode the configured page
                if std::fs::read(page_abs_path.as_ref())
                    .context("can't read file")
                    .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                    .and_then(|img| encode(&img).context("can't encode to blurhash"))
                    .okay(format!(
                        "can't use `{page_abs_path}` as cover for `{title_path_str}`"
                    ))
                    .map(|bh_result| {
                        page_config.blurhash = Some(bh_result.blurhash.clone());
                        page_config.image_width = bh_result.width;
                        page_config.image_height = bh_result.height;
                        page_config.modified_date_at_encode = Some(*page_modified);

                        cover_path = Some(page_config_path);
                        cover_blurhash = Some(bh_result.blurhash);
                        cover_width = Some(page_config.image_width);
                        cover_height = Some(page_config.image_height);
                    })
                    .is_some()
                {
                    break 'cover_finder;
                }
            }

            // else try every single pages
            pages_in_dir
                .iter()
                .try_find_map(|(ap, m, _)| {
                    ap.to_relative(Some(&title_path))
                        // .okay("can't convert path to relative to title's path")
                        .and_then(|p| {
                            std::fs::read(ap.as_ref())
                                .context("can't read file")
                                .and_then(|buf| {
                                    image::load_from_memory(&buf).context("can't decode image")
                                })
                                .and_then(|img| encode(&img).context("can't encode to blurhash"))
                                .map(|bh_result| (p.to_string_lossy().to_string(), m, bh_result))
                        })
                })
                .map(|(page_rel_path, last_modified, bh_result)| {
                    comicinfo.pages_mut().push(ComicPageInfo {
                        page_type: ComicPageType::FrontCover,
                        blurhash: Some(bh_result.blurhash.clone()),
                        image_path: Some(page_rel_path.clone()),
                        image_width: bh_result.width,
                        image_height: bh_result.height,
                        modified_date_at_encode: Some(*last_modified),
                        ..Default::default()
                    });

                    cover_path = Some(page_rel_path);
                    cover_blurhash = Some(bh_result.blurhash);
                    cover_width = Some(bh_result.width);
                    cover_height = Some(bh_result.height);
                })
                .okay(format!(
                    "no file in `{title_path_str}` can be encoded to blurhash"
                ));
        };
        // save ComicInfo.xml
        std::fs::write(
            title_path.as_ref().join(COMICINFO_FILENAME),
            comicinfo.to_pretty_string()?.as_bytes(),
        )
        .okay(format!(
            "can't write `{comicinfo_path_str}` back to storage"
        ));

        pages_in_title.extend(pages_in_dir.iter().filter_map(|(ap, m, s)| {
            ap.to_relative(Some(&title_path))
                .okay(format!("can't convert {ap} to relative to title's path"))
                .map(|p| p.to_string_lossy().to_string())
                .map(|p| PageInTitle {
                    path: p,
                    last_modified: *m,
                    size: Some(*s),
                })
        }));
    }

    let mut txn = app_state
        .pool
        .begin()
        .await
        .context("can't begin transaction")?;

    let category_id = upsert_category(&app_state, &parent_path, &category_path_to_id, &mut *txn)
        .await
        .context(format!(
            "can't upsert category `{}` to database",
            parent_path.unwrap_or_else(|| PathBuf::from("")).display()
        ))?;

    // upsert title and get its id
    let title_id = sqlx::query!(
        "INSERT INTO titles
            (id, title, category_id, author, description, release, path, is_dir,
            is_series, cover_path, cover_blurhash, cover_width, cover_height,
            date_updated)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, FALSE, $9, $10, $11, $12, $13)
        ON CONFLICT (path)
        DO UPDATE SET
            title = EXCLUDED.title,
            category_id = EXCLUDED.category_id,
            author = EXCLUDED.author,
            description = EXCLUDED.description,
            release = EXCLUDED.release,
            path = EXCLUDED.path,
            is_dir = EXCLUDED.is_dir,
            is_series = FALSE,
            cover_path = EXCLUDED.cover_path,
            cover_blurhash = EXCLUDED.cover_blurhash,
            cover_width = EXCLUDED.cover_width,
            cover_height = EXCLUDED.cover_height,
            date_updated = EXCLUDED.date_updated
        RETURNING id",
        app_state.id_generator.snowflake().await?,
        comicinfo.title,
        category_id,
        comicinfo.penciller.as_ref(),
        comicinfo.summary.as_ref(),
        comicinfo.get_release(),
        &title_path_str,
        match oneshot_type {
            OneshotType::Archive(_) => false,
            OneshotType::Directory(_) => true,
        },
        cover_path,
        cover_blurhash,
        cover_width,
        cover_height,
        title_last_modified,
    )
    .fetch_one(&mut *txn)
    .await
    .context("can't upsert title model to DB")?
    .id;

    upsert_tags(&app_state, &comicinfo, &title_id, &mut *txn).await?;

    '_upsert_pages: {
        let page_ids =
            join_all((0..pages_in_title.len()).map(|_| app_state.id_generator.snowflake()))
                .await
                .into_iter()
                .collect::<Result<Vec<i64>, _>>()
                .context("can't generate enough page ids")?;

        let mut page_paths = Vec::with_capacity(pages_in_title.len());
        let mut page_filesizes = Vec::with_capacity(pages_in_title.len());
        let mut page_descriptions = Vec::with_capacity(pages_in_title.len());

        for page in &pages_in_title {
            page_paths.push(page.path.clone());
            page_filesizes.push(page.size.unwrap_or_default());
            page_descriptions.push(
                comicinfo
                    .get_page_description(&page.path)
                    .unwrap_or_default(),
            );
        }

        sqlx::query!(
            "WITH _ AS (
            INSERT INTO oneshots_pages (id, title_id, path, filesize, description)
            SELECT id, $1, path, NULLIF(filesize, 0), NULLIF(description, '')
                FROM UNNEST($2::bigint[], $3::text[], $4::bigint[], $5::text[])
                AS t(id, path, filesize, description)
            ON CONFLICT (title_id, path) DO UPDATE
                SET description = EXCLUDED.description
            )
            DELETE FROM oneshots_pages WHERE title_id = $1
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

    txn.commit().await.context("can't commit transaction")?;

    Ok(title_id)
}
