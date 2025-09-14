use serde::{Deserialize, Deserializer, Serializer};

pub fn tags_deserializer<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    let mut tags: Vec<String> = String::deserialize(deserializer)?
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    tags.sort();
    tags.dedup();
    Ok(tags)
}

pub fn tags_serializer<S: Serializer>(tags: &[String], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&tags.join(", "))
}

#[cfg(test)]
mod tests {
    use crate::utils::comic_info::ComicInfo;

    #[test]
    fn tags() {
        let comic_info = ComicInfo::from_str(r"<ComicInfo><Tags>  </Tags></ComicInfo>").unwrap();
        assert_eq!(comic_info.tags, Vec::<String>::new());
        assert_eq!(
            comic_info.to_pretty_string(false).unwrap(),
            format!("<ComicInfo/>")
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><Tags>tag3  , tag1  , tag2  </Tags></ComicInfo>")
                .unwrap();
        assert_eq!(
            comic_info.tags,
            Vec::<String>::from(["tag1".into(), "tag2".into(), "tag3".into()])
        );
        assert_eq!(
            comic_info.to_pretty_string(false).unwrap(),
            format!("<ComicInfo>\n    <Tags>tag1, tag2, tag3</Tags>\n</ComicInfo>")
        );
    }
}
