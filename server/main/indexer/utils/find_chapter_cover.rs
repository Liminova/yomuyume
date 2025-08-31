use crate::{
    database::{self, content::PageIdentityPath},
    indexer::utils::IndexedChapterPages,
    utils::{
        average_color::HexColor,
        comic_info::{ComicPageInfo, ComicPageType},
    },
};

pub fn find_chapter_cover(
    chapter_pages: &IndexedChapterPages,
    comic_info_pages: Option<&[ComicPageInfo]>,
    pages_in_db: &[(PageIdentityPath, database::content::PageInfo)],
) -> Option<(PageIdentityPath, HexColor)> {
    let pages_in_db_preview = {
        let mut tmp = pages_in_db
            .iter()
            .filter(|(page_identity_path, _)| !chapter_pages.delete.contains(page_identity_path))
            .filter_map(|(page_identity_path, page_info)| {
                page_info.color.map(|color| (page_identity_path, color))
            })
            .chain(
                chapter_pages
                    .upsert
                    .iter()
                    .filter_map(|p| p.color.map(|color| (&p.identity_path, color))),
            )
            .collect::<Vec<_>>();
        tmp.sort_by(|a, b| b.0.cmp(&a.0));
        tmp
    };

    if pages_in_db_preview.is_empty() {
        return None;
    }

    if let Some(comic_info_pages) = comic_info_pages {
        comic_info_pages
            .iter()
            .filter(|p| p.page_type == ComicPageType::FrontCover)
            .map(|p| p.image)
            .find_map(|idx| pages_in_db_preview.get(idx as usize))
            .or_else(|| pages_in_db_preview.get(0))
            .map(|p| ((*p.0).clone(), p.1))
    } else {
        pages_in_db_preview.get(0).map(|p| ((*p.0).clone(), p.1))
    }
}
