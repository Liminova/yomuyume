use tracing::warn;

use crate::{
    library_processor::upsert_series::{ChapterInfo, ChapterType},
    traits::{
        do_something_and_ok::DoSomethingAndOk,
        to_blurhash::{ToBlurhashFromArchive, ToBlurhashFromFile},
    },
    types::comic_info::{ComicInfo, ComicPageInfo},
};

pub fn try_everything_as_cover(
    comicinfo: &mut ComicInfo,

    handled_chapters: &[ChapterInfo],

    cover_path: &mut Option<String>,
    cover_blurhash: &mut Option<String>,
    cover_width: &mut Option<i32>,
    cover_height: &mut Option<i32>,
) -> Option<()> {
    for chapter in handled_chapters {
        if let ChapterType::Archive(ref pages) = chapter.chapter_type {
            for page in pages {
                if let Some(bh_result) = chapter
                    .path
                    .as_ref()
                    .to_blurhash_from_archive(&page.path)
                    .okay(|e| warn!("can't encode `{}` to blurhash: {e:?}", page.path))
                {
                    cover_path.clone_from(&Some(page.path.clone()));
                    cover_blurhash.clone_from(&Some(bh_result.blurhash.clone()));
                    cover_width.clone_from(&Some(bh_result.width));
                    cover_height.clone_from(&Some(bh_result.height));

                    comicinfo.pages_mut().insert(
                        0,
                        ComicPageInfo {
                            image_path: Some(page.path.clone()),
                            image_width: bh_result.width,
                            image_height: bh_result.height,
                            blurhash: Some(bh_result.blurhash),
                            modified_date_at_encode: page.last_modified,
                            ..Default::default()
                        },
                    );

                    return Some(());
                }
            }
        } else if let ChapterType::Directory(ref pages) = chapter.chapter_type {
            for page in pages {
                if let Some(bh_result) = page
                    .path
                    .as_ref()
                    .to_blurhash_from_file()
                    .okay(|e| warn!("can't encode `{}` to blurhash: {e:?}", page.path))
                {
                    cover_path.clone_from(&match page
                        .path
                        .to_relative(Some(&chapter.path))
                        .map(|p| p.to_string_lossy().to_string())
                        .okay(|e| warn!("can't strip chapter path from page path: {e:?}"))
                    {
                        Some(p) => Some(p),
                        None => continue,
                    });
                    cover_blurhash.clone_from(&Some(bh_result.blurhash.clone()));
                    cover_width.clone_from(&Some(bh_result.width));
                    cover_height.clone_from(&Some(bh_result.height));

                    comicinfo.pages_mut().insert(
                        0,
                        ComicPageInfo {
                            image_path: cover_path.clone(),
                            image_width: bh_result.width,
                            image_height: bh_result.height,
                            blurhash: Some(bh_result.blurhash),
                            modified_date_at_encode: page.last_modified,
                            ..Default::default()
                        },
                    );

                    return Some(());
                }
            }
        }
    }
    None
}
