mod handle_archive_chapter;
mod handle_directory_chapter;
mod try_everything_as_cover;
mod try_get_configured_cover;

use std::{collections::HashMap, fs::read_to_string, path::PathBuf, sync::Arc};

use chrono::{DateTime, Utc};
use futures_util::future::join_all;
use handle_archive_chapter::handle_archive_chapter;
use handle_directory_chapter::handle_directory_chapter;
use tokio::sync::RwLock;
use tracing::warn;
use try_everything_as_cover::try_everything_as_cover;
use try_get_configured_cover::try_get_configured_cover;

use crate::{
    app_state::AppState,
    library_processor::{
        dir_entry_guesser::{ScannedChapterInfo, ScannedChapterType},
        upsert_category::upsert_category,
        upsert_tags::upsert_tags,
        UpsertTitleErr,
    },
    traits::{do_something_and_ok::DoSomethingAndOk, pathbuf_utils::PathBufUtils},
    types::{
        absolute_path::{AbsolutePath, ToAbsolute},
        comic_info::{ComicInfo, ComicPageType},
        TitleID,
    },
    utils::{config::COMICINFO_FILENAME, macros::bail_if_empty},
};

/// differ from [`PageInTitle`] in that it also contains the page description
#[derive(Debug, Clone)]
struct PageInArchive {
    pub path: String,
    pub size: Option<i64>,
    pub last_modified: Option<DateTime<Utc>>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
struct PageInDirectory {
    pub path: AbsolutePath,
    pub size: Option<i64>,
    pub last_modified: Option<DateTime<Utc>>,
    pub description: Option<String>,
}

#[derive(Debug)]
enum ChapterType {
    Archive(Vec<PageInArchive>),
    Directory(Vec<PageInDirectory>),
}

#[derive(Debug)]
struct ChapterInfo {
    pub path: AbsolutePath,
    pub volume: i32,
    pub description: Option<String>,
    pub chapter_type: ChapterType,
    pub modified: Option<DateTime<Utc>>,
}

/// upsert a series to the database and return its ID
pub async fn upsert_series(
    app_state: Arc<AppState>,
    title_path: PathBuf,
    chapters: Vec<ScannedChapterInfo>,
    parent_path: Option<PathBuf>,
    category_path_to_id: Arc<RwLock<HashMap<AbsolutePath, i64>>>,
    nomedia_support: bool,
) -> Result<TitleID, UpsertTitleErr> {
    let title_path = title_path.to_absolute(None)?;

    let comicinfo_path = title_path.as_ref().join(COMICINFO_FILENAME);
    let (original_comicinfo, mut comicinfo) = {
        if comicinfo_path.exists() {
            let s = read_to_string(&comicinfo_path).map_err(UpsertTitleErr::ComicInfoReadFromFs)?;
            let tmp = ComicInfo::from_str(&s).map_err(UpsertTitleErr::ComicInfoParse)?;
            (tmp.clone(), tmp)
        } else {
            (ComicInfo::default(), ComicInfo::default())
        }
    };

    let handled_chapters = chapters
        .into_iter()
        .filter_map(|c| {
            c.path
                .to_absolute(None)
                .okay(|e| warn!("can't convert chapter path to absolute: {e:?}"))
                .map(|p| (p, c))
        })
        .filter_map(|(chapter_path, scanned_chapter_info)| {
            let backup_volume_number = scanned_chapter_info.volume;
            match scanned_chapter_info.dir_or_archive {
                ScannedChapterType::Directory => {
                    handle_directory_chapter(chapter_path, backup_volume_number, nomedia_support)
                        .okay(|e| warn!("can't handle directory-chapter: {e:?}"))
                }
                ScannedChapterType::Archive(items_in_archive) => handle_archive_chapter(
                    chapter_path,
                    backup_volume_number,
                    items_in_archive,
                    nomedia_support,
                )
                .okay(|e| warn!("can't handle archive-chapter: {e:?}")),
            }
        })
        .collect::<Vec<_>>();
    bail_if_empty!(handled_chapters, Err(UpsertTitleErr::IsEmpty));

    if comicinfo != original_comicinfo {
        comicinfo_path
            .create_file_if_not_exists()
            .map_err(UpsertTitleErr::ComicInfoWriteDir)?;
        let s = comicinfo
            .to_pretty_string()
            .map_err(UpsertTitleErr::ComicInfoSerialize)?;
        std::fs::write(&comicinfo_path, s.as_bytes()).map_err(UpsertTitleErr::ComicInfoWriteDir)?;
    }

    let mut cover_path: Option<String> = None;
    let mut cover_blurhash: Option<String> = None;
    let mut cover_width: Option<i32> = None;
    let mut cover_height: Option<i32> = None;

    'cover_finder: {
        // try to use configured cover
        if comicinfo
            .pages_mut()
            .iter_mut()
            .rev()
            .filter(|page_cfg| page_cfg.page_type == ComicPageType::FrontCover)
            .filter_map(|page_cfg| {
                page_cfg
                    .image_path
                    .clone()
                    .map(|path| (page_cfg, path.replace('\\', "/")))
            })
            .find_map(|(page_cfg, page_cfg_path)| {
                try_get_configured_cover(
                    page_cfg,
                    &page_cfg_path,
                    &title_path,
                    &handled_chapters,
                    &mut cover_path,
                    &mut cover_blurhash,
                    &mut cover_width,
                    &mut cover_height,
                )
            })
            .is_some()
        {
            break 'cover_finder;
        };

        if try_everything_as_cover(
            &mut comicinfo,
            &handled_chapters,
            &mut cover_path,
            &mut cover_blurhash,
            &mut cover_width,
            &mut cover_height,
        )
        .is_none()
        {
            warn!("no page in `{title_path}` can be used as cover");
        };
    };

