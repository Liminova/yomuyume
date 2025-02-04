use std::{fs::DirEntry, os::unix::fs::MetadataExt};

use anyhow::Context;
use chrono::{DateTime, Utc};
use tracing::warn;

use crate::{
    library_processor::{upsert_oneshot::TitleHandlerOk, PageInTitle, UpsertTitleErr},
    structs::{
        absolute_path::{AbsolutePath, ToAbsolute},
        comic_info::{ComicInfo, ComicPageInfo, ComicPageType},
    },
    traits::{
        do_something_and_ok::DoSomethingAndOk, pathbuf_utils::PathBufUtils,
        to_blurhash::ToBlurhashFromFile, try_find_map::IteratorExt,
    },
    utils::{config::COMICINFO, macros::bail_if_empty},
};

#[derive(Debug, Clone)]
struct PageInDirTitle {
    rel_path: String,
    abs_path: AbsolutePath,
    last_modified: Option<DateTime<Utc>>,
    size: Option<i64>,
}

type PageInDirTitleTuple = (String, AbsolutePath, Option<DateTime<Utc>>, Option<i64>);
impl From<PageInDirTitleTuple> for PageInDirTitle {
    fn from(tuple: PageInDirTitleTuple) -> Self {
        Self {
            rel_path: tuple.0,
            abs_path: tuple.1,
            last_modified: tuple.2,
            size: tuple.3,
        }
    }
}

impl From<PageInDirTitle> for PageInTitle {
    fn from(page: PageInDirTitle) -> Self {
        Self {
            path: page.rel_path,
            last_modified: page.last_modified,
            size: page.size,
        }
    }
}

