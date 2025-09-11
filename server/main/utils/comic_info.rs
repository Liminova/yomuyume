//! ComicInfo Version 2.1
//!
//! Schema: https://github.com/anansi-project/comicinfo/blob/0b6e01/drafts/v2.1/ComicInfo.xsd

use std::str::FromStr;

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::skip_serializing_none;

use crate::utils::constants::COMICINFO_SCHEMA;

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ComicInfo {
    #[serde(
        rename = "Title",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub title: Option<String>,
    #[serde(
        rename = "Series",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub series: Option<String>,
    #[serde(
        rename = "Number",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub number: Option<String>,
    #[serde(
        rename = "Count",
        default = "int32_neg_one",
        skip_serializing_if = "int32_is_neg_one"
    )]
    pub count: i32,
    #[serde(
        rename = "Volume",
        default = "int32_neg_one",
        skip_serializing_if = "int32_is_neg_one"
    )]
    pub volume: i32,
    #[serde(
        rename = "AlternateSeries",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub alternate_series: Option<String>,
    #[serde(
        rename = "AlternateNumber",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub alternate_number: Option<String>,
    #[serde(
        rename = "AlternateCount",
        default = "int32_neg_one",
        skip_serializing_if = "int32_is_neg_one"
    )]
    pub alternate_count: i32,
    #[serde(
        rename = "Summary",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub summary: Option<String>,
    #[serde(
        rename = "Notes",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub notes: Option<String>,
    #[serde(
        rename = "Year",
        default = "int32_neg_one",
        skip_serializing_if = "int32_is_neg_one"
    )]
    pub year: i32,
    #[serde(
        rename = "Month",
        default = "int32_neg_one",
        skip_serializing_if = "int32_is_neg_one"
    )]
    pub month: i32,
    #[serde(
        rename = "Day",
        default = "int32_neg_one",
        skip_serializing_if = "int32_is_neg_one"
    )]
    pub day: i32,
    #[serde(
        rename = "Writer",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub writer: Option<String>,
    #[serde(
        rename = "Penciller",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub penciller: Option<String>,
    #[serde(
        rename = "Inker",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub inker: Option<String>,
    #[serde(
        rename = "Colorist",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub colorist: Option<String>,
    #[serde(
        rename = "Letterer",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub letterer: Option<String>,
    #[serde(
        rename = "CoverArtist",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub cover_artist: Option<String>,
    #[serde(
        rename = "Editor",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub editor: Option<String>,
    #[serde(
        rename = "Translator",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub translator: Option<String>,
    #[serde(
        rename = "Publisher",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub publisher: Option<String>,
    #[serde(
        rename = "Imprint",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub imprint: Option<String>,
    #[serde(
        rename = "Genre",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub genre: Option<String>,
    #[serde(
        rename = "Tags",
        default,
        deserialize_with = "tags_deserializer",
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "tags_serializer"
    )]
    pub tags: Vec<String>,
    #[serde(
        rename = "Web",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub web: Option<String>,
    #[serde(rename = "PageCount", default, skip_serializing_if = "int32_is_zero")]
    pub page_count: i32,
    #[serde(
        rename = "LanguageISO",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub language_iso: Option<String>,
    #[serde(
        rename = "Format",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub format: Option<String>,
    #[serde(
        rename = "BlackAndWhite",
        default,
        deserialize_with = "YesNo::deserializer",
        skip_serializing_if = "YesNo::is_unknown"
    )]
    pub black_and_white: YesNo,
    #[serde(rename = "Manga", default, skip_serializing_if = "Manga::is_unknown")]
    pub manga: Manga,
    #[serde(
        rename = "Characters",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub characters: Option<String>,
    #[serde(
        rename = "Teams",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub teams: Option<String>,
    #[serde(
        rename = "Locations",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub locations: Option<String>,
    #[serde(
        rename = "ScanInformation",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub scan_information: Option<String>,
    #[serde(
        rename = "StoryArc",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub story_arc: Option<String>,
    #[serde(
        rename = "StoryArcNumber",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub story_arc_number: Option<String>,
    #[serde(
        rename = "SeriesGroup",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub series_group: Option<String>,
    #[serde(
        rename = "AgeRating",
        default,
        skip_serializing_if = "AgeRating::is_unknown"
    )]
    pub age_rating: AgeRating,
    #[serde(
        rename = "Pages",
        default,
        skip_serializing_if = "ArrayOfComicPageInfo::is_empty"
    )]
    pages_: ArrayOfComicPageInfo,
    #[serde(
        rename = "CommunityRating",
        default,
        serialize_with = "Rating::serializer",
        deserialize_with = "Rating::deserializer"
    )]
    pub community_rating: Option<Rating>,
    #[serde(
        rename = "MainCharacterOrTeam",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub main_character_or_team: Option<String>,
    #[serde(
        rename = "Review",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub review: Option<String>,
    #[serde(
        rename = "GTIN",
        default,
        deserialize_with = "option_string_deserializer"
    )]
    pub gtin: Option<String>,
}

