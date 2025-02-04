mod get_categories;
mod get_chapter;
mod get_oneshot;
mod get_series;
mod get_tags;
mod post_search;
mod structs;

pub use get_categories::*;
pub use get_chapter::*;
pub use get_oneshot::*;
pub use get_series::*;
pub use get_tags::*;
pub use post_search::*;
use serde_json::Value;
pub use structs::*;

fn parse_tags(vals: Vec<Value>) -> Option<Vec<(String, String)>> {
    let mut tags = vals
        .into_iter()
        .filter_map(|tag| {
            let mut pair = tag.as_array()?.iter();
            Some((
                pair.next()?.as_i64()?.to_string(),
                pair.next()?.as_str()?.to_string(),
            ))
        })
        .collect::<Vec<_>>();

    (!tags.is_empty()).then(|| {
        tags.sort_by(|a, b| a.1.cmp(&b.1));
        tags.dedup();
        tags
    })
}

fn is_jxl(p: impl AsRef<str>) -> bool {
    std::path::Path::new(p.as_ref())
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("jxl"))
}

fn parse_pages(vals: Vec<Value>) -> Vec<BasePageResponse> {
    let mut pages: Vec<(String, BasePageResponse)> = vals
        .into_iter()
        .filter_map(|r| {
            let page = r.as_object()?;
            let path = page.get("path")?.as_str()?;

            Some((
                path.to_string(),
                BasePageResponse {
                    id: page.get("id")?.as_i64()?.to_string(),
                    blurhash: None,
                    width: None,
                    height: None,
                    jxl: is_jxl(path),
                    description: page
                        .get("description")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string()),
                },
            ))
        })
        .collect();

    pages.sort_by(|a, b| a.0.cmp(&b.0));
    pages.dedup_by(|a, b| a.0 == b.0);

    pages.into_iter().map(|(_, page)| page).collect()
}
