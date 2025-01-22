use std::sync::Arc;

use anyhow::{anyhow, Context};
use tracing::warn;

use crate::{
    library_processor::{upsert_oneshot::TitleHandlerOk, UpsertTitleErr},
    traits::{
        do_something_and_ok::DoSomethingAndOk, to_blurhash::ToBlurhashFromArchive,
        try_find_map::IteratorExt,
    },
    types::{
        absolute_path::AbsolutePath,
        comic_info::{ComicInfo, ComicPageInfo, ComicPageType},
    },
    utils::{
        archive_file::{ArchiveFile, ItemInArchive, ItemsInArchiveUtils},
        config::COMICINFO_FILENAME,
        macros::bail_if_empty,
    },
};

/// Treat an oneshot title as an archive and extract information from it into
/// [`TitleHandlerOk`].
///
/// Cover-related might be forgiving if there's a problem, but not ComicInfo.
pub fn handle_title_as_archive(
    title_path: &AbsolutePath,
    files_in_archive: Vec<ItemInArchive>,
    nomedia_support: bool,
) -> Result<TitleHandlerOk, UpsertTitleErr> {
    let pages_in_archive = files_in_archive.keep_images(nomedia_support);
    bail_if_empty!(pages_in_archive, Err(UpsertTitleErr::IsEmpty));

    let (original_comicinfo, mut comicinfo) = {
        let tmp = title_path
            .as_ref()
            .read_file_from_archive(COMICINFO_FILENAME)
            .map_err(UpsertTitleErr::ComicInfoExtract)
            .and_then(|b| String::from_utf8(b).map_err(UpsertTitleErr::ComicInfoReadFromVecU8))
            .and_then(|s| ComicInfo::from_str(&s).map_err(UpsertTitleErr::ComicInfoParse))?;
        (tmp.clone(), tmp)
    };

    let mut cover_path: Option<String> = None;
    let mut cover_blurhash: Option<String> = None;
    let mut cover_width: Option<i32> = None;
    let mut cover_height: Option<i32> = None;

    'cover_finder: {
        // check if there's a configured cover
        let mut page_modified = None;
        let page_cfg = comicinfo
            .pages_mut()
            .iter_mut()
            .rev()
            .filter(|p| p.page_type == ComicPageType::FrontCover)
            .filter(|page_cfg| page_cfg.image_path.is_some())
            .find(|page_cfg| {
                pages_in_archive
                    .iter()
                    .filter_map(|p| p.last_modified.map(|m| (p, m)))
                    .any(|(p, modified)| {
                        let Some(ref page_cfg_path) = page_cfg.image_path else {
                            unreachable!()
                        };

                        if p.path == *page_cfg_path {
                            page_modified = Some(modified);
                            true
                        } else {
                            false
                        }
                    })
            });

        // and use it if the fields are valid or encode-able to blurhash
        if let Some((page_cfg, modified)) = page_cfg.zip(page_modified) {
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
                    cover_path.clone_from(&page_cfg.image_path);
                    cover_blurhash.clone_from(&Some(bh.clone()));
                    cover_width.clone_from(&Some(page_cfg.image_width));
                    cover_height.clone_from(&Some(page_cfg.image_height));
                })
                .is_some()
            {
                break 'cover_finder;
            };

            let Some(ref page_cfg_path) = page_cfg.image_path else {
                unreachable!()
            };

            // try re-encode if something went wrong
            if title_path
                .as_ref()
                .to_blurhash_from_archive(page_cfg_path)
                .okay(|e| warn!("can't re-encode cover file: {e:?}"))
                .map(|blurhash_result| {
                    page_cfg.blurhash = Some(blurhash_result.blurhash.clone());
                    page_cfg.image_width = blurhash_result.width;
                    page_cfg.image_height = blurhash_result.height;
                    page_cfg.modified_date_at_encode = Some(modified);

                    cover_path.clone_from(&page_cfg.image_path);
                    cover_blurhash.clone_from(&Some(blurhash_result.blurhash));
                    cover_width.clone_from(&Some(page_cfg.image_width));
                    cover_height.clone_from(&Some(page_cfg.image_height));
                })
                .is_some()
            {
                break 'cover_finder;
            }
        }

        // else try every single pages
        pages_in_archive
            .iter()
            .filter_map(|page| page.last_modified.map(|m| (page, m)))
            .try_find_map(|(page, modified)| {
                title_path
                    .as_ref()
                    .to_blurhash_from_archive(&page.path)
                    .context(anyhow!(
                        "can't use `{}` as cover for `{title_path}`",
                        page.path
                    ))
                    .map(|blurhash| (page.path.clone(), modified, blurhash))
            })
            .map(|(path, modified, blurhash)| {
                comicinfo.pages_mut().insert(
                    0,
                    ComicPageInfo {
                        page_type: ComicPageType::FrontCover,
                        blurhash: Some(blurhash.blurhash.clone()),
                        image_path: Some(path.clone()),
                        image_width: blurhash.width,
                        image_height: blurhash.height,
                        modified_date_at_encode: Some(modified),
                        ..Default::default()
                    },
                );
                cover_path = Some(path);
                cover_blurhash = Some(blurhash.blurhash);
                cover_width = Some(blurhash.width);
                cover_height = Some(blurhash.height);
            })
            .okay(|e| warn!("no file in oneshot archive can be use as cover: {e:?}"));
    };

    if comicinfo != original_comicinfo {
        comicinfo
            .to_pretty_string()
            .map_err(UpsertTitleErr::ComicInfoSerialize)
            .and_then(|s| {
                title_path
                    .as_ref()
                    .upsert_file_to_archive(COMICINFO_FILENAME, Arc::new(s.as_bytes().to_vec()))
                    .map_err(UpsertTitleErr::ComicInfoWriteArchive)
            })?;
    }

    Ok(TitleHandlerOk {
        comicinfo,
        title_last_modified: title_path
            .last_modified()
            .okay(|e| warn!("can't get last modified date of oneshot archive: {e:?}")),
        pages_in_title: pages_in_archive,
        cover_path,
        cover_blurhash,
        cover_width,
        cover_height,
        is_dir: false,
    })
}
