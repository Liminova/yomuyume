use std::{path::Path, sync::Arc};

use chrono::TimeZone;
use tantivy::TantivyDocument;
use tracing::error;

use crate::{
    AppState,
    utils::{comic_info::ComicInfo, result_utils::ResultUtils},
};

pub async fn comic_info_to_tantivy(
    app_state: &Arc<AppState>,
    title_identity_path: &str,
    category_identity_path: Option<&str>,
    comic_info: Option<&ComicInfo>,
    title_path: &Path,
) {
    let mut doc = TantivyDocument::default();
    doc.add_text(
        app_state.indexer.fields.title,
        comic_info
            .and_then(|ci| ci.title.as_ref())
            .map(|t| t.as_str())
            .unwrap_or(title_identity_path),
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
    doc.add_text(app_state.indexer.fields.identity_path, &title_identity_path);
    if let Some(category_identity_path) = category_identity_path.as_ref() {
        doc.add_text(
            app_state.indexer.fields.category_identity_path,
            category_identity_path,
        );
    }
    if let Some(release_date) = comic_info
        .map(|ci| (ci.year, ci.month as u32, ci.day as u32))
        .and_then(|(year, month, day)| {
            chrono::NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        })
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
            )
        });
}
