use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Rating(f32);

impl Eq for Rating {
    fn assert_receiver_is_total_eq(&self) {
        assert!(self.eq(self));
    }
}

impl FromStr for Rating {
    type Err = String;

    fn from_str(s: &str) -> Result<Rating, Self::Err> {
        let f = s.parse::<f32>().map_err(|e| e.to_string())?;
        if !(0.0..=5.0).contains(&f) {
            return Err(format!(
                "Rating must be between 0.0 and 5.0 (inclusive), got {f}"
            ));
        }
        Ok(Rating((f * 10.0).round() / 10.0))
    }
}

impl Rating {
    pub fn deserializer<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Rating>, D::Error> {
        let s = String::deserialize(deserializer)?.trim().to_string();
        if s.is_empty() {
            return Err(serde::de::Error::custom("empty string"));
        }
        Rating::from_str(s.as_str())
            .map_err(serde::de::Error::custom)
            .map(Some)
    }

    #[allow(clippy::ref_option)]
    pub fn serializer<S: Serializer>(
        rating: &Option<Rating>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match rating {
            Some(rating) => serializer.serialize_str(&format!("{:.1}", rating.0)),
            None => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::comic_info::{ComicInfo, rating::Rating};

    #[test]
    fn rating() {
        let xml = r"<ComicInfo><CommunityRating>0.0  </CommunityRating></ComicInfo>";
        let comic_info = ComicInfo::from_str(xml).unwrap();
        assert_eq!(comic_info.community_rating, Some(Rating(0.0)));
        assert_eq!(
            comic_info.to_pretty_string(false).unwrap(),
            format!("<ComicInfo>\n    <CommunityRating>0.0</CommunityRating>\n</ComicInfo>")
        );

        assert!(
            ComicInfo::from_str(r"<ComicInfo><CommunityRating>-1</CommunityRating></ComicInfo>")
                .is_err()
        );
        assert!(
            ComicInfo::from_str(r"<ComicInfo><CommunityRating>6.0</CommunityRating></ComicInfo>")
                .is_err()
        );
        assert!(
            ComicInfo::from_str(r"<ComicInfo><CommunityRating></CommunityRating></ComicInfo>")
                .is_err()
        );

        let comic_info = ComicInfo::from_str(
            r"<ComicInfo><CommunityRating>4.12  </CommunityRating></ComicInfo>",
        )
        .unwrap();
        assert_eq!(
            comic_info.to_pretty_string(false).unwrap(),
            format!("<ComicInfo>\n    <CommunityRating>4.1</CommunityRating>\n</ComicInfo>")
        );
    }
}
