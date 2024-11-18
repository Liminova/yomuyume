use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use chrono::{DateTime, Timelike, Utc};
use futures_util::future::join_all;
use tokio::sync::RwLock;
use tracing::warn;

use crate::{
    app_state::AppState,
    archive_file::{ArchiveFile, ItemsInArchiveUtils},
    config::COMICINFO_FILENAME,
    library_processor::{
        blurhash::encode,
        dir_entry_guesser::{ChapterDirOrArchive, ChapterInfo},
        upsert_category::upsert_category,
        upsert_tags::upsert_tags,
        UpsertTitleError,
    },
    traits::{PathBufUtils, StringUtils},
    types::{
        absolute_path::AbsolutePath,
        comic_info::{ComicInfo, ComicPageInfo, ComicPageType},
        TitleID,
    },
};

/// differ from [`PageInTitle`] in that it also contains the page description
#[derive(Debug, Clone)]
struct ArchiveChapterPage {
    pub path: String,
    pub size: Option<i64>,
    pub last_modified: DateTime<Utc>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
struct DirChapterPage {
    pub path: AbsolutePath,
    pub size: Option<i64>,
    pub last_modified: DateTime<Utc>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
enum ChapterType {
    Archive(Vec<ArchiveChapterPage>),
    Dir(Vec<DirChapterPage>),
}

#[derive(Debug, Clone)]
struct ProcessedChapter {
    pub path: AbsolutePath,
    pub number: i32,
    pub description: Option<String>,
    pub chapter_type: ChapterType,
}

/// upsert a series to the database and return its ID
pub async fn upsert_series(
    app_state: Arc<AppState>,
    title_path: PathBuf,
    chapters: Vec<ChapterInfo>,
    parent_path: Option<PathBuf>,
    category_path_to_id: Arc<RwLock<HashMap<AbsolutePath, i64>>>,
    nomedia_support: bool,
) -> Result<TitleID, UpsertTitleError> {
    let title_path = AbsolutePath::from(&title_path, None).context(format!(
        "can't convert `{}` to absolute",
        title_path.display()
    ))?;

    let mut title_last_modified: Option<DateTime<Utc>> = None;
    let mut processed_chapters: Vec<ProcessedChapter> = vec![];

    'next_chapter: for chapter_entry in chapters {
        let chapter_path = match AbsolutePath::from(&chapter_entry.path, None) {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "can't convert `{}` to absolute: {e:?}",
                    chapter_entry.path.display()
                );
                continue 'next_chapter;
            }
        };
        let chapter_path_string = chapter_path.to_string_lossy();

        // === Handle chapter as archive === //

