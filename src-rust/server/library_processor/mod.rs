mod dir_entry_guesser;
mod upsert_category;
mod upsert_oneshot;
mod upsert_series;
mod upsert_tags;

use std::{
    collections::{HashMap, VecDeque},
    fs::DirEntry,
    mem::drop,
    path::PathBuf,
    string::FromUtf8Error,
    sync::Arc,
};

use futures_core::future::BoxFuture;
use futures_util::future::join_all;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::{debug, error, info};

use crate::{
    library_processor::{
        dir_entry_guesser::{DirEntryType, DirEntryTypeGuesser},
        upsert_category::UpsertCategoryErr,
        upsert_oneshot::{upsert_oneshot, OneshotType},
        upsert_series::upsert_series,
        upsert_tags::UpsertTagsErr,
    },
    structs::{
        absolute_path::{AbsolutePath, AbsolutePathErr},
        ids::{CategoryID, TitleID},
    },
    traits::do_something_and_ok::DoSomethingAndOk,
    utils::{
        app_state::AppState,
        archive_file::{ArchiveFileError, ItemInArchive},
        constants::COMICINFO,
        id_generator::GenerateIDErr,
    },
};

#[derive(Debug)]
struct ScannedEntry {
    entry: DirEntry,
    parent: Option<PathBuf>, // None for the library root
}

/// ONLY add errors that would need to handle differently, e.g. [`IsIgnored`]
/// would tell the caller the function "failed" because the file is ignored,
/// not something wrong happened.
///
/// [`IsIgnored`]: UpsertTitleError::IsIgnored
#[derive(Debug, thiserror::Error)]
enum UpsertTitleErr {
    #[error("it's empty")]
    IsEmpty,

    #[error("can't transform absolute path: {0:?}")]
    PathAbsoluteConv(#[from] AbsolutePathErr),

    #[error("can't start a database transaction: {0:?}")]
    TransactionBegin(sqlx::Error),
    #[error("can't commit a database transaction: {0:?}")]
    TransactionCommit(sqlx::Error),

    #[error("can't upsert one-shot title: {0:?}")]
    UpsertOneShot(sqlx::Error),

    #[error("can't upsert series title: {0:?}")]
    UpsertSeries(sqlx::Error),
    #[error("can't upsert series' chapters: {0:?}")]
    UpsertChapters(sqlx::Error),
    #[error("can't upsert oneshot' chapters: {0:?}")]
    UpsertChapter(sqlx::Error),

    #[error("can't extract {COMICINFO} from archive: {0:?}")]
    ComicInfoExtract(ArchiveFileError),
    #[error("can't decode {COMICINFO} from vec<u8>: {0:?}")]
    ComicInfoReadFromVecU8(FromUtf8Error),
    #[error("can't read {COMICINFO} from filesystem: {0:?}")]
    ComicInfoReadFromFs(std::io::Error),
    #[error("can't deserialize {COMICINFO}: {0:?}")]
    ComicInfoParse(quick_xml::DeError),
    #[error("can't serialize {COMICINFO}: {0:?}")]
    ComicInfoSerialize(quick_xml::errors::serialize::SeError),
    #[error("can't write {COMICINFO} to archive: {0:?}")]
    ComicInfoWriteArchive(ArchiveFileError),
    #[error("can't write {COMICINFO} to directory: {0:?}")]
    ComicInfoWriteDir(std::io::Error),

    #[error("can't generate an ID for the title: {0:?}")]
    GenTitleID(GenerateIDErr),
    #[error("can't generate enough IDs for pages: {0:?}")]
    GenPageIDs(GenerateIDErr),
    #[error("can't generate enough IDs for chapters: {0:?}")]
    GenChapterIDs(GenerateIDErr),

    #[error("can't upsert category to database: {0:?}")]
    UpsertCategory(#[from] UpsertCategoryErr),
    #[error("can't upsert tags to database: {0:?}")]
    UpsertTags(#[from] UpsertTagsErr),
    #[error("can't upsert pages to database: {0:?}")]
    UpsertPages(sqlx::Error),
}

type PageInTitle = ItemInArchive;

pub async fn full_scan(app_state: Arc<AppState>) {
    // scan library directory
    let Some(library_dir) = std::fs::read_dir(app_state.config.library_path.as_ref()).okay(|e| {
        error!(
            "can't read library path `{}`: {e:?}",
            app_state.config.library_path
        );
    }) else {
        return;
    };

    let mut queue: VecDeque<ScannedEntry> = VecDeque::new();

    // populate first layer of the library tree to queue
    for entry in library_dir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                error!("can't extract entry from root library: {e:?}",);
                continue;
            }
        };
        queue.push_front(ScannedEntry {
            entry,
            parent: None,
        });
    }

