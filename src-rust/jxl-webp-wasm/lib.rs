use image::DynamicImage;
use jxl_oxide::integration::JxlDecoder;
use std::io::Cursor;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn decode(buf: Vec<u8>) -> Vec<u8> {
    console_error_panic_hook::set_once();

    let decoder = JxlDecoder::new(std::io::Cursor::new(buf)).unwrap();
    let img = DynamicImage::from_decoder(decoder).unwrap();

    let mut out = Cursor::new(Vec::new());

    match img {
        DynamicImage::ImageRgb8(img) => img,
        _ => img.to_rgb8(),
    }
    .write_to(&mut out, image::ImageFormat::WebP)
    .unwrap();

    out.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() {
        decode(include_bytes!("sunset_logo.jxl").to_vec());
    }
}