impl Default for ComicInfo {
    fn default() -> ComicInfo {
        ComicInfo {
            title: None,
            series: None,
            number: None,
            count: -1,
            volume: -1,
            alternate_series: None,
            alternate_number: None,
            alternate_count: -1,
            summary: None,
            notes: None,
            year: -1,
            month: -1,
            day: -1,
            writer: None,
            penciller: None,
            inker: None,
            colorist: None,
            letterer: None,
            cover_artist: None,
            editor: None,
            translator: None,
            publisher: None,
            imprint: None,
            genre: None,
            tags: vec![],
            web: None,
            page_count: 0,
            language_iso: None,
            format: None,
            black_and_white: YesNo::Unknown,
            manga: Manga::Unknown,
            characters: None,
            teams: None,
            locations: None,
            scan_information: None,
            story_arc: None,
            story_arc_number: None,
            series_group: None,
            age_rating: AgeRating::Unknown,
            pages_: ArrayOfComicPageInfo { pages_: vec![] },
            community_rating: None,
            main_character_or_team: None,
            review: None,
            gtin: None,
        }
    }
}

fn option_string_deserializer<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let s = String::deserialize(deserializer)?.trim().to_string();
    Ok(s.is_empty().then_some(s))
}

fn tags_deserializer<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let mut tags: Vec<String> = String::deserialize(deserializer)?
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    tags.sort();
    tags.dedup();
    Ok(tags)
}

fn tags_serializer<S: Serializer>(tags: &[String], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&tags.join(", "))
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum YesNo {
    Yes,
    No,
    #[default]
    Unknown,
}

impl YesNo {
    fn deserializer<'de, D: Deserializer<'de>>(deserializer: D) -> Result<YesNo, D::Error> {
        let s = String::deserialize(deserializer)?.trim().to_string();
        if s.is_empty() {
            return Err(serde::de::Error::custom("empty string"));
        }
        match s.as_str() {
            "Yes" => Ok(YesNo::Yes),
            "No" => Ok(YesNo::No),
            "Unknown" => Ok(YesNo::Unknown),
            _ => Err(serde::de::Error::custom("invalid value for YesNo")),
        }
    }

