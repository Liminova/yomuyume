use std::sync::Arc;

use anyhow::Context;

use crate::{
    archive_file::{ArchiveFile, ItemInArchive, ItemsInArchiveUtils},
    config::COMICINFO_FILENAME,
    library_processor::{
        blurhash_encode::encode_blurhash, upsert_oneshot::TitleHandlerOk, UpsertTitleErr,
    },
    macros::bail_if_empty,
    traits::{IteratorExt, PathBufUtils, WarnResultThenOk},
    types::{
        absolute_path::AbsolutePath,
        comic_info::{ComicInfo, ComicPageInfo, ComicPageType},
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
        let page_cfg_and_path = comicinfo
            .pages_mut()
            .iter_mut()
            .rev()
            // is configured as Cover
            .filter(|p| p.page_type == ComicPageType::FrontCover)
            // contains page path
            .filter_map(|page_cfg| {
                page_cfg
                    .image_path
                    .clone()
                    .map(|page_cfg_path| (page_cfg, page_cfg_path))
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
        if let Some(((page_cfg, page_cfg_path), modified)) = page_cfg_and_path.zip(page_modified) {
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
            };

            // try re-encode if something went wrong
            if title_path
                .as_ref()
                .read_file_from_archive(&page_cfg_path)
                .context("can't read from archive")
                .and_then(|buf| image::load_from_memory(&buf).context("can't parse as image"))
                .and_then(|img| encode_blurhash(&img).context("can't encode to blurhash"))
                .okay(format!(
                    "can't use `{page_cfg_path}` as cover for `{title_path}`"
                ))
                .map(|blurhash_result| {
                    page_cfg.blurhash = Some(blurhash_result.blurhash.clone());
                    page_cfg.image_width = blurhash_result.width;
                    page_cfg.image_height = blurhash_result.height;
                    page_cfg.modified_date_at_encode = Some(modified);

                    cover_path = Some(page_cfg_path);
                    cover_blurhash = Some(blurhash_result.blurhash);
                    cover_width = Some(page_cfg.image_width);
                    cover_height = Some(page_cfg.image_height);
                })
                .is_some()
            {
                break 'cover_finder;
            }
        }

        // else try every single pages
        pages_in_archive
            .iter()
            .try_find_map(|real_p| {
                title_path
                    .as_ref()
                    .read_file_from_archive(&real_p.path)
                    .context("can't read from archive")
                    .and_then(|buf| image::load_from_memory(&buf).context("can't parse as image"))
                    .and_then(|img| encode_blurhash(&img).context("can't encode to blurhash"))
                    .map(|blurhash| (real_p.path.clone(), real_p.last_modified, blurhash))
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
            .okay(format!("no file in `{title_path}` can be use as cover"));
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
            })?
    }

    Ok(TitleHandlerOk {
        comicinfo,
        title_last_modified: title_path
            .as_ref()
            .last_modified()
            .okay(format!("can't get last modified date of {title_path}")),
        pages_in_title: pages_in_archive,
        cover_path,
        cover_blurhash,
        cover_width,
        cover_height,
        is_dir: false,
    })
}
