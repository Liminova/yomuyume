use crate::utils::constants::SUPPORTED_IMAGE_FORMATS;

pub trait StringUtils {
    fn has_image_ext(&self) -> bool;
}

impl StringUtils for String {
    /// Check if the string has an image extension
    fn has_image_ext(&self) -> bool {
        self.split('.')
            .next_back()
            .is_some_and(|ext| SUPPORTED_IMAGE_FORMATS.contains(&ext))
    }
}