    fn is_unknown(yes_no: &YesNo) -> bool {
        matches!(yes_no, YesNo::Unknown)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Manga {
    #[default]
    Unknown,
    No,
    Yes,
    YesAndRightToLeft,
}

impl Manga {
    fn is_unknown(manga: &Manga) -> bool {
        matches!(manga, Manga::Unknown)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
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
    fn is_unknown(age_rating: &AgeRating) -> bool {
        matches!(age_rating, AgeRating::Unknown)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ArrayOfComicPageInfo {
    #[serde(rename = "Page", default, skip_serializing_if = "Vec::is_empty")]
    pages_: Vec<ComicPageInfo>,
}

impl ArrayOfComicPageInfo {
    fn is_empty(&self) -> bool {
        self.pages_.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ComicPageInfo {
    #[serde(rename = "@Image", default)]
    pub image: i32,
    #[serde(
        rename = "@Type",
        default,
        skip_serializing_if = "ComicPageType::is_story"
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
    fn deserializer<'de, D: Deserializer<'de>>(
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

    fn serializer<S: Serializer>(
        rating: &Option<Rating>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match rating {
            Some(rating) => serializer.serialize_str(&format!("{:.1}", rating.0)),
            None => serializer.serialize_none(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
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
    fn is_story(&self) -> bool {
        matches!(self, ComicPageType::Story)
    }
}

const fn int32_neg_one() -> i32 {
    -1
}

const fn int32_is_neg_one(i: &i32) -> bool {
    *i == -1
}

const fn int32_is_zero(i: &i32) -> bool {
    *i == 0
}

const fn int64_is_zero(i: &i64) -> bool {
    *i == 0
}

impl ComicInfo {
    pub fn pages(&self) -> &Vec<ComicPageInfo> {
        &self.pages_.pages_
    }

    pub fn pages_mut(&mut self) -> &mut Vec<ComicPageInfo> {
        &mut self.pages_.pages_
    }

    /// Get the release date of the comic.
    pub fn get_release(&self) -> Option<NaiveDate> {
        DateTime::parse_from_str(
            format!("{}-{}-{}", self.year, self.month, self.day).as_str(),
            "%Y-%m-%d",
        )
        .ok()
        .map(|d| d.with_timezone(&Utc))
        .and_then(|d| NaiveDate::from_ymd_opt(d.year(), d.month(), d.day()))
    }

    /// Get the description of a page file given its file name.
    pub fn get_page_description(&self, _page_file_name: &str) -> Option<String> {
        // let with_ext = page_file_name.to_string();

        // let no_ext = PathBuf::from(page_file_name)
        //     .file_stem()
        //     .map(|s| s.to_string_lossy().to_string())
        //     .unwrap_or_default();

        // self.pages().iter().find_map(|page| {
        //     let image_path = page.image_path.as_ref()?;

        //     let matched_with_ext = image_path == &with_ext;
        //     let matched_no_ext = image_path == &no_ext;

        //     if matched_with_ext || matched_no_ext {
        //         return page.description.clone();
        //     }

        //     None
        // })

        None
    }

    /// Parse a ComicInfo.xml string and return a [`ComicInfo`] object.
    ///
    /// If the string is empty, return a default one.
    pub fn from_str(s: &str) -> Result<ComicInfo, quick_xml::DeError> {
        if s.is_empty() {
            return Ok(ComicInfo::default());
        }
        quick_xml::de::from_str(s)
    }

    pub fn to_pretty_string(&self) -> Result<String, quick_xml::errors::serialize::SeError> {
        let mut buffer = format!("{COMICINFO_SCHEMA}\n");
        let mut ser = quick_xml::se::Serializer::new(&mut buffer);
        ser.indent(' ', 4);
        self.serialize(ser)?;
        Ok(buffer)
    }
}

impl TryFrom<&str> for ComicInfo {
    type Error = quick_xml::DeError;

    fn try_from(value: &str) -> Result<ComicInfo, Self::Error> {
        quick_xml::de::from_str(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_string_some() {
        let xml = r"<ComicInfo>
                <Series>  series  </Series>
                <Number>  number  </Number>
                <AlternateSeries>  alternate series  </AlternateSeries>
                <AlternateNumber>  alternate number  </AlternateNumber>
                <Summary>  summary  </Summary>
                <Notes>  notes  </Notes>
                <Writer>  writer  </Writer>
                <Penciller>  penciller  </Penciller>
                <Inker>  inker  </Inker>
                <Colorist>  colorist  </Colorist>
                <Letterer>  letterer  </Letterer>
                <CoverArtist>  cover artist  </CoverArtist>
                <Editor>  editor  </Editor>
                <Translator>  translator  </Translator>
                <Publisher>  publisher  </Publisher>
                <Imprint>  imprint  </Imprint>
                <Genre>  genre  </Genre>
                <Web>  web  </Web>
                <LanguageISO>  language iso  </LanguageISO>
                <Format>  format  </Format>
                <Characters>  characters  </Characters>
                <Teams>  teams  </Teams>
                <Locations>  locations  </Locations>
                <ScanInformation>  scan information  </ScanInformation>
                <StoryArc>  story arc  </StoryArc>
                <StoryArcNumber>  story arc number  </StoryArcNumber>
                <SeriesGroup>  series group  </SeriesGroup>
                <MainCharacterOrTeam>  main character or team  </MainCharacterOrTeam>
                <Review>  review  </Review>
                <GTIN>  gtin  </GTIN>
            </ComicInfo>";
        let comic_info = ComicInfo::from_str(xml).unwrap();
        assert_eq!(comic_info.series, Some("series".to_string()));
        assert_eq!(comic_info.number, Some("number".to_string()));
        assert_eq!(
            comic_info.alternate_series,
            Some("alternate series".to_string())
        );
        assert_eq!(
            comic_info.alternate_number,
            Some("alternate number".to_string())
        );
        assert_eq!(comic_info.summary, Some("summary".to_string()));
        assert_eq!(comic_info.notes, Some("notes".to_string()));
        assert_eq!(comic_info.writer, Some("writer".to_string()));
        assert_eq!(comic_info.penciller, Some("penciller".to_string()));
        assert_eq!(comic_info.inker, Some("inker".to_string()));
        assert_eq!(comic_info.colorist, Some("colorist".to_string()));
        assert_eq!(comic_info.letterer, Some("letterer".to_string()));
        assert_eq!(comic_info.cover_artist, Some("cover artist".to_string()));
        assert_eq!(comic_info.editor, Some("editor".to_string()));
        assert_eq!(comic_info.translator, Some("translator".to_string()));
        assert_eq!(comic_info.publisher, Some("publisher".to_string()));
        assert_eq!(comic_info.imprint, Some("imprint".to_string()));
        assert_eq!(comic_info.genre, Some("genre".to_string()));
        assert_eq!(comic_info.web, Some("web".to_string()));
        assert_eq!(comic_info.language_iso, Some("language iso".to_string()));
        assert_eq!(comic_info.format, Some("format".to_string()));
        assert_eq!(comic_info.characters, Some("characters".to_string()));
        assert_eq!(comic_info.teams, Some("teams".to_string()));
        assert_eq!(comic_info.locations, Some("locations".to_string()));
        assert_eq!(
            comic_info.scan_information,
            Some("scan information".to_string())
        );
        assert_eq!(comic_info.story_arc, Some("story arc".to_string()));
        assert_eq!(
            comic_info.story_arc_number,
            Some("story arc number".to_string())
        );
        assert_eq!(comic_info.series_group, Some("series group".to_string()));
        assert_eq!(
            comic_info.main_character_or_team,
            Some("main character or team".to_string())
        );
        assert_eq!(comic_info.review, Some("review".to_string()));
        assert_eq!(comic_info.gtin, Some("gtin".to_string()));
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}
<ComicInfo>
    <Series>series</Series>
    <Number>number</Number>
    <AlternateSeries>alternate series</AlternateSeries>
    <AlternateNumber>alternate number</AlternateNumber>
    <Summary>summary</Summary>
    <Notes>notes</Notes>
    <Writer>writer</Writer>
    <Penciller>penciller</Penciller>
    <Inker>inker</Inker>
    <Colorist>colorist</Colorist>
    <Letterer>letterer</Letterer>
    <CoverArtist>cover artist</CoverArtist>
    <Editor>editor</Editor>
    <Translator>translator</Translator>
    <Publisher>publisher</Publisher>
    <Imprint>imprint</Imprint>
    <Genre>genre</Genre>
    <Web>web</Web>
    <LanguageISO>language iso</LanguageISO>
    <Format>format</Format>
    <Characters>characters</Characters>
    <Teams>teams</Teams>
    <Locations>locations</Locations>
    <ScanInformation>scan information</ScanInformation>
    <StoryArc>story arc</StoryArc>
    <StoryArcNumber>story arc number</StoryArcNumber>
    <SeriesGroup>series group</SeriesGroup>
    <MainCharacterOrTeam>main character or team</MainCharacterOrTeam>
    <Review>review</Review>
    <GTIN>gtin</GTIN>
</ComicInfo>"
            )
        );
    }

    #[test]
    fn option_string_none() {
        let xml = r"<ComicInfo>
            <Series>  </Series>
            <Number>  </Number>
            <AlternateSeries>  </AlternateSeries>
            <AlternateNumber>  </AlternateNumber>
            <Summary>  </Summary>
            <Notes>  </Notes>
            <Writer>  </Writer>
            <Penciller>  </Penciller>
            <Inker>  </Inker>
            <Colorist>  </Colorist>
            <Letterer>  </Letterer>
            <CoverArtist>  </CoverArtist>
            <Editor>  </Editor>
            <Translator>  </Translator>
            <Publisher>  </Publisher>
            <Imprint>  </Imprint>
            <Genre>  </Genre>
            <Web>  </Web>
            <LanguageISO>  </LanguageISO>
            <Format>  </Format>
            <Characters>  </Characters>
            <Teams>  </Teams>
            <Locations>  </Locations>
            <ScanInformation>  </ScanInformation>
            <StoryArc>  </StoryArc>
            <StoryArcNumber>  </StoryArcNumber>
            <SeriesGroup>  </SeriesGroup>
            <MainCharacterOrTeam>  </MainCharacterOrTeam>
            <Review>  </Review>
            <GTIN>  </GTIN>
        </ComicInfo>";
        let comic_info = ComicInfo::from_str(xml).unwrap();
        assert_eq!(comic_info.series, None);
        assert_eq!(comic_info.number, None);
        assert_eq!(comic_info.alternate_series, None);
        assert_eq!(comic_info.alternate_number, None);
        assert_eq!(comic_info.summary, None);
        assert_eq!(comic_info.notes, None);
        assert_eq!(comic_info.writer, None);
        assert_eq!(comic_info.penciller, None);
        assert_eq!(comic_info.inker, None);
        assert_eq!(comic_info.colorist, None);
        assert_eq!(comic_info.letterer, None);
        assert_eq!(comic_info.cover_artist, None);
        assert_eq!(comic_info.editor, None);
        assert_eq!(comic_info.translator, None);
        assert_eq!(comic_info.publisher, None);
        assert_eq!(comic_info.imprint, None);
        assert_eq!(comic_info.genre, None);
        assert_eq!(comic_info.web, None);
        assert_eq!(comic_info.language_iso, None);
        assert_eq!(comic_info.format, None);
        assert_eq!(comic_info.characters, None);
        assert_eq!(comic_info.teams, None);
        assert_eq!(comic_info.locations, None);
        assert_eq!(comic_info.scan_information, None);
        assert_eq!(comic_info.story_arc, None);
        assert_eq!(comic_info.story_arc_number, None);
        assert_eq!(comic_info.series_group, None);
        assert_eq!(comic_info.main_character_or_team, None);
        assert_eq!(comic_info.review, None);
        assert_eq!(comic_info.gtin, None);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );
    }

    #[test]
    fn default_numbers() {
        assert!(ComicInfo::from_str(
            r"<ComicInfo><Count/><Volume/><AlternateCount/><Year/><Month/><Day/><PageCount/></ComicInfo>"
        )
        .is_err());

        let comic_info = ComicInfo::from_str(r"<ComicInfo/>").unwrap();
        assert_eq!(comic_info.count, -1);
        assert_eq!(comic_info.volume, -1);
        assert_eq!(comic_info.alternate_count, -1);
        assert_eq!(comic_info.year, -1);
        assert_eq!(comic_info.month, -1);
        assert_eq!(comic_info.day, -1);
        assert_eq!(comic_info.page_count, 0);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        assert_eq!(
            ComicInfo::from_str(
                r"<ComicInfo>
                <Count>-1</Count>
                <Volume>-1</Volume>
                <AlternateCount>-1</AlternateCount>
                <Year>-1</Year>
                <Month>-1</Month>
                <Day>-1</Day>
                <PageCount>0</PageCount>
            </ComicInfo>"
            )
            .unwrap()
            .to_pretty_string()
            .unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );
    }

    #[test]
    fn title() {
        assert_eq!(
            ComicInfo::from_str(r"<ComicInfo><Title>  Foo  </Title></ComicInfo>")
                .unwrap()
                .to_pretty_string()
                .unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <Title>Foo</Title>\n</ComicInfo>")
        );

        assert_eq!(
            ComicInfo::from_str(r"<ComicInfo><Title /></ComicInfo>")
                .unwrap()
                .to_pretty_string()
                .unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );
    }

    #[test]
    fn age_rating() {
        assert!(ComicInfo::from_str(r"<ComicInfo><AgeRating>  </AgeRating></ComicInfo>").is_err());

        let xml = r"<ComicInfo></ComicInfo>";
        let comic_info = ComicInfo::from_str(xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::Unknown);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        assert_eq!(
            ComicInfo::from_str(r"<ComicInfo><AgeRating>Unknown  </AgeRating></ComicInfo>")
                .unwrap()
                .to_pretty_string()
                .unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>Adults Only 18+  </AgeRating></ComicInfo>")
                .unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::AdultsOnly18);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Adults Only 18+</AgeRating>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>Early Childhood  </AgeRating></ComicInfo>")
                .unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::EarlyChildhood);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Early Childhood</AgeRating>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>Everyone  </AgeRating></ComicInfo>")
                .unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::Everyone);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Everyone</AgeRating>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>Everyone 10+  </AgeRating></ComicInfo>")
                .unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::Everyone10);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Everyone 10+</AgeRating>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>G  </AgeRating></ComicInfo>").unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::G);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>G</AgeRating>\n</ComicInfo>")
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>Kids to Adults  </AgeRating></ComicInfo>")
                .unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::KidsToAdults);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Kids to Adults</AgeRating>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>M  </AgeRating></ComicInfo>").unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::M);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>M</AgeRating>\n</ComicInfo>")
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>MA15+  </AgeRating></ComicInfo>").unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::MA15);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>MA15+</AgeRating>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>Mature 17+  </AgeRating></ComicInfo>")
                .unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::Mature17);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Mature 17+</AgeRating>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>PG  </AgeRating></ComicInfo>").unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::PG);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>PG</AgeRating>\n</ComicInfo>")
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>R18+  </AgeRating></ComicInfo>").unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::R18);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>R18+</AgeRating>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>Rating Pending  </AgeRating></ComicInfo>")
                .unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::RatingPending);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Rating Pending</AgeRating>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>Teen  </AgeRating></ComicInfo>").unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::Teen);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Teen</AgeRating>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><AgeRating>X18+  </AgeRating></ComicInfo>").unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::X18);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>X18+</AgeRating>\n</ComicInfo>"
            )
        );
    }

    #[test]
    fn pages() {
        assert_eq!(
            ComicInfo::from_str(r"<ComicInfo></ComicInfo>")
                .unwrap()
                .to_pretty_string()
                .unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        assert_eq!(
            ComicInfo::from_str(r"<ComicInfo><Pages></Pages></ComicInfo>")
                .unwrap()
                .to_pretty_string()
                .unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let comic_info = ComicInfo::from_str(r#"<ComicInfo>
            <Pages>
                <Page Image="1" Type="Story" DoublePage="false" ImageSize="0" Key="" Bookmark="" ImageWidth="-1" ImageHeight="-1" ImagePath="" Description=""/>
            </Pages>
        </ComicInfo>"#).unwrap();
        assert_eq!(
            comic_info.pages(),
            &vec![ComicPageInfo {
                image: 1,
                page_type: ComicPageType::Story,
                double_page: false,
                image_size: 0,
                key: String::new(),
                bookmark: String::new(),
                image_width: -1,
                image_height: -1
            }]
        );
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}
<ComicInfo>
    <Pages>
        <Page Image=\"1\"/>
    </Pages>
</ComicInfo>"
            )
        );

        let xml = r#"<ComicInfo>
            <Pages>
                <Page Image="0" Type="FrontCover" DoublePage="false" ImageSize="0" Key="1" Bookmark="2" ImageWidth="3" ImageHeight="4" ImagePath="5" Description="6"/>
            </Pages>
        </ComicInfo>"#;
        let comic_info = ComicInfo::from_str(xml).unwrap();
        assert_eq!(
            comic_info.pages(),
            &vec![ComicPageInfo {
                image: 0,
                page_type: ComicPageType::FrontCover,
                double_page: false,
                image_size: 0,
                key: "1".to_string(),
                bookmark: "2".to_string(),
                image_width: 3,
                image_height: 4
            }]
        );
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}
<ComicInfo>
    <Pages>
        <Page Image=\"0\" Type=\"FrontCover\" Key=\"1\" Bookmark=\"2\" ImageWidth=\"3\" ImageHeight=\"4\"/>
    </Pages>
