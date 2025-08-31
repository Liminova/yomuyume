use std::{fs::DirEntry, sync::Arc};

use redb::ReadableDatabase;
use tracing::warn;

use crate::{
    AppState,
    database::{
        self,
        content::{CategoryIdentityPath, TitleIdentityPath},
    },
    indexer::{
        start_index::UpsertTitleErr,
        utils::{
            find_chapter_cover::find_chapter_cover,
            index_archive_chap_pages::read_chap_pages_archive,
            index_directory_chap_pages::read_chap_pages_dir,
        },
    },
    utils::{
        absolute_path::AbsolutePath,
        archive_file::{ArchiveFile, ItemInArchive},
        comic_info::ComicInfo,
        constants::COMICINFO,
        okay::MapErrorThenOk,
    },
};

#[derive(Debug)]
pub enum OneshotType {
    Archive(Vec<ItemInArchive>),
    Directory(Vec<DirEntry>),
}

/// index a oneshot to the database and return its ID
#[allow(clippy::cognitive_complexity)]
pub async fn index_oneshot(
    app_state: Arc<AppState>,
    title_path: AbsolutePath,
    oneshot_type: OneshotType,
    parent_path: Option<AbsolutePath>,
) -> Result<(Option<CategoryIdentityPath>, TitleIdentityPath), UpsertTitleErr> {
    let title_identity_path = title_path
        .to_relative(Some(
            parent_path
                .as_ref()
                .unwrap_or(&app_state.config.library_path),
        ))?
        .to_string_lossy()
        .to_string();

    let category_identity_path = parent_path
        .as_ref()
        .map(|p| {
            p.to_relative(Some(&app_state.config.library_path))
                .map(|rp| rp.to_string_lossy().to_string())
        })
        .transpose()?;

    'skip_process_when_db_is_newer: {
        if title_path.is_dir() || app_state.first_time_index_content {
            break 'skip_process_when_db_is_newer;
        }

        let read_txn = app_state.db.content.begin_read().unwrap();
        let titles_table = read_txn.open_table(database::content::TITLES).unwrap();
        if let Some(old_last_modified) = titles_table
            .get((category_identity_path.clone(), title_identity_path.clone()))
            .unwrap()
            .map(|v| v.value().last_modified)
            .flatten()
            && let Some(new_last_modified) = title_path.last_modified().okay(|e| {
                warn!(
                    "can't get last modified of title {}: {e}",
                    title_path.display()
                )
            })
            && old_last_modified >= new_last_modified
        {
            return Ok((category_identity_path, title_identity_path));
        };
    }

    let pages_in_db = 'scoped: {
        if app_state.first_time_index_content {
            break 'scoped vec![];
        }

        let read_txn = app_state.db.user.begin_read().unwrap();
        let chapters_table = read_txn.open_table(database::content::CHAPTERS).unwrap();
        let pages_table = read_txn.open_table(database::content::PAGES).unwrap();

        let Some(page_identity_paths) = chapters_table
            .get((title_identity_path.clone(), None))
            .unwrap()
            .map(|v| v.value().pages)
        else {
            break 'scoped vec![];
        };

        page_identity_paths
            .into_iter()
            .filter_map(|page_path| {
                pages_table
                    .get((title_identity_path.clone(), None, page_path.clone()))
                    .okay(|e| warn!("can't get PageInfo: {e}"))
                    .flatten()
                    .map(|p| (page_path, p.value()))
            })
            .collect::<Vec<_>>()
    };

    let (indexed_pages, comic_info) = match oneshot_type {
        OneshotType::Archive(files_in_archive) => (
            read_chap_pages_archive(&app_state, &title_path, files_in_archive, &pages_in_db)
                .await?,
            title_path
                .as_ref()
                .read_file_from_archive(COMICINFO)
                .okay(|e| {
                    warn!(
                        "can't read ComicInfo.xml from oneshot {}: {e}",
                        title_path.display()
                    )
                })
                .and_then(|b| {
                    String::from_utf8(b).okay(|e| {
                        warn!(
                            "can't convert ComicInfo.xml from oneshot {} to UTF-8: {e}",
                            title_path.display()
                        )
                    })
                })
                .and_then(|s| {
                    ComicInfo::from_str(&s).okay(|e| {
                        warn!(
                            "can't parse ComicInfo.xml from oneshot {}: {e}",
                            title_path.display()
                        )
                    })
                }),
        ),

        OneshotType::Directory(sub_entries) => (
            read_chap_pages_dir(&app_state, &title_path, sub_entries, &pages_in_db).await?,
            title_path
                .as_ref()
                .join(COMICINFO)
                .exists()
                .then(|| {
                    std::fs::read_to_string(title_path.as_ref().join(COMICINFO))
                        .okay(|e| {
                            warn!(
                                "can't read ComicInfo.xml from oneshot {}: {e}",
                                title_path.display()
                            )
                        })
                        .and_then(|s| {
                            ComicInfo::from_str(&s).okay(|e| {
                                warn!(
                                    "can't parse ComicInfo.xml from oneshot {}: {e}",
                                    title_path.display()
                                )
                            })
                        })
                })
                .unwrap_or_default(),
        ),
    };

    let write_txn = app_state.db.content.begin_write().unwrap();

    if let Some(category_identity_path) = category_identity_path.clone() {
        let mut categories_table = write_txn.open_table(database::content::CATEGORIES).unwrap();
        if let Some(existing_category) = categories_table
            .get_mut(category_identity_path.clone())
            .unwrap()
        {
            existing_category.value().push(title_identity_path.clone());
        } else {
            categories_table.insert(category_identity_path, vec![title_identity_path.clone()]);
        }
    }

    let mut titles_table = write_txn.open_table(database::content::TITLES).unwrap();
    titles_table.insert(
        (category_identity_path.clone(), title_identity_path.clone()),
        database::content::TitleInfo {
            chapters: None,
            last_modified: title_path.last_modified().okay(|e| {
                warn!(
                    "can't get last modified of title {}: {e}",
                    title_path.display()
                )
            }),
        },
    );
    drop(titles_table);

    let mut chapters_table = write_txn.open_table(database::content::CHAPTERS).unwrap();
    chapters_table.insert(
        (title_identity_path.clone(), None),
        database::content::ChapterInfo {
            pages: indexed_pages
                .upsert
                .iter()
                .map(|p| p.identity_path.clone())
                .collect(),
            cover: find_chapter_cover(
                &indexed_pages,
                comic_info.as_ref().map(|ci| ci.pages().as_slice()),
                &pages_in_db,
            ),
            fallback_vol_num: None,
            last_modified: None,
        },
    );
    drop(chapters_table);

    let mut pages_table = write_txn.open_table(database::content::PAGES).unwrap();
    for page in indexed_pages.upsert.into_iter() {
        pages_table.insert(
            (title_identity_path.clone(), None, page.identity_path),
            database::content::PageInfo {
                width: page.width,
                height: page.height,
                color: page.color,
                size: page.size,
                last_modified: page.last_modified,
                parent_path: title_path.to_relative(Some(&app_state.config.library_path))?,
            },
        );
    }
    for page_to_delete in indexed_pages.delete {
        pages_table.remove((title_identity_path.clone(), None, page_to_delete));
    }
    drop(pages_table);

    write_txn.commit().unwrap();

    Ok((category_identity_path, title_identity_path))
}
