use anyhow::Context;
use tracing::warn;

use crate::{
    library_processor::upsert_series::{ChapterInfo, ChapterType, PageInArchive},
    structs::{absolute_path::AbsolutePath, comic_info::ComicInfo},
    traits::{do_something_and_ok::DoSomethingAndOk, pathbuf_utils::PathBufUtils},
    utils::{
        archive_file::{ArchiveFile, ItemInArchive, ItemsInArchiveUtils},
        constants::COMICINFO,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum HandleArchiveChapterErr {
    #[error("it's empty")]
    IsEmpty,
}

pub fn handle_archive_chapter(
    chapter_path: AbsolutePath,
    backup_volume_number: i32,
    items_in_archive: Vec<ItemInArchive>,
    nomedia_support: bool,
) -> Result<ChapterInfo, HandleArchiveChapterErr> {
    let pages_in_archive = items_in_archive.keep_images(nomedia_support);
    if pages_in_archive.is_empty() {
        return Err(HandleArchiveChapterErr::IsEmpty);
    }

    let chapter_comicinfo = chapter_path
        .as_ref()
        .read_file_from_archive(COMICINFO)
        .context("can't extract from archive")
        .and_then(|buf| String::from_utf8(buf).context("can't decode to string"))
        .and_then(|str| ComicInfo::from_str(&str).context("can't deserialize"))
        .okay(|e| warn!("can't parse {COMICINFO} in archive-chapter: {e:?}"))
        .unwrap_or_default();

    Ok(ChapterInfo {
        volume: match chapter_comicinfo.volume {
            v if v != -1 => v,
            _ => backup_volume_number,
        },
        modified: chapter_path
            .as_ref()
            .last_modified()
            .okay(|e| warn!("can't get last modified date of archive-chapter: {e:?}")),
        chapter_type: ChapterType::Archive(
            pages_in_archive
                .into_iter()
                .map(|p| PageInArchive {
                    description: chapter_comicinfo.get_page_description(&p.path),
                    path: p.path,
                    size: p.size,
                    last_modified: p.last_modified,
                })
                .collect(),
        ),
        description: chapter_comicinfo.summary,
        path: chapter_path,
    })
}