</ComicInfo>")
        );
    }

    #[test]
    fn manga() {
        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><Manga>Yes  </Manga></ComicInfo>").unwrap();
        assert_eq!(comic_info.manga, Manga::Yes);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <Manga>Yes</Manga>\n</ComicInfo>")
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><Manga>YesAndRightToLeft  </Manga></ComicInfo>")
                .unwrap();
        assert_eq!(comic_info.manga, Manga::YesAndRightToLeft);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <Manga>YesAndRightToLeft</Manga>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><Manga>No  </Manga></ComicInfo>").unwrap();
        assert_eq!(comic_info.manga, Manga::No);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <Manga>No</Manga>\n</ComicInfo>")
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><Manga>Unknown  </Manga></ComicInfo>").unwrap();
        assert_eq!(comic_info.manga, Manga::Unknown);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let comic_info = ComicInfo::from_str(r"<ComicInfo></ComicInfo>").unwrap();
        assert_eq!(comic_info.manga, Manga::Unknown);

        assert!(ComicInfo::from_str(r"<ComicInfo><Manga/></ComicInfo>").is_err());
    }

    #[test]
    fn yesno() {
        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><BlackAndWhite>Yes  </BlackAndWhite></ComicInfo>")
                .unwrap();
        assert_eq!(comic_info.black_and_white, YesNo::Yes);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <BlackAndWhite>Yes</BlackAndWhite>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><BlackAndWhite>No  </BlackAndWhite></ComicInfo>")
                .unwrap();
        assert_eq!(comic_info.black_and_white, YesNo::No);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <BlackAndWhite>No</BlackAndWhite>\n</ComicInfo>"
            )
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><BlackAndWhite>Unknown</BlackAndWhite></ComicInfo>")
                .unwrap();
        assert_eq!(comic_info.black_and_white, YesNo::Unknown);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let comic_info = ComicInfo::from_str(r"<ComicInfo></ComicInfo>").unwrap();
        assert_eq!(comic_info.black_and_white, YesNo::Unknown);

        assert!(
            ComicInfo::from_str(r"<ComicInfo><BlackAndWhite></BlackAndWhite></ComicInfo>").is_err()
        );
    }

    #[test]
    fn rating() {
        let xml = r"<ComicInfo><CommunityRating>0.0  </CommunityRating></ComicInfo>";
        let comic_info = ComicInfo::from_str(xml).unwrap();
        assert_eq!(comic_info.community_rating, Some(Rating(0.0)));
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <CommunityRating>0.0</CommunityRating>\n</ComicInfo>"
            )
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
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <CommunityRating>4.1</CommunityRating>\n</ComicInfo>"
            )
        );
    }

    #[test]
    fn tags() {
        let comic_info = ComicInfo::from_str(r"<ComicInfo><Tags>  </Tags></ComicInfo>").unwrap();
        assert_eq!(comic_info.tags, Vec::<String>::new());
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let comic_info =
            ComicInfo::from_str(r"<ComicInfo><Tags>tag3  , tag1  , tag2  </Tags></ComicInfo>")
                .unwrap();
        assert_eq!(
            comic_info.tags,
            Vec::<String>::from(["tag1".into(), "tag2".into(), "tag3".into()])
        );
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <Tags>tag1, tag2, tag3</Tags>\n</ComicInfo>"
            )
        );
    }
}