    let mut txn = app_state
        .pool
        .begin()
        .await
        .map_err(UpsertTitleErr::TransactionBegin)?;

    let category_id = upsert_category(
        &app_state,
        parent_path.as_ref(),
        &category_path_to_id,
        &mut *txn,
    )
    .await?;

    // upsert title and get its id
    let title_id = sqlx::query!(
        "INSERT INTO titles
            (id, title, category_id, author, description, release, path, is_dir,
            is_series, cover_path, cover_blurhash, cover_width, cover_height,
            date_updated)
        VALUES ($1, $2, $3, $4, $5, $6, $7, TRUE, TRUE, $8, $9, $10, $11, $12)
        ON CONFLICT (path)
        DO UPDATE SET
            title = EXCLUDED.title,
            category_id = EXCLUDED.category_id,
            author = EXCLUDED.author,
            description = EXCLUDED.description,
            release = EXCLUDED.release,
            path = EXCLUDED.path,
            is_dir = TRUE,
            is_series = TRUE,
            cover_path = EXCLUDED.cover_path,
            cover_blurhash = EXCLUDED.cover_blurhash,
            cover_width = EXCLUDED.cover_width,
            cover_height = EXCLUDED.cover_height,
            date_updated = EXCLUDED.date_updated
        RETURNING id",
        app_state
            .id_generator
            .snowflake()
            .await
            .map_err(UpsertTitleErr::GenTitleID)?,
        comicinfo.title,
        category_id,
        comicinfo.penciller.as_ref(),
        comicinfo.summary.as_ref(),
        comicinfo.get_release(),
        title_path
            .to_relative(Some(&app_state.config.library_path))?
            .to_string_lossy()
            .to_string(),
        cover_path,
        cover_blurhash,
        cover_width,
        cover_height,
        handled_chapters.iter().map(|c| c.modified).flatten().max(),
    )
    .fetch_one(&mut *txn)
    .await
    .map_err(UpsertTitleErr::UpsertSeries)?
    .id;

    tracing::debug!("upserted series: `{}`", title_path.as_ref().display());

    upsert_tags(&app_state, &comicinfo, &title_id, &mut *txn).await?;

    let chapter_path_to_id = '_upsert_chapters: {
        let chapter_ids =
            join_all((0..handled_chapters.len()).map(|_| app_state.id_generator.snowflake()))
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .map_err(UpsertTitleErr::GenChapterIDs)?;

        let mut chapter_paths = Vec::with_capacity(handled_chapters.len());
        let mut chapter_numbers = Vec::with_capacity(handled_chapters.len());
        let mut chapter_descriptions = Vec::with_capacity(handled_chapters.len());
        let mut chapter_is_dirs = Vec::with_capacity(handled_chapters.len());

