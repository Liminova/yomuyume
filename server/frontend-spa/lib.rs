include!(concat!(env!("OUT_DIR"), "/spa.rs"));

pub struct Content {
    pub content: &'static [u8],
    pub mime_type: &'static str,
}

pub fn get_file(path: &str) -> Content {
    let content = if IMPLICIT_INDEX_HTML.contains(&path) {
        get_file_(format!("{}/index.html", path))
    } else {
        get_file_(path)
    };

    if let Some(content) = content {
        return Content {
            content,
            mime_type: path
                .split('.')
                .next_back()
                .and_then(get_mime_type)
                .unwrap_or("text/plain"),
        };
    }

    get_file_("404.html")
        .map(|content| Content {
            content,
            mime_type: "text/html",
        })
        .unwrap_or(Content {
            content: b"404",
            mime_type: "text/plain",
        })
}