    // processing tasks waiting to be .await-ed
    let mut tasks: Vec<(
        BoxFuture<'static, anyhow::Result<TitleID, UpsertTitleErr>>,
        PathBuf,
    )> = vec![];
    let category_path_to_id: Arc<RwLock<HashMap<AbsolutePath, CategoryID>>> =
        Arc::new(RwLock::new(HashMap::new()));

    // BFS
    while let Some(scanned) = queue.pop_back() {
        let entry_path = scanned.entry.path();
        match scanned.entry.guess(
            app_state.live_config.get_nomediasupport().await,
            app_state.live_config.get_komga_oneshot_support().await,
            app_state.live_config.get_komga_recycle_support().await,
        ) {
            Ok(entry_type) => match entry_type {
                DirEntryType::CategoryDir(sub_entries) => {
                    for sub_entry in sub_entries {
                        queue.push_front(ScannedEntry {
                            entry: sub_entry,
                            parent: Some(entry_path.clone()),
                        });
                    }
                }
                DirEntryType::SeriesDir(chapters) => {
                    tasks.push((
                        Box::pin(upsert_series(
                            app_state.clone(),
                            entry_path.clone(),
                            chapters,
                            scanned.parent,
                            category_path_to_id.clone(),
                            app_state.live_config.get_nomediasupport().await,
                        )),
                        entry_path,
                    ));
                }
                DirEntryType::OneShotDir(pages) => {
                    tasks.push((
                        Box::pin(upsert_oneshot(
                            app_state.clone(),
                            entry_path.clone(),
                            OneshotType::Directory(pages),
                            scanned.parent,
                            category_path_to_id.clone(),
                            app_state.live_config.get_nomediasupport().await,
                        )),
                        entry_path,
                    ));
                }
                DirEntryType::OneShotArchiveFile(files_in_archive) => {
                    tasks.push((
                        Box::pin(upsert_oneshot(
                            app_state.clone(),
                            entry_path.clone(),
                            OneshotType::Archive(files_in_archive),
                            scanned.parent,
                            category_path_to_id.clone(),
                            app_state.live_config.get_nomediasupport().await,
                        )),
                        entry_path,
                    ));
                }
                DirEntryType::Ignored => {
                    debug!("ignored entry `{}`", entry_path.display());
                }
            },
            Err(e) => {
                error!(
                    "can't guess the directory type for `{}`: {e:?}",
                    entry_path.display()
                );
            }
        }
    }

    let sem = Arc::new(Semaphore::new(app_state.config.concurrent_scan_tasks));
    let mut futs = vec![];
    let upserted_title_ids = Arc::new(Mutex::new(vec![]));

    for (task, title_path) in tasks {
        let sem = sem.clone();
        let upserted_title_ids = upserted_title_ids.clone();
        futs.push(tokio::spawn(async move {
            let title_path = title_path.display();

            #[allow(clippy::significant_drop_in_scrutinee)]
            let permit = match sem.acquire().await {
                Ok(permit) => permit,
                Err(e) => {
                    error!("can't process `{title_path}`: can't acquire a permit from semaphore: {e:?}");
                    return;
                }
            };

            match task.await {
                Ok(upserted_title_id) => upserted_title_ids.lock().await.push(upserted_title_id),
                Err(e) => match e {
                    UpsertTitleErr::IsEmpty => info!("title `{title_path}` is empty"),
                    e => error!("can't process `{title_path}`: {e:?}"),
                },
            }

            drop(permit);
        }));
    }

    for join_err in join_all(futs).await {
        if let Err(e) = join_err {
            error!("can't join task: {e:?}");
        }
    }

    if let Err(e) = sqlx::query!(
        "WITH _ AS (
            DELETE FROM categories
            WHERE id NOT IN (
                    SELECT id
                    FROM UNNEST($1::bigint []) AS id
                )
        )
        DELETE FROM titles
        WHERE id NOT IN (
                SELECT id
                FROM UNNEST($2::bigint []) AS id
            )",
        &category_path_to_id
            .read()
            .await
            .values()
            .copied()
            .map(|id| id.as_ref())
            .collect::<Vec<i64>>(),
        &*upserted_title_ids
            .lock()
            .await
            .iter()
            .map(|id| id.as_ref())
            .collect::<Vec<i64>>(),
    )
    .execute(&app_state.pool)
    .await
    {
        error!("can't cleanup non-exist categories and titles from database: {e:?}");
    }

    info!("finished processing library");
}
