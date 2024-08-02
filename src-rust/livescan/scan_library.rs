use std::path::PathBuf;
use tracing::warn;

#[derive(Debug)]
pub struct ScannedCategory {
    pub path: PathBuf,
    pub name: String,
}

/// Scanning all categories dirs inside library
pub async fn scan_library(library_path: &str) -> Vec<ScannedCategory> {
    let mut categories = Vec::new();
    let mut entries = match tokio::fs::read_dir(library_path).await {
        Ok(entries) => entries,
        Err(e) => {
            warn!("rrror reading category dir: {}\n", e);
            return Vec::new();
        }
    };
    'next_category: while let Some(entry) = entries.next_entry().await.unwrap_or_default() {
        let path = entry.path();
        if path.is_dir() {
            match path {
                p if p.to_str().unwrap_or_default().is_empty() => continue 'next_category,
                p => categories.push(ScannedCategory {
                    path: p.clone(),
                    name: p
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .to_string(),
                }),
            }
        }
    }
    categories
}