/// Treat an oneshot title as a directory and extract information from it into
/// [`TitleHandlerOk`].
///
/// Cover-related might be forgiving if there's a problem, but not ComicInfo.
pub fn handle_title_as_directory(
    title_path: &AbsolutePath,
    sub_entries: &[DirEntry],
    nomedia_support: bool,
) -> Result<TitleHandlerOk, UpsertTitleErr> {
    bail_if_empty!(sub_entries, Err(UpsertTitleErr::IsEmpty));

    let comicinfo_path = title_path.as_ref().join(COMICINFO);

    let (original_comicinfo, mut comicinfo) = if comicinfo_path.exists() {
        let s = std::fs::read_to_string(&comicinfo_path)
            .map_err(UpsertTitleErr::ComicInfoReadFromFs)?;
        let tmp = ComicInfo::from_str(&s).map_err(UpsertTitleErr::ComicInfoParse)?;
        (tmp.clone(), tmp)
    } else {
        (ComicInfo::default(), ComicInfo::default())
    };

    let mut cover_path: Option<String> = None;
    let mut cover_blurhash: Option<String> = None;
    let mut cover_width: Option<i32> = None;
    let mut cover_height: Option<i32> = None;

    let mut sub_pages: Vec<PageInDirTitle> = Vec::with_capacity(sub_entries.len());
    let mut pages_in_dir = sub_entries
        .iter()
        .filter(|e| {
            if e.path().is_file() {
                e.path().has_image_ext()
            } else {
                true
            }
        })
        .filter_map(|entry| {
            entry.path().to_absolute(None).okay(|e| {
                warn!(
                    "can't convert page path `{}` to absolute: {e:?}",
                    entry.path().display()
                );
            })
        })
        .filter_map(|abs_path| {
            if abs_path.is_file() {
                return Some(PageInDirTitle {
                    rel_path: abs_path
                        .to_relative(Some(title_path))
                        .okay(|e| warn!("can't strip title path from page path: {e:?}"))
                        .map(|p| p.to_string_lossy().to_string())?,
                    last_modified: abs_path
                        .last_modified()
                        .okay(|e| warn!("can't get last modified date of page file: {e:?}")),
                    size: abs_path
                        .metadata()
                        .okay(|e| warn!("can't get metadata of page file: {e:?}"))
                        .map(|m| m.size() as i64),
                    abs_path,
                });
            }
            sub_pages.extend(
                abs_path
                    .as_ref()
                    .scan_dir_recursively_for_image(nomedia_support)
                    .iter()
                    .filter_map(|p| {
                        p.to_absolute(None).okay(|e| {
                            warn!(
                                "can't convert page path `{}` to absolute: {e:?}",
                                p.display()
                            );
                        })
                    })
                    .filter_map(|abs_path| {
                        Some(PageInDirTitle {
                            rel_path: abs_path
                                .to_relative(Some(title_path))
                                .okay(|e| warn!("can't strip title path from page path: {e:?}"))
                                .map(|rel_p| rel_p.to_string_lossy().to_string())?,
                            last_modified: abs_path.last_modified().okay(|e| {
                                warn!("can't get last modified date of page file: {e:?}");
                            }),
                            size: abs_path
                                .metadata()
                                .okay(|e| warn!("can't get metadata of page file: {e:?}"))
                                .map(|m| m.size() as i64),
                            abs_path,
                        })
                    }),
            );
            None
        })
        .collect::<Vec<_>>();
    pages_in_dir.extend(sub_pages);
    bail_if_empty!(pages_in_dir, Err(UpsertTitleErr::IsEmpty));

    'cover_finder: {
        // use `page_abs_path` to interact with the file
        // use `page_cfg_path` to store in ComicInfo and DB
        let mut page_abs_path = None;
        let mut page_modified = None;
        let page_cfg = comicinfo
            .pages_mut()
            .iter_mut()
            // is configured as Cover
            .filter(|pc| pc.page_type == ComicPageType::FrontCover)
            .filter(|page_cfg| page_cfg.image_path.is_some())
            .find(|page_cfg| {
                let Some(ref page_cfg_path) = page_cfg.image_path else {
                    unreachable!()
                };

                pages_in_dir
                    .iter()
                    .filter_map(|page| page.last_modified.map(|m| (page, m)))
                    .any(|(page, modified)| {
                        let Some(page_rel_path) = page
                            .abs_path
                            .to_relative(Some(title_path))
                            .okay(|e| warn!("can't strip title path from page path: {e:?}"))
                            .map(|p| p.to_string_lossy().to_string())
                        else {
                            return false;
                        };

                        if page_rel_path == *page_cfg_path {
                            page_abs_path = Some(page.abs_path.clone());
                            page_modified = Some(modified);
                            return true;
                        }
                        false
                    })
            });

        // and use it if the fields are valid or encode-able to blurhash
        if let Some((page_cfg, (page_abs_path, modified))) =
            page_cfg.zip(page_abs_path.zip(page_modified))
        {
            // use the configured page if lgtm
            if page_cfg
                .blurhash
                .as_ref()
                .zip(page_cfg.modified_date_at_encode.as_ref())
                .filter(|(_, page_cfg_modified)| {
                    let valid_dimension = page_cfg.image_width > 0 && page_cfg.image_height > 0;
                    let unmodified = **page_cfg_modified == modified;

                    valid_dimension && unmodified
                })
                .map(|(bh, _)| {
                    cover_path.clone_from(&page_cfg.image_path);
                    cover_blurhash.clone_from(&Some(bh.clone()));
                    cover_width.clone_from(&Some(page_cfg.image_width));
                    cover_height.clone_from(&Some(page_cfg.image_height));
                })
                .is_some()
            {
                break 'cover_finder;
            }

            // try re-encode the configured page
            if page_abs_path
                .as_ref()
                .to_blurhash_from_file()
                .okay(|e| warn!("can't re-encode cover file: {e:?}"))
                .map(|bh_result| {
                    page_cfg.blurhash = Some(bh_result.blurhash.clone());
                    page_cfg.image_width = bh_result.width;
                    page_cfg.image_height = bh_result.height;
                    page_cfg.modified_date_at_encode = Some(modified);

                    cover_path.clone_from(&page_cfg.image_path);
                    cover_blurhash.clone_from(&Some(bh_result.blurhash));
                    cover_width.clone_from(&Some(page_cfg.image_width));
                    cover_height.clone_from(&Some(page_cfg.image_height));
                })
                .is_some()
            {
                break 'cover_finder;
            }
        }

        // else try every single pages
        pages_in_dir
            .iter()
            .filter_map(|page| page.last_modified.map(|m| (page, m)))
            .try_find_map(|(page, modified)| {
                page.abs_path
                    .as_ref()
                    .to_blurhash_from_file()
                    .context(anyhow::anyhow!(
                        "can't use `{}` as cover for `{title_path}`",
                        page.abs_path
                    ))
                    .map(|blurhash| (page, blurhash, modified))
            })
            .map(|(page, blurhash, modified)| {
                comicinfo.pages_mut().insert(
                    0,
                    ComicPageInfo {
                        page_type: ComicPageType::FrontCover,
                        blurhash: Some(blurhash.blurhash.clone()),
                        image_path: Some(page.rel_path.clone()),
                        image_width: blurhash.width,
                        image_height: blurhash.height,
                        modified_date_at_encode: Some(modified),
                        ..Default::default()
                    },
                );

                cover_path = Some(page.rel_path.clone());
                cover_blurhash = Some(blurhash.blurhash);
                cover_width = Some(blurhash.width);
                cover_height = Some(blurhash.height);
            })
            .okay(|e| warn!("no file in `{title_path}` can be use as cover: {e:?}"));
    };

    if comicinfo != original_comicinfo {
        comicinfo_path
            .create_file_if_not_exists()
            .map_err(UpsertTitleErr::ComicInfoWriteDir)?;

        let s = comicinfo
            .to_pretty_string()
            .map_err(UpsertTitleErr::ComicInfoSerialize)?;
        std::fs::write(&comicinfo_path, s.as_bytes()).map_err(UpsertTitleErr::ComicInfoWriteDir)?;
    }

    Ok(TitleHandlerOk {
        comicinfo,
        title_last_modified: pages_in_dir.iter().filter_map(|p| p.last_modified).max(),
        pages_in_title: pages_in_dir.into_iter().map(|p| p.into()).collect(),
        cover_path,
        cover_blurhash,
        cover_width,
        cover_height,
        is_dir: true,
    })
}
