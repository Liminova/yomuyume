use std::fs::read_to_string;

use tracing::warn;

use crate::{
    library_processor::upsert_series::{ChapterInfo, ChapterType, PageInDirectory},
    structs::{
        absolute_path::{AbsolutePath, ToAbsolute},
        comic_info::ComicInfo,
    },
    traits::{do_something_and_ok::DoSomethingAndOk, pathbuf_utils::PathBufUtils},
    utils::constants::COMICINFO,
};

#[derive(Debug, thiserror::Error)]
pub enum HandleDirectoryChapterErr {
    #[error("it's empty")]
    IsEmpty,

    #[error("can't read directory-chapter: {0:?}")]
    ReadDir(std::io::Error),
}

pub fn handle_directory_chapter(
    chapter_path: AbsolutePath,
    backup_volume_number: i32,
    nomedia_support: bool,
) -> Result<ChapterInfo, HandleDirectoryChapterErr> {
    let chapter_comicinfo = {
        let path = chapter_path.as_ref().join(COMICINFO);
        path.exists()
            .then(|| {
                read_to_string(&path)
                    .okay(|e| warn!("can't read `{COMICINFO}` in directory-chapter `{chapter_path}`: {e:?}"))
                    .and_then(|s| {
                        ComicInfo::from_str(&s)
                        .okay(|e| warn!("can't parse `{COMICINFO}` in directory-chapter `{chapter_path}`: {e:?}"))
                    })
            })
            .flatten()
            .unwrap_or_default()
    };

    // gotta have an empty vec first to store the sub-pages
    let mut subpages_in_chapter: Vec<PageInDirectory> = vec![];
    let mut pages_in_chapter = chapter_path
        .as_ref()
        .read_dir()
        .map_err(HandleDirectoryChapterErr::ReadDir)?
        // filter out any error
        .filter_map(|item| item.okay(|e| warn!("can't read page in directory-chapter: {e:?}")))
        // dir -> continue | file -> must be image
        .filter(|e| {
            if e.path().is_dir() {
                true
            } else {
                e.path().has_image_ext()
            }
        })
        // -> AbsolutePath
        .filter_map(|e| {
            e.path()
                .to_absolute(None)
                .okay(|e| warn!("can't convert page path to absolute: {e:?}"))
        })
        // -> AbsolutePath, RelativePath
        .filter_map(|abs_path| {
            abs_path
                .to_relative(Some(&chapter_path))
                .okay(|e| warn!("can't strip title path from page path: {e:?}"))
                .map(|p| (abs_path, p.to_string_lossy().to_string()))
        })
        // -> AbsolutePath, RelativePath, Size
        .filter_map(|(abs_path, rel_path)| {
            abs_path
                .metadata()
                .okay(|e| warn!("can't get page metadata: {e:?}"))
                .map(|m| (abs_path, rel_path, m.len() as i64))
        })
        // -> AbsolutePath, RelativePath, Modified, Size
        .map(|(abs_path, rel_path, size)| {
            let m = abs_path
                .as_ref()
                .last_modified()
                .okay(|e| warn!("can't get page modified date: {e:?}"));
            (abs_path, rel_path, m, size)
        })
        // -> DirChapterPage
        .filter_map(|(abs_path, rel_path, last_modified, size)| {
            if abs_path.is_file() {
                return Some(PageInDirectory {
                    path: abs_path,
                    size: Some(size),
                    last_modified,
                    description: chapter_comicinfo.get_page_description(&rel_path),
                });
            }

            subpages_in_chapter.extend(
                abs_path
                    .as_ref()
                    .scan_dir_recursively_for_image(nomedia_support)
                    .into_iter()
                    // -> AbsolutePath
                    .filter_map(|p| {
                        p.to_absolute(None)
                            .okay(|e| warn!("can't convert page path to absolute: {e:?}"))
                    })
                    // -> AbsolutePath, RelativePath
                    .filter_map(|abs_p| {
                        abs_p
                            .to_relative(Some(&chapter_path))
                            .okay(|e| warn!("can't strip title path from page path: {e:?}"))
                            .map(|rel_p| (abs_p, rel_p.to_string_lossy().to_string()))
                    })
                    // -> AbsolutePath, RelativePath, Size
                    .filter_map(|(abs_p, rel_p)| {
                        abs_p
                            .metadata()
                            .okay(|e| warn!("can't get page metadata: {e:?}"))
                            .map(|m| (abs_p, rel_p, m.len() as i64))
                    })
                    // -> AbsolutePath, RelativePath, Modified, Size
                    .map(|(abs_p, rel_p, s)| {
                        let m = abs_p
                            .last_modified()
                            .okay(|e| warn!("can't get page modified date: {e:?}"));
                        (abs_p, rel_p, m, s)
                    })
                    // -> DirChapterPage
                    .map(|(abs_p, rel_p, m, s)| PageInDirectory {
                        path: abs_p,
                        size: Some(s),
                        last_modified: m,
                        description: chapter_comicinfo.get_page_description(&rel_p),
                    }),
            );
            None
        })
        .collect::<Vec<_>>();

    pages_in_chapter.extend(subpages_in_chapter);
    if pages_in_chapter.is_empty() {
        return Err(HandleDirectoryChapterErr::IsEmpty);
    }

    Ok(ChapterInfo {
        volume: match chapter_comicinfo.volume {
            v if v != -1 => v,
            _ => backup_volume_number,
        },
        modified: pages_in_chapter
            .iter()
            .filter_map(|p| p.last_modified)
            .max(),
        chapter_type: ChapterType::Directory(pages_in_chapter),
        description: chapter_comicinfo.summary,
        path: chapter_path,
    })
}
