use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub enum Manga {
    #[default]
    Unknown,
    No,
    Yes,
    YesAndRightToLeft,
}

impl Manga {
    pub const fn is_unknown(manga: &Manga) -> bool {
        matches!(manga, Manga::Unknown)
    }

    pub fn deserializer<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Manga, D::Error> {
        match String::deserialize(deserializer)?.trim() {
            "Yes" => Ok(Manga::Yes),
            "No" => Ok(Manga::No),
            "YesAndRightToLeft" => Ok(Manga::YesAndRightToLeft),
            _ => Ok(Manga::Unknown),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::comic_info::{ComicInfo, manga::Manga};

    #[test]
    fn manga() {
        macro_rules! assert_manga {
            ($xml:expr, $expected:expr, $expected_xml:expr) => {
                let mut comic_info = ComicInfo::from_str($xml).unwrap();
                assert_eq!(comic_info.manga, $expected);
                comic_info.manga = $expected;
                assert_eq!(
                    comic_info.to_pretty_string(false).unwrap(),
                    format!($expected_xml)
                );
            };
        }
        assert_manga!(
            "<ComicInfo><Manga>  </Manga></ComicInfo>",
            Manga::Unknown,
            "<ComicInfo/>"
        );
        assert_manga!("<ComicInfo></ComicInfo>", Manga::Unknown, "<ComicInfo/>");
        assert_manga!(
            "<ComicInfo><Manga>Unknown  </Manga></ComicInfo>",
            Manga::Unknown,
            "<ComicInfo/>"
        );
        assert_manga!(
            "<ComicInfo><Manga>Yes  </Manga></ComicInfo>",
            Manga::Yes,
            "<ComicInfo>\n    <Manga>Yes</Manga>\n</ComicInfo>"
        );
        assert_manga!(
            "<ComicInfo><Manga>No  </Manga></ComicInfo>",
            Manga::No,
            "<ComicInfo>\n    <Manga>No</Manga>\n</ComicInfo>"
        );
        assert_manga!(
            "<ComicInfo><Manga>YesAndRightToLeft  </Manga></ComicInfo>",
            Manga::YesAndRightToLeft,
            "<ComicInfo>\n    <Manga>YesAndRightToLeft</Manga>\n</ComicInfo>"
        );
    }
}
