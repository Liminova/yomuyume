use std::{fs::DirEntry, os::unix::fs::MetadataExt};

use anyhow::Context;
use chrono::{DateTime, Utc};

use crate::{
    config::COMICINFO_FILENAME,
    library_processor::{
        blurhash_encode::encode_blurhash, upsert_oneshot::TitleHandlerOk, PageInTitle,
        UpsertTitleErr,
    },
    macros::bail_if_empty,
    traits::{IteratorExt, PathBufUtils, WarnResultThenOk},
    types::{
        absolute_path::{AbsolutePath, ToAbsolute},
        comic_info::{ComicInfo, ComicPageInfo, ComicPageType},
    },
};

#[derive(Debug, Clone)]
struct PageInDirTitle {
    rel_path: String,
    abs_path: AbsolutePath,
    last_modified: DateTime<Utc>,
    size: Option<i64>,
}

impl From<(String, AbsolutePath, DateTime<Utc>, i64)> for PageInDirTitle {
    fn from(tuple: (String, AbsolutePath, DateTime<Utc>, i64)) -> Self {
        Self {
            rel_path: tuple.0,
            abs_path: tuple.1,
            last_modified: tuple.2,
            size: Some(tuple.3),
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
    sub_entries: Vec<DirEntry>,
    nomedia_support: bool,
) -> Result<TitleHandlerOk, UpsertTitleErr> {
    bail_if_empty!(sub_entries, Err(UpsertTitleErr::IsEmpty));

    let comicinfo_path = title_path.as_ref().join(COMICINFO_FILENAME);

    let (original_comicinfo, mut comicinfo) = if !comicinfo_path.exists() {
        (ComicInfo::default(), ComicInfo::default())
    } else {
        let s = std::fs::read_to_string(&comicinfo_path)
            .map_err(UpsertTitleErr::ComicInfoReadFromFs)?;
        let tmp = ComicInfo::from_str(&s).map_err(UpsertTitleErr::ComicInfoParse)?;
        (tmp.clone(), tmp)
    };

    let mut cover_path: Option<String> = None;
    let mut cover_blurhash: Option<String> = None;
    let mut cover_width: Option<i32> = None;
    let mut cover_height: Option<i32> = None;

    let mut sub_pages: Vec<PageInDirTitle> = Vec::with_capacity(sub_entries.len());
    let mut pages_in_dir = sub_entries
        .iter()
        // dir -> continue | file -> must be image
        .filter(|e| match e.path().is_dir() {
            true => true,
            false => e.path().has_image_ext(),
        })
        // -> AbsolutePath
        .filter_map(|e| {
            e.path().to_absolute(None).okay(format!(
                "can't convert page path `{}` to absolute",
                e.path().display()
            ))
        })
        // -> AbsolutePath, RelativePath
        .filter_map(|abs_p| {
            abs_p
                .to_relative(Some(title_path))
                .okay(format!(
                    "can't convert page path `{abs_p}` to relative to title's path"
                ))
                .map(|rel_p| rel_p.to_string_lossy().to_string())
                .map(|rel_p| (abs_p, rel_p))
        })
        // -> AbsolutePath, RelativePath, Size
        .filter_map(|(abs_p, rel_p)| {
            abs_p
                .as_ref()
                .metadata()
                .okay(format!("can't get metadata of page file `{abs_p}`"))
                .map(|m| (abs_p, rel_p, m.size() as i64))
        })
        // -> AbsolutePath, RelativePath, Modified, Size
        .filter_map(|(abs_p, rel_p, s)| {
            abs_p
                .as_ref()
                .last_modified()
                .okay(format!("can't get modified date of page file `{abs_p}`"))
                .map(|d| (abs_p, rel_p, d, s))
        })
        // -> PageInDirTitle
        .filter_map(|(abs_p, rel_p, m, size)| {
            if abs_p.as_ref().is_file() {
                return Some((rel_p, abs_p, m, size).into());
            }
            sub_pages.extend(
                abs_p
                    .as_ref()
                    .scan_dir_recursively_for_image(nomedia_support)
                    .iter()
                    // -> AbsolutePath
                    .filter_map(|p| {
                        p.to_absolute(None).okay(format!(
                            "can't convert page path `{}` to absolute",
                            p.display()
                        ))
                    })
                    // -> AbsolutePath, RelativePath
                    .filter_map(|abs_p| {
                        abs_p
                            .to_relative(Some(title_path))
                            .okay(format!("can't convert page path `{abs_p}` to absolute"))
                            .map(|rel_p| rel_p.to_string_lossy().to_string())
                            .map(|rel_p| (abs_p, rel_p))
                    })
                    // -> AbsolutePath, RelativePath, Size
                    .filter_map(|(abs_p, rel_p)| {
                        abs_p
                            .as_ref()
                            .metadata()
                            .okay(format!("can't get metadata of `{abs_p}`"))
                            .map(|m| (abs_p, rel_p, m.size() as i64))
                    })
                    // -> AbsolutePath, RelativePath, Modified, Size
                    .filter_map(|(abs_p, rel_p, s)| {
                        abs_p
                            .as_ref()
                            .last_modified()
                            .okay(format!(
                                "can't get last modified date of page file `{abs_p}`"
                            ))
                            .map(|d| (rel_p, abs_p, d, s))
                    })
                    // -> PageInDirTitle
                    .map(|res| res.into()),
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
        let page_cfg_and_path = comicinfo
            .pages_mut()
            .iter_mut()
            .rev()
            // is configured as Cover
            .filter(|pc| pc.page_type == ComicPageType::FrontCover)
            // contains page path
            .filter_map(|page_cfg| {
                page_cfg
                    .image_path
                    .clone()
                    .map(|page_cfg_path| (page_cfg, page_cfg_path))
            })
            .find(|(_, page_cfg_path)| {
                pages_in_dir.iter().any(|page| {
                    let page_rel_path = match page
                        .abs_path
                        .to_relative(Some(title_path))
                        .okay(format!(
                            "can't convert `{}` to relative to title's path",
                            page.abs_path
                        ))
                        .map(|p| p.to_string_lossy().to_string())
                    {
                        Some(p) => p,
                        None => return false,
                    };

                    if page_rel_path == *page_cfg_path {
                        page_abs_path = Some(page.abs_path.clone());
                        page_modified = Some(page.last_modified);
                        return true;
                    }
                    false
                })
            });

        // and use it if the fields are valid or encode-able to blurhash
        if let Some(((page_cfg, page_cfg_path), (page_abs_path, modified))) =
            page_cfg_and_path.zip(page_abs_path.zip(page_modified))
        {
            // use the configured page if lgtm
            if page_cfg
                .blurhash
                .as_ref()
                .zip(page_cfg.modified_date_at_encode.as_ref())
                .filter(|(_, m)| {
                    let valid_dimension = page_cfg.image_width > 0 && page_cfg.image_height > 0;
                    let unmodified = **m == modified;

                    valid_dimension && unmodified
                })
                .map(|(bh, _)| {
                    cover_path = Some(page_cfg_path.clone());
                    cover_blurhash = Some(bh.clone());
                    cover_width = Some(page_cfg.image_width);
                    cover_height = Some(page_cfg.image_height);
                })
                .is_some()
            {
                break 'cover_finder;
            }

            // try re-encode the configured page
            if std::fs::read(page_abs_path.as_ref())
                .context("can't read file")
                .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                .and_then(|img| encode_blurhash(&img).context("can't encode to blurhash"))
                .okay(format!(
                    "can't use `{page_abs_path}` as cover for `{title_path}`"
                ))
                .map(|bh_result| {
                    page_cfg.blurhash = Some(bh_result.blurhash.clone());
                    page_cfg.image_width = bh_result.width;
                    page_cfg.image_height = bh_result.height;
                    page_cfg.modified_date_at_encode = Some(modified);

                    cover_path = Some(page_cfg_path);
                    cover_blurhash = Some(bh_result.blurhash);
                    cover_width = Some(page_cfg.image_width);
                    cover_height = Some(page_cfg.image_height);
                })
                .is_some()
            {
                break 'cover_finder;
            }
        }

        // else try every single pages
        pages_in_dir
            .iter()
            .try_find_map(|real_p| {
                std::fs::read(real_p.abs_path.as_ref())
                    .context("can't read file")
                    .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                    .and_then(|img| encode_blurhash(&img).context("can't encode to blurhash"))
                    .map(|blurhash| (real_p, blurhash))
            })
            .map(|(page, blurhash)| {
                comicinfo.pages_mut().push(ComicPageInfo {
                    page_type: ComicPageType::FrontCover,
                    blurhash: Some(blurhash.blurhash.clone()),
                    image_path: Some(page.rel_path.clone()),
                    image_width: blurhash.width,
                    image_height: blurhash.height,
                    modified_date_at_encode: Some(page.last_modified),
                    ..Default::default()
                });

                cover_path = Some(page.rel_path.clone());
                cover_blurhash = Some(blurhash.blurhash);
                cover_width = Some(blurhash.width);
                cover_height = Some(blurhash.height);
            })
            .okay(format!("no file in `{title_path}` can be use as cover"));
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
        title_last_modified: pages_in_dir.iter().map(|p| &p.last_modified).max().cloned(),
        pages_in_title: pages_in_dir.into_iter().map(|p| p.into()).collect(),
        cover_path,
        cover_blurhash,
        cover_width,
        cover_height,
        is_dir: true,
    })
}
