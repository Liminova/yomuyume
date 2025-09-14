use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub enum AgeRating {
    #[default]
    Unknown,
    #[serde(rename = "Adults Only 18+")]
    AdultsOnly18,
    #[serde(rename = "Early Childhood")]
    EarlyChildhood,
    Everyone,
    #[serde(rename = "Everyone 10+")]
    Everyone10,
    G,
    #[serde(rename = "Kids to Adults")]
    KidsToAdults,
    M,
    #[serde(rename = "MA15+")]
    MA15,
    #[serde(rename = "Mature 17+")]
    Mature17,
    PG,
    #[serde(rename = "R18+")]
    R18,
    #[serde(rename = "Rating Pending")]
    RatingPending,
    Teen,
    #[serde(rename = "X18+")]
    X18,
}

impl AgeRating {
    pub const fn is_unknown(age_rating: &AgeRating) -> bool {
        matches!(age_rating, AgeRating::Unknown)
    }

    pub fn deserializer<'de, D: Deserializer<'de>>(deserializer: D) -> Result<AgeRating, D::Error> {
        match String::deserialize(deserializer)?.trim() {
            "Adults Only 18+" => Ok(AgeRating::AdultsOnly18),
            "Early Childhood" => Ok(AgeRating::EarlyChildhood),
            "Everyone" => Ok(AgeRating::Everyone),
            "Everyone 10+" => Ok(AgeRating::Everyone10),
            "G" => Ok(AgeRating::G),
            "Kids to Adults" => Ok(AgeRating::KidsToAdults),
            "M" => Ok(AgeRating::M),
            "MA15+" => Ok(AgeRating::MA15),
            "Mature 17+" => Ok(AgeRating::Mature17),
            "PG" => Ok(AgeRating::PG),
            "R18+" => Ok(AgeRating::R18),
            "Rating Pending" => Ok(AgeRating::RatingPending),
            "Teen" => Ok(AgeRating::Teen),
            "X18+" => Ok(AgeRating::X18),
            _ => Ok(AgeRating::Unknown),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::comic_info::{ComicInfo, age_rating::AgeRating};

    #[test]
    fn age_rating() {
        macro_rules! assert_age_rating {
            ($xml:expr, $expected:expr, $expected_xml:expr) => {
                let mut comic_info = ComicInfo::from_str($xml).unwrap();
                assert_eq!(comic_info.age_rating, $expected);
                comic_info.age_rating = $expected;
                assert_eq!(
                    comic_info.to_pretty_string(false).unwrap(),
                    format!($expected_xml)
                );
            };
        }

        assert_age_rating!(
            "<ComicInfo><AgeRating>  </AgeRating></ComicInfo>",
            AgeRating::Unknown,
            "<ComicInfo/>"
        );
        assert_age_rating!(
            "<ComicInfo></ComicInfo>",
            AgeRating::Unknown,
            "<ComicInfo/>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>Unknown  </AgeRating></ComicInfo>",
            AgeRating::Unknown,
            "<ComicInfo/>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>Adults Only 18+  </AgeRating></ComicInfo>",
            AgeRating::AdultsOnly18,
            "<ComicInfo>\n    <AgeRating>Adults Only 18+</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>Early Childhood  </AgeRating></ComicInfo>",
            AgeRating::EarlyChildhood,
            "<ComicInfo>\n    <AgeRating>Early Childhood</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>Everyone  </AgeRating></ComicInfo>",
            AgeRating::Everyone,
            "<ComicInfo>\n    <AgeRating>Everyone</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>Everyone 10+  </AgeRating></ComicInfo>",
            AgeRating::Everyone10,
            "<ComicInfo>\n    <AgeRating>Everyone 10+</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>G  </AgeRating></ComicInfo>",
            AgeRating::G,
            "<ComicInfo>\n    <AgeRating>G</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>Kids to Adults  </AgeRating></ComicInfo>",
            AgeRating::KidsToAdults,
            "<ComicInfo>\n    <AgeRating>Kids to Adults</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>M  </AgeRating></ComicInfo>",
            AgeRating::M,
            "<ComicInfo>\n    <AgeRating>M</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>MA15+  </AgeRating></ComicInfo>",
            AgeRating::MA15,
            "<ComicInfo>\n    <AgeRating>MA15+</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>Mature 17+  </AgeRating></ComicInfo>",
            AgeRating::Mature17,
            "<ComicInfo>\n    <AgeRating>Mature 17+</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>PG  </AgeRating></ComicInfo>",
            AgeRating::PG,
            "<ComicInfo>\n    <AgeRating>PG</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>R18+  </AgeRating></ComicInfo>",
            AgeRating::R18,
            "<ComicInfo>\n    <AgeRating>R18+</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>Rating Pending  </AgeRating></ComicInfo>",
            AgeRating::RatingPending,
            "<ComicInfo>\n    <AgeRating>Rating Pending</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>Teen  </AgeRating></ComicInfo>",
            AgeRating::Teen,
            "<ComicInfo>\n    <AgeRating>Teen</AgeRating>\n</ComicInfo>"
        );
        assert_age_rating!(
            "<ComicInfo><AgeRating>X18+  </AgeRating></ComicInfo>",
            AgeRating::X18,
            "<ComicInfo>\n    <AgeRating>X18+</AgeRating>\n</ComicInfo>"
        );
    }
}
