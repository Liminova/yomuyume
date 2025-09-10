// Reference: https://mimetype.io/all-types

pub fn mimatcher(file_extension: &str) -> Option<&'static str> {
    match file_extension {
        "html" => Some("text/html"),
        "css" => Some("text/css"),
        "js" => Some("application/javascript"),
        "json" => Some("application/json"),
        "ico" => Some("image/x-icon"),
        "wasm" => Some("application/wasm"),
        "woff2" => Some("font/woff2"),
        "ttf" => Some("font/ttf"),
        "xml" => Some("application/xml"),
        "svg" => Some("image/svg+xml"),

        "zip" => Some("application/zip"),
        "7z" => Some("application/x-7z-compressed"),
        "rar" => Some("application/vnd.rar"),
        "cbz" => Some("application/x-cbr"),
        "cbr" => Some("application/x-cbr"),

        "avif" => Some("image/avif"),
        "bmp" => Some("image/bmp"),
        "gif" => Some("image/gif"),
        "jpg" => Some("image/jpeg"),
        "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "tif" => Some("image/tiff"),
        "tiff" => Some("image/tiff"),
        "webp" => Some("image/webp"),

        _ => None,
    }
}
