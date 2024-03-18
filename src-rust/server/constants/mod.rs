pub mod version;

/// Filestem of cover files used in cover_finder.rs
pub fn cover_filestems<'a>() -> Vec<&'a str> {
    vec!["cover", "cover", "_", "folder"]
}

pub fn native_img_formats<'a>() -> Vec<&'a str> {
    vec!["png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "webp"]
}

pub fn extended_img_formats<'a>() -> Vec<&'a str> {
    let mut formats = native_img_formats();
    formats.extend(vec!["jxl", "avif"]);
    formats
}

pub fn blurhash_dimension_cap() -> f32 {
    20.0
}

pub fn ratio_percision() -> u32 {
    1000
}
