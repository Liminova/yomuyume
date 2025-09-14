use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum YesNo {
    Yes,
    No,
    #[default]
    Unknown,
}

impl YesNo {
    pub fn deserializer<'de, D: Deserializer<'de>>(deserializer: D) -> Result<YesNo, D::Error> {
        match String::deserialize(deserializer)?.trim() {
            "Yes" => Ok(YesNo::Yes),
            "No" => Ok(YesNo::No),
            _ => Ok(YesNo::Unknown),
        }
    }

    pub const fn is_unknown(yes_no: &YesNo) -> bool {
        matches!(yes_no, YesNo::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::comic_info::{ComicInfo, yes_no::YesNo};

    #[test]
    fn yesno() {
        macro_rules! assert_yes_no {
            ($xml:expr, $expected:expr, $expected_xml:expr) => {
                let mut comic_info = ComicInfo::from_str($xml).unwrap();
                assert_eq!(comic_info.black_and_white, $expected);
                comic_info.black_and_white = $expected;
                assert_eq!(
                    comic_info.to_pretty_string(false).unwrap(),
                    format!($expected_xml)
                );
            };
        }

        assert_yes_no!(
            "<ComicInfo><BlackAndWhite>  </BlackAndWhite></ComicInfo>",
            YesNo::Unknown,
            "<ComicInfo/>"
        );
        assert_yes_no!("<ComicInfo></ComicInfo>", YesNo::Unknown, "<ComicInfo/>");
        assert_yes_no!(
            "<ComicInfo><BlackAndWhite>Unknown  </BlackAndWhite></ComicInfo>",
            YesNo::Unknown,
            "<ComicInfo/>"
        );
        assert_yes_no!(
            "<ComicInfo><BlackAndWhite>Yes  </BlackAndWhite></ComicInfo>",
            YesNo::Yes,
            "<ComicInfo>\n    <BlackAndWhite>Yes</BlackAndWhite>\n</ComicInfo>"
        );
        assert_yes_no!(
            "<ComicInfo><BlackAndWhite>No  </BlackAndWhite></ComicInfo>",
            YesNo::No,
            "<ComicInfo>\n    <BlackAndWhite>No</BlackAndWhite>\n</ComicInfo>"
        );
    }
}