        'next_chapter: for chapter in &handled_chapters {
            let chapter_path_string = match chapter.path.to_relative(Some(&title_path)) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(e) => {
                    warn!(
                        "can't convert `{}` to relative to upsert to database: {e:?}",
                        chapter.path
                    );
                    continue 'next_chapter;
                }
            };

            chapter_paths.push(chapter_path_string);
            chapter_numbers.push(chapter.volume);
            chapter_descriptions.push(chapter.description.clone().unwrap_or_default());
            chapter_is_dirs.push(match chapter.chapter_type {
                ChapterType::Archive(_) => false,
                ChapterType::Directory(_) => true,
            });
        }

        let upserted_chapters = sqlx::query!(
            "INSERT INTO chapters (id, title_id, path, number, description, is_dir)
            SELECT id, $1, path, number, NULLIF(description, ''), is_dir
                FROM UNNEST($2::bigint[], $3::text[], $4::integer[], $5::text[], $6::boolean[])
                AS t(id, path, number, description, is_dir)
            ON CONFLICT (title_id, path) DO UPDATE SET
                number = EXCLUDED.number,
                description = EXCLUDED.description,
                is_dir = EXCLUDED.is_dir
            RETURNING id, path",
            title_id,
            &chapter_ids,
            &chapter_paths,
            &chapter_numbers,
            &chapter_descriptions,
            &chapter_is_dirs,
        )
        .fetch_all(&mut *txn)
        .await
        .map_err(UpsertTitleErr::UpsertChapters)?;

        upserted_chapters
            .into_iter()
            .map(|chapter| (chapter.path, chapter.id))
            .collect::<HashMap<_, _>>()
    };

    '_upsert_chapters_pages: {
        let total_page_count: usize = handled_chapters
            .iter()
            .map(|chapter| match chapter.chapter_type {
                ChapterType::Archive(ref pages) => pages.len(),
                ChapterType::Directory(ref pages) => pages.len(),
            })
            .sum();

        let mut page_ids_pool =
            join_all((0..total_page_count).map(|_| app_state.id_generator.snowflake()))
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .map_err(UpsertTitleErr::GenPageIDs)?;

        let mut chapter_ids = Vec::with_capacity(handled_chapters.len());
        let mut page_ids = Vec::with_capacity(total_page_count);
        let mut page_paths = Vec::with_capacity(total_page_count);
        let mut page_filesizes = Vec::with_capacity(total_page_count);
        let mut page_descriptions = Vec::with_capacity(total_page_count);

        'next_chapter: for chapter in &handled_chapters {
            let Some(chapter_path_string) = chapter
                .path
                .to_relative(Some(&title_path))
                .map(|p| p.to_string_lossy().to_string())
                .okay(|e| warn!("can't strip title path from chapter path: {e:?}"))
            else {
                continue 'next_chapter;
            };

            let Some(chapter_id) = chapter_path_to_id.get(&chapter_path_string) else {
                warn!("can't get chapter ID from hashmap");
                continue 'next_chapter;
            };

            match &chapter.chapter_type {
                ChapterType::Archive(pages) => {
                    for page in pages {
                        chapter_ids.push(*chapter_id);
                        page_ids.push(if let Some(id) = page_ids_pool.pop() {
                            id
                        } else {
                            warn!("can't get page ID from pool, this should never happen");
                            continue 'next_chapter;
                        });
                        page_paths.push(page.path.clone());
                        page_filesizes.push(page.size.unwrap_or_default());
                        page_descriptions.push(page.description.clone().unwrap_or_default());
                    }
                }
                ChapterType::Directory(pages) => {
                    for page in pages {
                        let Some(page_path_str) = page
                            .path
                            .to_relative(Some(&chapter.path))
                            .map(|p| p.to_string_lossy().to_string())
                            .okay(|e| warn!("can't strip title path from page path to upsert to database: {e:?}"))
                        else {
                            continue 'next_chapter;
                        };

                        chapter_ids.push(*chapter_id);
                        page_ids.push(if let Some(id) = page_ids_pool.pop() {
                            id
                        } else {
                            warn!("can't get page ID from pool, this should never happen");
                            continue 'next_chapter;
                        });
                        page_descriptions.push(page.description.clone().unwrap_or_default());
                        page_paths.push(page_path_str);
                        page_filesizes.push(page.size.unwrap_or_default());
                    }
                }
            }
        }

        sqlx::query!(
            "INSERT INTO chapters_pages (id, chapter_id, path, filesize, description)
            SELECT id, chapter_id, path, NULLIF(filesize, 0), NULLIF(description, '')
                FROM UNNEST($1::bigint[], $2::bigint[], $3::text[], $4::bigint[], $5::text[])
                AS t(id, chapter_id, path, filesize, description)
            ON CONFLICT (chapter_id, path) DO UPDATE
            SET description = EXCLUDED.description",
            &page_ids,
            &chapter_ids,
            &page_paths,
            &page_filesizes,
            &page_descriptions
        )
        .execute(&mut *txn)
        .await
        .map_err(UpsertTitleErr::UpsertPages)?;
    }

    tracing::debug!(
        "upserted chapter pages: `{}`",
        title_path.as_ref().display()
    );

    txn.commit()
        .await
        .map_err(UpsertTitleErr::TransactionCommit)?;

    Ok(title_id)
}
