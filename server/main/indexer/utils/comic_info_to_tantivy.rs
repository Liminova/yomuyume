use std::{path::Path, sync::Arc};

use chrono::TimeZone;
use tantivy::TantivyDocument;
use tracing::{error, warn};

use crate::{
    AppState,
    utils::{comic_info::ComicInfo, constants::COMIC_INFO, result_utils::ResultUtils},
};

pub async fn comic_info_to_tantivy(
    app_state: &Arc<AppState>,
    comic_info: Option<&ComicInfo>,
    title_id: &str,
    title_path: &Path,
) {
    let mut doc = TantivyDocument::default();
    doc.add_text(
        app_state.indexer.fields.title,
        comic_info.and_then(|ci| ci.title.as_ref()).map_or(
            title_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled"),
            |t| t.as_str(),
        ),
    );
    if let Some(author) = comic_info.and_then(|ci| ci.writer.as_ref()) {
        doc.add_text(app_state.indexer.fields.author, author);
    }
    if let Some(desc) = comic_info.and_then(|ci| ci.summary.as_ref()) {
        doc.add_text(app_state.indexer.fields.description, desc);
    }
    if let Some(tags) = comic_info.map(|ci| &ci.tags) {
        for tag in tags {
            doc.add_text(app_state.indexer.fields.tag, tag);
        }
    }
    doc.add_text(app_state.indexer.fields.id, title_id);
    if let Some(release_date) = comic_info
        .and_then(|ci| {
            Some((
                ci.year,
                u32::try_from(if ci.month == -1 { 1 } else { ci.month }).okay(|e| {
                    warn!(
                        "invalid month {} for {COMIC_INFO} in {}: {e}",
                        ci.month,
                        title_path.display()
                    );
                })?,
                u32::try_from(if ci.day == -1 { 1 } else { ci.day }).okay(|e| {
                    warn!(
                        "invalid day {} for {COMIC_INFO} in {}: {e}",
                        ci.day,
                        title_path.display()
                    );
                })?,
            ))
        })
        .and_then(|(year, month, day)| chrono::NaiveDate::from_ymd_opt(year, month, day))
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|d| chrono::Utc.from_utc_datetime(&d).timestamp())
        .map(tantivy::time::OffsetDateTime::from_unix_timestamp)
        .transpose()
        .ok()
        .flatten()
        .map(tantivy::DateTime::from_utc)
    {
        doc.add_date(app_state.indexer.fields.release_date, release_date);
    }

    app_state
        .indexer
        .writer
        .lock()
        .await
        .add_document(doc)
        .okay(|e| {
            error!(
                "can't add title {} to tantivy index: {e:?}",
                title_path.display()
            );
        });
}
