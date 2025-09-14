use serde::{Deserialize, Deserializer, Serialize};

use crate::utils::comic_info::integer_skip_condition::{
    int32_is_neg_one, int32_neg_one, int64_is_zero,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ArrayOfComicPageInfo {
    #[serde(rename = "Page", default, skip_serializing_if = "Vec::is_empty")]
    pub pages_: Vec<ComicPageInfo>,
}

impl ArrayOfComicPageInfo {
    pub const fn is_empty(&self) -> bool {
        self.pages_.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub enum ComicPageType {
    FrontCover,
    InnerCover,
    Roundup,
    #[default]
    Story,
    Advertisement,
    Editorial,
    Letters,
    Preview,
    BackCover,
    Other,
    Deleted,
}

impl ComicPageType {
    pub const fn is_story(&self) -> bool {
        matches!(self, ComicPageType::Story)
    }

    pub fn deserializer<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<ComicPageType, D::Error> {
        match String::deserialize(deserializer)?.trim() {
            "FrontCover" => Ok(ComicPageType::FrontCover),
            "InnerCover" => Ok(ComicPageType::InnerCover),
            "Roundup" => Ok(ComicPageType::Roundup),
            "Advertisement" => Ok(ComicPageType::Advertisement),
            "Editorial" => Ok(ComicPageType::Editorial),
            "Letters" => Ok(ComicPageType::Letters),
            "Preview" => Ok(ComicPageType::Preview),
            "BackCover" => Ok(ComicPageType::BackCover),
            "Other" => Ok(ComicPageType::Other),
            "Deleted" => Ok(ComicPageType::Deleted),
            _ => Ok(ComicPageType::Story),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ComicPageInfo {
    #[serde(rename = "@Image", default)]
    pub image: i32,
    #[serde(
        rename = "@Type",
        default,
        skip_serializing_if = "ComicPageType::is_story",
        deserialize_with = "ComicPageType::deserializer"
    )]
    pub page_type: ComicPageType,
    #[serde(
        rename = "@DoublePage",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub double_page: bool,
    #[serde(rename = "@ImageSize", default, skip_serializing_if = "int64_is_zero")]
    pub image_size: i64,
    #[serde(rename = "@Key", default, skip_serializing_if = "String::is_empty")]
    pub key: String,
    #[serde(
        rename = "@Bookmark",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub bookmark: String,
    #[serde(
        rename = "@ImageWidth",
        default = "int32_neg_one",
        skip_serializing_if = "int32_is_neg_one"
    )]
    pub image_width: i32,
    #[serde(
        rename = "@ImageHeight",
        default = "int32_neg_one",
        skip_serializing_if = "int32_is_neg_one"
    )]
    pub image_height: i32,
}

#[cfg(test)]
mod tests {
    use crate::utils::comic_info::{
        ComicInfo,
        comic_page_info::{ArrayOfComicPageInfo, ComicPageInfo, ComicPageType},
    };

    #[test]
    fn pages() {
        macro_rules! assert_pages {
            ($xml:expr, $expected:expr, $expected_xml:expr) => {
                let mut comic_info = ComicInfo::from_str($xml).unwrap();
                assert_eq!(comic_info.pages(), $expected);
                comic_info.pages_ = ArrayOfComicPageInfo {
                    pages_: $expected.clone(),
                };
                assert_eq!(
                    comic_info.to_pretty_string(false).unwrap(),
                    format!($expected_xml)
                );
            };
        }

        assert_pages!(
            r"<ComicInfo></ComicInfo>",
            &Vec::<ComicPageInfo>::new(),
            "<ComicInfo/>"
        );
        assert_pages!(
            r#"<ComicInfo>
                <Pages>
                    <Page Image="0" Type="Story  " DoublePage="false" ImageSize="0" Key="" Bookmark="" ImageWidth="-1" ImageHeight="-1" ImagePath="" Description=""/>
                </Pages>
            </ComicInfo>"#,
            &vec![ComicPageInfo {
                image: 0,
                page_type: ComicPageType::Story,
                double_page: false,
                image_size: 0,
                key: String::new(),
                bookmark: String::new(),
                image_width: -1,
                image_height: -1
            }],
            "<ComicInfo>\n    <Pages>\n        <Page Image=\"0\"/>\n    </Pages>\n</ComicInfo>"
        );
        assert_pages!(
            r#"<ComicInfo>
                <Pages>
                    <Page Image="0" Type="FrontCover" DoublePage="false" ImageSize="0" Key="1" Bookmark="2" ImageWidth="3" ImageHeight="4" ImagePath="5" Description="6"/>
                    <Page Image="1" Type="Story  " DoublePage="true" ImageSize="12345" Key="" Bookmark="" ImageWidth="-1" ImageHeight="-1" ImagePath="" Description=""/>
                </Pages>
            </ComicInfo>"#,
            &vec![
                ComicPageInfo {
                    image: 0,
                    page_type: ComicPageType::FrontCover,
                    double_page: false,
                    image_size: 0,
                    key: "1".to_string(),
                    bookmark: "2".to_string(),
                    image_width: 3,
                    image_height: 4
                },
                ComicPageInfo {
                    image: 1,
                    page_type: ComicPageType::Story,
                    double_page: true,
                    image_size: 12345,
                    key: String::new(),
                    bookmark: String::new(),
                    image_width: -1,
                    image_height: -1
                }
            ],
            "<ComicInfo>\n    <Pages>\n        <Page Image=\"0\" Type=\"FrontCover\" Key=\"1\" Bookmark=\"2\" ImageWidth=\"3\" ImageHeight=\"4\"/>\n        <Page Image=\"1\" DoublePage=\"true\" ImageSize=\"12345\"/>\n    </Pages>\n</ComicInfo>"
        );
    }
}
