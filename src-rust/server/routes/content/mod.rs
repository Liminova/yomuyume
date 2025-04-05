mod get_categories;
mod get_pages;
mod get_search;
mod get_tags;
mod get_title;
mod structs;

pub use get_categories::*;
pub use get_pages::*;
pub use get_search::*;
pub use get_tags::*;
pub use get_title::*;
use serde_json::Value;
pub use structs::*;

type TagID = String;
type TagName = String;
fn parse_tags(vals: Vec<Value>) -> Option<Vec<(TagID, TagName)>> {
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
