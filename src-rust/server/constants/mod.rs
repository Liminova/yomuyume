pub mod version;

/// Filestem of thumbnail files used in thumbnail_finder.rs
pub fn thumbnail_filestems<'a>() -> Vec<&'a str> {
    vec!["thumbnail", "cover", "_", "folder"]
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
