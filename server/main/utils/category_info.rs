use serde::{Deserialize, Serialize};

use crate::utils::comic_info::option_string_deserializer;

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct CategoryInfo {
    #[serde(
        rename = "Name",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<String>,
    #[serde(
        rename = "Description",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
}

impl CategoryInfo {
    pub fn from_str(s: &str) -> Result<CategoryInfo, quick_xml::DeError> {
        if s.is_empty() {
            return Ok(CategoryInfo::default());
        }
        quick_xml::de::from_str::<CategoryInfo>(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_filled() {
        let category_info: CategoryInfo = CategoryInfo::from_str(
            r#"<CategoryInfo>
                <Name>  Adventure    </Name>
                <Description>  Lorem Ipsum</Description>
            </CategoryInfo>"#,
        )
        .unwrap();

        assert_eq!(category_info.name, Some("Adventure".to_string()));
        assert_eq!(category_info.description, Some("Lorem Ipsum".to_string()));
    }

    #[test]
    fn no_fields() {
        let category_info: CategoryInfo =
            quick_xml::de::from_str::<CategoryInfo>(r"<CategoryInfo></CategoryInfo>").unwrap();

        assert_eq!(category_info.name, None);
        assert_eq!(category_info.description, None);
    }

    #[test]
    fn all_empty() {
        let category_info: CategoryInfo =
            CategoryInfo::from_str(r"<CategoryInfo><Name/><Description/><Cover/></CategoryInfo>")
                .unwrap();

        assert_eq!(category_info.name, None);
        assert_eq!(category_info.description, None);
    }
}
