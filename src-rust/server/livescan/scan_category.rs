use async_recursion::async_recursion;
use std::path::PathBuf;

pub struct ScannedTitle {
    pub path: PathBuf,
    pub name: String,
}

impl ScannedTitle {
    pub fn path_lossy(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

/// Scanning all title inside a category
#[async_recursion]
pub async fn scan_category(item_dir: &PathBuf) -> Vec<ScannedTitle> {
    let mut files = Vec::new();
    let mut entries = match tokio::fs::read_dir(item_dir).await {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!("error reading item file: {}\n", e);
            return Vec::new();
        }
    };

    'next_title: while let Some(entry) = entries.next_entry().await.unwrap_or_default() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(scan_category(&path).await);
        }
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if ext == "zip" {
            match path {
                p if p.to_str().unwrap_or_default().is_empty() => continue 'next_title,
                p => files.push(ScannedTitle {
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
    files
}
