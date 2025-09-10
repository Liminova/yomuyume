use crate::{
    indexer::utils::{IndexedChapterPages, PageInDB},
    utils::comic_info::{ComicPageInfo, ComicPageType},
};

pub fn find_chapter_cover_page_id(
    chapter_pages: &IndexedChapterPages,
    comic_info_pages: Option<&[ComicPageInfo]>,
    pages_in_db: &[PageInDB],
) -> Option<String> {
    let pages_have_avg_color = {
        let mut tmp = pages_in_db
            .iter()
            .filter(|p| !chapter_pages.delete.contains(&p.id))
            .filter_map(|p| p.avg_color.map(|_| (&p.path, &p.id)))
            .chain(
                chapter_pages
                    .upsert
                    .iter()
                    .filter_map(|p| p.avg_hex_color.as_ref().map(|_| (&p.path, &p.id))),
            )
            .collect::<Vec<_>>();
        tmp.sort_by(|a, b| b.0.cmp(a.0));
        tmp.into_iter().map(|(_, id)| id).collect::<Vec<_>>()
    };

    if pages_have_avg_color.is_empty() {
        return None;
    }

    if let Some(comic_info_pages) = comic_info_pages {
        comic_info_pages
            .iter()
            .filter(|p| p.page_type == ComicPageType::FrontCover)
            .map(|p| p.image)
            .find_map(|idx| pages_have_avg_color.get(idx as usize))
            .or_else(|| pages_have_avg_color.get(0))
    } else {
        pages_have_avg_color.get(0)
    }
    .map(|id| (*id).clone())
}
