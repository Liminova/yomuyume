use tracing::warn;

use crate::{
    library_processor::upsert_series::{ChapterInfo, ChapterType, PageInArchive, PageInDirectory},
    traits::{
        do_something_and_ok::DoSomethingAndOk,
        to_blurhash::{ToBlurhashFromArchive, ToBlurhashFromFile},
    },
    types::{
        absolute_path::{AbsolutePath, ToAbsolute},
        comic_info::ComicPageInfo,
    },
};

enum MatchedPage<'a> {
    Archive(&'a PageInArchive),
    Directory(&'a PageInDirectory),
}

pub fn try_get_configured_cover(
    page_cfg: &ComicPageInfo,
    page_cfg_path: &str,

    title_path: &AbsolutePath,
    handled_chapters: &[ChapterInfo],

    cover_path: &mut Option<String>,
    cover_blurhash: &mut Option<String>,
    cover_width: &mut Option<i32>,
    cover_height: &mut Option<i32>,
) -> Option<()> {
    // in the same dir as ComicInfo.xml

    if !page_cfg_path.contains('/') {
        let page_cfg_abs_path = title_path
            .as_ref()
            .join(page_cfg_path)
            .to_absolute(None)
            .okay(|e| warn!("can't convert page path to absolute: {e:?}"))?;

        // unchanged configured cover
        let real_p_modified = page_cfg_abs_path
            .last_modified()
            .okay(|e| warn!("can't get last modified date of page file: {e:?}"));
        if page_cfg.blurhash.is_some()
            && page_cfg.blurhash != Some(String::new())
            && real_p_modified.is_some()
            && page_cfg.modified_date_at_encode == real_p_modified
            && page_cfg.image_width > 0
            && page_cfg.image_height > 0
        {
            cover_path.clone_from(&page_cfg.image_path);
            cover_blurhash.clone_from(&page_cfg.blurhash);
            cover_width.clone_from(&Some(page_cfg.image_width));
            cover_height.clone_from(&Some(page_cfg.image_height));
            return Some(());
        }

        // try re-encode the configured cover
        return page_cfg_abs_path
            .as_ref()
            .to_blurhash_from_file()
            .okay(|e| warn!("can't use `{page_cfg_abs_path}` as cover for `{title_path}`: {e:?}"))
            .map(|blurhash_result| {
                cover_path.clone_from(&page_cfg.image_path);
                cover_blurhash.clone_from(&Some(blurhash_result.blurhash));
                cover_width.clone_from(&Some(page_cfg.image_width));
                cover_height.clone_from(&Some(page_cfg.image_height));
            });
    }

    // in (archive/directory) chapter

    let splitted_page_cfg_path = &page_cfg_path.split_once('/')?;
    let page_cfg_chapter_abs_path = title_path
        .as_ref()
        .join(splitted_page_cfg_path.0)
        .to_absolute(None)
        .okay(|e| warn!("can't convert chapter path to absolute: {e:?}"))?;

    let matched_chapter = handled_chapters
        .iter()
        .find(|chapter| chapter.path == page_cfg_chapter_abs_path)?;

    let matched_page = match &matched_chapter.chapter_type {
        ChapterType::Archive(pages) => pages
            .iter()
            .find(|p| p.path == splitted_page_cfg_path.1)
            // .map(|p| (p.last_modified, None)),
            .map(MatchedPage::Archive),
        ChapterType::Directory(pages) => {
            let page_cfg_abs_path = page_cfg_chapter_abs_path
                .as_ref()
                .join(page_cfg_path)
                .to_absolute(None)
                .okay(|e| warn!("can't convert page path to absolute: {e:?}"))?;
            pages
                .iter()
                .find(|p| p.path == page_cfg_abs_path)
                .map(MatchedPage::Directory)
        }
    }?;

    let real_p_modified = match matched_page {
        MatchedPage::Archive(p) => p.last_modified,
        MatchedPage::Directory(p) => p.last_modified,
    };

    if page_cfg.blurhash.is_some()
        && page_cfg.blurhash != Some(String::new())
        && real_p_modified.is_some()
        && real_p_modified == page_cfg.modified_date_at_encode
        && page_cfg.image_width > 0
        && page_cfg.image_height > 0
    {
        cover_path.clone_from(&page_cfg.image_path);
        cover_blurhash.clone_from(&page_cfg.blurhash);
        cover_width.clone_from(&Some(page_cfg.image_width));
        cover_height.clone_from(&Some(page_cfg.image_height));
        return Some(());
    }

    // try re-encode the configured page
    match matched_page {
        MatchedPage::Archive(page) => {
            let blurhash = matched_chapter
                .path
                .as_ref()
                .to_blurhash_from_archive(&page.path)
                .okay(|e| {
                    warn!("can't use `{page_cfg_path}` as cover for `{title_path}`: {e:?}");
                })?;

            cover_path.clone_from(&page_cfg.image_path);
            cover_blurhash.clone_from(&Some(blurhash.blurhash));
            cover_width.clone_from(&Some(page_cfg.image_width));
            cover_height.clone_from(&Some(page_cfg.image_height));
            Some(())
        }
        // ChapterType::Directory(_) => {
        MatchedPage::Directory(page) => {
            let blurhash = page.path.as_ref().to_blurhash_from_file().okay(|e| {
                warn!(
                    "can't use `{}` as cover for `{title_path}`: {e:?}",
                    page.path
                );
            })?;

            cover_path.clone_from(&page_cfg.image_path);
            cover_blurhash.clone_from(&Some(blurhash.blurhash));
            cover_width.clone_from(&Some(page_cfg.image_width));
            cover_height.clone_from(&Some(page_cfg.image_height));
            Some(())
        }
    }
}