        if let ChapterDirOrArchive::Archive(ref items_in_archive) = chapter_entry.dir_or_archive {
            _ = chapter_path
                .as_ref()
                .metadata()
                .context("can't get metadata")
                .and_then(|m| m.modified().context("can't extract modified date"))
                .map(DateTime::<Utc>::from)
                .map(|d| d.with_nanosecond(0).unwrap_or_default())
                .map_err(|e| {
                    warn!("can't get modified date of `{chapter_path_string}`: {e:?}");
                })
                .ok()
                .map(|chapter_file_last_modified| match title_last_modified {
                    Some(ref mut title_last_modified) => {
                        if chapter_file_last_modified > *title_last_modified {
                            *title_last_modified = chapter_file_last_modified;
                        }
                    }
                    None => title_last_modified = Some(chapter_file_last_modified),
                });

            let pages_in_chapter = items_in_archive
                .filter_non_image_and_nomedia_subdirs(nomedia_support)
                // TODO: do something about this
                .cloned()
                .collect::<Vec<_>>();

            if pages_in_chapter.is_empty() {
                return Err(UpsertTitleError::IsEmpty);
            }

            let chapter_comicinfo = match title_path
                .as_ref()
                .read_file_from_archive(COMICINFO_FILENAME)
                .context("can't extract ComicInfo.xml")
                .and_then(|buf| String::from_utf8(buf).context("can't decode to string from utf-8"))
                .and_then(|str| ComicInfo::from_str(&str))
            {
                Ok(comicinfo) => comicinfo,
                Err(e) => {
                    warn!("can't parse `{COMICINFO_FILENAME}` in archive-chapter `{chapter_path_string}`: {e:?}",);
                    continue 'next_chapter;
                }
            };

            processed_chapters.push(ProcessedChapter {
                path: chapter_path,
                number: match chapter_comicinfo.volume != -1 {
                    true => chapter_comicinfo.volume,
                    false => chapter_entry.number,
                },
                chapter_type: ChapterType::Archive(
                    pages_in_chapter
                        .into_iter()
                        .map(|p| ArchiveChapterPage {
                            description: chapter_comicinfo.get_page_description(&p.path),
                            path: p.path,
                            size: p.size,
                            last_modified: p.last_modified,
                        })
                        .collect(),
                ),
                description: chapter_comicinfo.summary,
            });
        } else {
            // === Handle chapter as directory === //

            type PageLastModified = DateTime<Utc>;
            type PageFileSize = i64;

            let reader = match chapter_path.as_ref().read_dir() {
                Ok(reader) => reader,
                Err(e) => {
                    warn!("can't read directory-chapter `{chapter_path_string}`: {e:?}");
                    continue 'next_chapter;
                }
            };

            let mut pages_in_chapter: Vec<(AbsolutePath, PageLastModified, PageFileSize)> = vec![];
            'next_item: for item in reader {
                let entry = match item {
                    Ok(item) => item,
                    Err(e) => {
                        warn!("can't extract entry from directory-chapter `{chapter_path_string}`: {e:?}",);
                        continue 'next_item;
                    }
                };

                let entry_path = match AbsolutePath::from(&entry.path(), None) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!(
                            "can't convert `{}` to absolute: {e:?}",
                            entry.path().display()
                        );
                        continue 'next_item;
                    }
                };
                let entry_path_string = entry_path.to_string_lossy().to_string();

                let metadata = match entry.metadata() {
                    Ok(metadata) => metadata,
                    Err(e) => {
                        warn!("can't get metadata of `{entry_path_string}`: {e:?}",);
                        continue 'next_item;
                    }
                };

                if !entry_path.as_ref().has_image_extension() {
                    continue 'next_item;
                }

                if metadata.is_dir() {
                    let sub_pages = match entry_path
                        .as_ref()
                        .scan_dir_recursively_for_image(nomedia_support)
                    {
                        Ok(pages) => pages,
                        Err(e) => {
                            warn!("can't scan images in `{entry_path_string}`: {e:?}",);
                            continue 'next_item;
                        }
                    };

                    'next_page: for page in sub_pages {
                        let page_path = match AbsolutePath::from(&page, None) {
                            Ok(p) => p,
                            Err(e) => {
                                warn!("can't convert `{}` to absolute: {e:?}", page.display());
                                continue 'next_page;
                            }
                        };
                        let page_string = page_path.to_string_lossy();

                        let metadata = match page_path.as_ref().metadata() {
                            Ok(m) => m,
                            Err(e) => {
                                warn!("can't get metadata of `{page_string}`: {e:?}");
                                continue 'next_page;
                            }
                        };

                        let modified = match metadata
                            .modified()
                            .map(DateTime::<Utc>::from)
                            .map(|d| d.with_nanosecond(0).unwrap_or_default())
                        {
                            Ok(m) => m,
                            Err(e) => {
                                warn!("can't get modified date of `{page_string}`: {e:?}");
                                continue 'next_page;
                            }
                        };

                        match title_last_modified {
                            None => title_last_modified = Some(modified),
                            Some(ref mut title_last_modified) => {
                                if modified > *title_last_modified {
                                    *title_last_modified = modified;
                                }
                            }
                        };

                        pages_in_chapter.push((page_path, modified, metadata.len() as i64));
                    }
                }

                let last_modified = match metadata
                    .modified()
                    .map(DateTime::<Utc>::from)
                    .map(|d| d.with_nanosecond(0).unwrap_or_default())
                {
                    Ok(last_modified) => last_modified,
                    Err(e) => {
                        warn!("can't extract modified date of `{entry_path_string}`: {e:?}",);
                        continue 'next_item;
                    }
                };

                match title_last_modified {
                    Some(ref mut title_last_modified) => {
                        if last_modified > *title_last_modified {
                            *title_last_modified = last_modified;
                        }
                    }
                    None => title_last_modified = Some(last_modified),
                }

                pages_in_chapter.push((entry_path, last_modified, metadata.len() as i64));
            }

            if pages_in_chapter.is_empty() {
                continue 'next_chapter;
            }

            let chapter_comicinfo = match chapter_path.as_ref().join(COMICINFO_FILENAME).exists() {
                true => {
                    match std::fs::read_to_string(chapter_path.as_ref().join(COMICINFO_FILENAME)) {
                        Ok(s) => match ComicInfo::from_str(&s) {
                            Ok(comicinfo) => comicinfo,
                            Err(e) => {
                                warn!("can't parse `{COMICINFO_FILENAME}` in directory-chapter `{chapter_path_string}`: {e:?}",);
                                continue 'next_chapter;
                            }
                        },
                        Err(e) => {
                            warn!("can't read `{COMICINFO_FILENAME}` in directory-chapter `{chapter_path_string}`: {e:?}");
                            continue 'next_chapter;
                        }
                    }
                }
                false => ComicInfo::default(),
            };

            processed_chapters.push(ProcessedChapter {
                path: chapter_path.clone(),
                number: match chapter_comicinfo.volume != -1 {
                    true => chapter_comicinfo.volume,
                    false => chapter_entry.number,
                },
                chapter_type: ChapterType::Dir(
                    pages_in_chapter
                        .into_iter()
                        .map(|p| {
                            let page_path_string = match p
                                .0
                                .to_relative(Some(chapter_path.as_ref()))
                                .map(|p| p.to_string_lossy().to_string())
                            {
                                Ok(p) => p,
                                Err(e) => {
                                    warn!(
                                        "can't convert `{}` to relative, can't get page description: {e:?}",
                                        p.0.as_ref().display(),
                                    );
                                    p.0.to_string_lossy().to_string()
                                }
                            };

                            DirChapterPage {
                                description: chapter_comicinfo
                                    .get_page_description(&page_path_string),
                                path: p.0,
                                size: Some(p.2),
                                last_modified: p.1,
                            }
                        })
                        .collect(),
                ),
                description: chapter_comicinfo.summary,
            });
        }
    }

    if processed_chapters.is_empty() {
        return Err(UpsertTitleError::IsEmpty);
    }

    let comicinfo_path = title_path.as_ref().join(COMICINFO_FILENAME);
    let mut comicinfo = ComicInfo::from_str(
        &std::fs::read_to_string(&comicinfo_path)
            .context(format!("can't read `{}`", comicinfo_path.display()))?,
    )?;

    let mut cover_path: Option<String> = None;
    let mut cover_blurhash: Option<String> = None;
    let mut cover_width: Option<i32> = None;
    let mut cover_height: Option<i32> = None;

    'cover_finder: {
        'find_configured_one: for page_info in comicinfo.pages_mut().iter_mut().rev() {
            // continue if the page's not cover || don't have a path
            if page_info.page_type != ComicPageType::FrontCover {
                continue;
            }
            let configured_cover_path = match page_info.image_path {
                Some(ref path) => path.replace('\\', "/"),
                None => continue,
            };

            // continue if the path points to a file in the same directory
            // as CominInfo.xml but not exist || not an image
            if !configured_cover_path.contains("/") {
                if !PathBuf::from(&configured_cover_path).exists() {
                    continue;
                }
                if !configured_cover_path.has_image_extension() {
                    continue;
                }
            }

            // None case shouldn't reachable, but just in case
            let components = match configured_cover_path.split_once('/') {
                Some(components) => components,
                None => continue,
            };

            // if configured cover reference to an archive-chapter
            if components.0.to_string().has_archive_extension() {
                let matched_chapter = 'scoped: {
                    for chapter in &processed_chapters {
                        if chapter.path.as_ref() == &title_path.as_ref().join(components.0) {
                            break 'scoped chapter;
                        }
                    }
                    warn!("configured cover `{configured_cover_path}` in `{}` doesn't point to a valid chapter archive`", comicinfo_path.display());
                    continue 'find_configured_one;
                };

                // find a matching valid page
                let matched_page = 'scoped: {
                    if let ChapterType::Archive(ref pages) = matched_chapter.chapter_type {
                        for page in pages {
                            if page.path == components.1 {
                                break 'scoped page;
                            }
                        }
                    }
                    warn!("configured cover `{configured_cover_path}` in `{}` doesn't exist in the chapter", comicinfo_path.display());
                    continue 'find_configured_one;
                };

                // use the configured page if lgtm
                if let (Some(blurhash), Some(modified_date_at_encode)) = (
                    page_info.blurhash.as_ref(),
                    page_info.modified_date_at_encode.as_ref(),
                ) {
                    let valid_dimension = page_info.image_width != 0 && page_info.image_height != 0;
                    let unmodified = modified_date_at_encode == &matched_page.last_modified;

                    if valid_dimension && unmodified {
                        cover_path = Some(configured_cover_path);
                        cover_blurhash = Some(blurhash.clone());
                        cover_width = Some(page_info.image_width);
                        cover_height = Some(page_info.image_height);

                        break 'cover_finder;
                    }
                }

                // try to re-encode the configured page
                match matched_chapter
                    .path
                    .as_ref()
                    .read_file_from_archive(components.1)
                    .context("can't read file")
                    .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                    .and_then(|img| encode(&img).context("can't encode image to blurhash"))
                {
                    Ok(bh_result) => {
                        cover_path = Some(configured_cover_path.clone());
                        cover_blurhash = Some(bh_result.blurhash.clone());
                        cover_width = Some(page_info.image_width);
                        cover_height = Some(page_info.image_height);

                        page_info.blurhash = Some(bh_result.blurhash.clone());
                        page_info.image_path = Some(configured_cover_path);
                        page_info.image_width = bh_result.width;
                        page_info.image_height = bh_result.height;
                        page_info.modified_date_at_encode = Some(matched_page.last_modified);

                        break 'cover_finder;
                    }
                    Err(e) => {
                        warn!(
                            "can't encode configured cover path of `{}` to blurhash: {e:?}",
                            matched_chapter.path.as_ref().display()
                        );
                    }
                };

                continue;
            }

            // else configured cover reference to a directory-chapter

            let matched_chapter = 'scoped: {
                for chapter in &processed_chapters {
                    if chapter.path.as_ref() == &title_path.as_ref().join(components.0) {
                        break 'scoped chapter;
                    }
                }
                warn!("configured cover `{configured_cover_path}` in `{}` doesn't point to a valid chapter directory", comicinfo_path.display());
                continue 'find_configured_one;
            };

            let matched_page = 'scoped: {
                if let ChapterType::Dir(ref pages) = matched_chapter.chapter_type {
                    for page in pages {
                        let configured_cover_path = match AbsolutePath::from(
                            &PathBuf::from(components.1),
                            Some(matched_chapter.path.as_ref()),
                        ) {
                            Ok(p) => p,
                            Err(e) => {
                                warn!("can't convert `{configured_cover_path}` to absolute: {e:?}");
                                continue;
                            }
                        };
                        if configured_cover_path == page.path {
                            break 'scoped page;
                        }
                    }
                }
                warn!("configured cover `{configured_cover_path}` in `{}` doesn't point to a valid chapter directory", comicinfo_path.display());
                continue 'find_configured_one;
            };

            // use the configured page if lgtm
            if let (Some(blurhash), Some(modified_date_at_encode)) = (
                page_info.blurhash.as_ref(),
                page_info.modified_date_at_encode.as_ref(),
            ) {
                let valid_dimension = page_info.image_width != 0 && page_info.image_height != 0;
                let unmodified = modified_date_at_encode == &matched_page.last_modified;

                if valid_dimension && unmodified {
                    cover_path = Some(configured_cover_path.clone());
                    cover_blurhash = Some(blurhash.clone());
                    cover_width = Some(page_info.image_width);
                    cover_height = Some(page_info.image_height);

                    break 'cover_finder;
                }
            }

            // try to re-encode the configured page
            match std::fs::read(matched_page.path.as_ref())
                .context("can't read file")
                .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                .and_then(|img| encode(&img).context("can't encode image to blurhash"))
            {
                Ok(bh_result) => {
                    cover_path = Some(configured_cover_path.clone());
                    cover_blurhash = Some(bh_result.blurhash.clone());
                    cover_width = Some(page_info.image_width);
                    cover_height = Some(page_info.image_height);

                    page_info.blurhash = Some(bh_result.blurhash.clone());
                    page_info.image_path = Some(configured_cover_path);
                    page_info.image_width = bh_result.width;
                    page_info.image_height = bh_result.height;
                    page_info.modified_date_at_encode = Some(matched_page.last_modified);

                    break 'cover_finder;
                }
                Err(e) => {
                    warn!(
                        "can't encode configured cover path of `{}` to blurhash: {e:?}",
                        matched_chapter.path.as_ref().display()
                    );
                }
            };
        }

        'exhaustive: for chapter in &processed_chapters {
            match chapter.chapter_type {
                ChapterType::Archive(ref pages) => {
                    for page in pages {
                        match chapter
                            .path
                            .as_ref()
                            .read_file_from_archive(&page.path)
                            .context("can't read file")
                            .and_then(|buf| {
                                image::load_from_memory(&buf).context("can't decode image")
                            })
                            .and_then(|img| encode(&img).context("can't encode image to blurhash"))
                        {
                            Ok(bh_result) => {
                                cover_path = Some(page.path.clone());
                                cover_blurhash = Some(bh_result.blurhash.clone());
                                cover_width = Some(bh_result.width);
                                cover_height = Some(bh_result.height);

                                comicinfo.pages_mut().push(ComicPageInfo {
                                    image_path: Some(page.path.clone()),
                                    image_width: bh_result.width,
                                    image_height: bh_result.height,
                                    blurhash: Some(bh_result.blurhash.clone()),
                                    modified_date_at_encode: Some(page.last_modified),
                                    ..Default::default()
                                });

                                break 'exhaustive;
                            }
                            Err(e) => {
                                warn!(
                                    "page `{}` in `{}` might not be an image: {e:?}",
                                    page.path,
                                    chapter.path.as_ref().display()
                                );
                            }
                        }
                    }
                }
                ChapterType::Dir(ref pages) => {
                    for page in pages {
                        match std::fs::read(page.path.as_ref())
                            .context("can't read file")
                            .and_then(|buf| {
                                image::load_from_memory(&buf).context("can't decode image")
                            })
                            .and_then(|img| encode(&img).context("can't encode image to blurhash"))
                        {
                            Ok(bh_result) => {
                                cover_path = match page
                                    .path
                                    .to_relative(Some(chapter.path.as_ref()))
                                    .map(|p| p.to_string_lossy().to_string())
                                {
                                    Ok(p) => Some(p),
                                    Err(e) => {
                                        warn!(
                                            "can't convert `{}` to relative: {e:?}",
                                            page.path.as_ref().display()
                                        );
                                        continue;
                                    }
                                };
                                cover_blurhash = Some(bh_result.blurhash.clone());
                                cover_width = Some(bh_result.width);
                                cover_height = Some(bh_result.height);

                                comicinfo.pages_mut().push(ComicPageInfo {
                                    image_path: cover_path.clone(),
                                    image_width: bh_result.width,
                                    image_height: bh_result.height,
                                    blurhash: Some(bh_result.blurhash),
                                    modified_date_at_encode: Some(page.last_modified),
                                    ..Default::default()
                                });

                                break 'exhaustive;
                            }
                            Err(e) => {
                                warn!(
                                    "page `{}` might not be an image: {e:?}",
                                    page.path.as_ref().display()
                                );
                            }
                        }
                    }
                }
            }
        }
    };

    tracing::debug!("cover_path: {:?}", cover_path);
    tracing::debug!("cover_blurhash: {:?}", cover_blurhash);
    tracing::debug!("cover_width: {:?}", cover_width);
    tracing::debug!("cover_height: {:?}", cover_height);

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
        VALUES ($1, $2, $3, $4, $5, $6, $7, TRUE, TRUE, $8, $9, $10, $11, $12)
        ON CONFLICT (path)
        DO UPDATE SET
            title = EXCLUDED.title,
            category_id = EXCLUDED.category_id,
            author = EXCLUDED.author,
            description = EXCLUDED.description,
            release = EXCLUDED.release,
            path = EXCLUDED.path,
            is_dir = FALSE,
            is_series = TRUE,
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
        title_path
            .to_relative(Some(&app_state.config.library_path))?
            .to_string_lossy()
            .to_string(),
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

    let chapter_path_to_id = '_upsert_chapters: {
        let chapter_ids =
            join_all((0..processed_chapters.len()).map(|_| app_state.id_generator.snowflake()))
                .await
                .into_iter()
                .collect::<Result<Vec<_>>>()
                .context("can't generate enough chapter ids")?;

        let mut chapter_paths = Vec::with_capacity(processed_chapters.len());
        let mut chapter_numbers = Vec::with_capacity(processed_chapters.len());
        let mut chapter_descriptions = Vec::with_capacity(processed_chapters.len());
        let mut chapter_is_dirs = Vec::with_capacity(processed_chapters.len());

        'next_chapter: for chapter in &processed_chapters {
            let chapter_path_string = match chapter.path.to_relative(Some(title_path.as_ref())) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(e) => {
                    warn!(
                        "can't convert `{}` to relative to upsert to database: {e:?}",
                        chapter.path.as_ref().display()
                    );
                    continue 'next_chapter;
                }
            };

            chapter_paths.push(chapter_path_string);
            chapter_numbers.push(chapter.number);
            chapter_descriptions.push(chapter.description.clone().unwrap_or_default());
            chapter_is_dirs.push(match chapter.chapter_type {
                ChapterType::Archive(_) => false,
                ChapterType::Dir(_) => true,
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
        .context("can't upsert new chapters")?;

        upserted_chapters
            .into_iter()
            .map(|chapter| (chapter.path, chapter.id))
            .collect::<HashMap<_, _>>()
    };

    '_upsert_chapters_pages: {
        let total_page_count: usize = processed_chapters
            .iter()
            .map(|chapter| match chapter.chapter_type {
                ChapterType::Archive(ref pages) => pages.len(),
                ChapterType::Dir(ref pages) => pages.len(),
            })
            .sum();

        let mut page_ids_pool =
            join_all((0..total_page_count).map(|_| app_state.id_generator.snowflake()))
                .await
                .into_iter()
                .collect::<Result<Vec<_>>>()
                .context("can't generate enough page ids")?;

        let mut chapter_ids = Vec::with_capacity(processed_chapters.len());
        let mut page_ids = Vec::with_capacity(total_page_count);
        let mut page_paths = Vec::with_capacity(total_page_count);
        let mut page_filesizes = Vec::with_capacity(total_page_count);
        let mut page_descriptions = Vec::with_capacity(total_page_count);

        'next_chapter: for chapter in &processed_chapters {
            let chapter_path_string = match chapter.path.to_relative(Some(title_path.as_ref())) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(e) => {
                    warn!(
                        "can't convert chapter path `{}` to relative to get upserted chapter ID from hashmap: {e:?}",
                        chapter.path.as_ref().display()
                    );
                    continue 'next_chapter;
                }
            };
            let chapter_id = match chapter_path_to_id.get(&chapter_path_string) {
                Some(id) => *id,
                None => {
                    warn!("can't get chapter ID from hashmap");
                    continue 'next_chapter;
                }
            };

            match chapter.chapter_type {
                ChapterType::Archive(ref pages) => {
                    for page in pages {
                        chapter_ids.push(chapter_id);
                        page_ids.push(match page_ids_pool.pop() {
                            Some(id) => id,
                            None => {
                                warn!("can't get page ID from pool, this should never happen");
                                continue 'next_chapter;
                            }
                        });
                        page_paths.push(page.path.clone());
                        page_filesizes.push(page.size.unwrap_or_default());
                        page_descriptions.push(page.description.clone().unwrap_or_default());
                    }
                }
                ChapterType::Dir(ref pages) => {
                    for page in pages {
                        let page_path_string = match page
                            .path
                            .to_relative(Some(chapter.path.as_ref()))
                        {
                            Ok(p) => p.to_string_lossy().to_string(),
                            Err(e) => {
                                warn!(
                                    "can't convert page path `{}` to relative to upsert to database: {e:?}",
                                    page.path.as_ref().display()
                                );
                                continue 'next_chapter;
                            }
                        };

                        chapter_ids.push(chapter_id);
                        page_ids.push(match page_ids_pool.pop() {
                            Some(id) => id,
                            None => {
                                warn!("can't get page ID from pool, this should never happen");
                                continue 'next_chapter;
                            }
                        });
                        page_descriptions.push(page.description.clone().unwrap_or_default());
                        page_paths.push(page_path_string);
                        page_filesizes.push(page.size.unwrap_or_default());
                    }
                }
            };
        }

        sqlx::query!(
            "INSERT INTO chapters_pages (id, chapter_id, path, filesize, description)
            SELECT id, chapter_id, path, NULLIF(filesize, 0), NULLIF(description, '')
            FROM UNNEST($1::bigint[], $2::bigint[], $3::text[], $4::bigint[], $5::text[])
            AS t(id, chapter_id, path, filesize, description)
        ON CONFLICT (chapter_id, path) DO UPDATE
            SET description = EXCLUDED.description",
            &chapter_ids,
            &page_ids,
            &page_paths,
            &page_filesizes,
            &page_descriptions
        )
        .execute(&mut *txn)
        .await
        .context("can't upsert new pages")?;
    }

    Ok(title_id)
}
