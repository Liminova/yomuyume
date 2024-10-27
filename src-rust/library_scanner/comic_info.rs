//! ComicInfo Version 2.1 Extended
//!
//! Schema: [`crate::COMICINFO_SCHEMA`]
//!
//! Based on the [ComicInfo Version 2.1 Schema](https://anansi-project.github.io/docs/comicinfo/schemas/v2.1)

use std::{fmt::Display, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use murmur3::murmur3_32;
use quick_xml::de::from_str;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::COMICINFO_SCHEMA;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ComicInfo {
    #[serde(
        rename = "Title",
        default,
        deserialize_with = "Title::deserializer",
        serialize_with = "Title::serializer",
        skip_serializing_if = "Title::is_untitled"
    )]
    pub title: Title,
    #[serde(
        rename = "Series",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub series: Option<String>,
    #[serde(
        rename = "Number",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
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
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub alternate_series: Option<String>,
    #[serde(
        rename = "AlternateNumber",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
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
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub summary: Option<String>,
    #[serde(
        rename = "Notes",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
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
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub writer: Option<String>,
    #[serde(
        rename = "Penciller",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub penciller: Option<String>,
    #[serde(
        rename = "Inker",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub inker: Option<String>,
    #[serde(
        rename = "Colorist",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub colorist: Option<String>,
    #[serde(
        rename = "Letterer",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub letterer: Option<String>,
    #[serde(
        rename = "CoverArtist",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub cover_artist: Option<String>,
    #[serde(
        rename = "Editor",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub editor: Option<String>,
    #[serde(
        rename = "Translator",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub translator: Option<String>,
    #[serde(
        rename = "Publisher",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub publisher: Option<String>,
    #[serde(
        rename = "Imprint",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub imprint: Option<String>,
    #[serde(
        rename = "Genre",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
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
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub web: Option<String>,
    #[serde(rename = "PageCount", default, skip_serializing_if = "int32_is_zero")]
    pub page_count: i32,
    // TODO: maybe use isolang crate?
    #[serde(
        rename = "LanguageISO",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub language_iso: Option<String>,
    #[serde(
        rename = "Format",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
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
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub characters: Option<String>,
    #[serde(
        rename = "Teams",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub teams: Option<String>,
    #[serde(
        rename = "Locations",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub locations: Option<String>,
    #[serde(
        rename = "ScanInformation",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub scan_information: Option<String>,
    #[serde(
        rename = "StoryArc",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub story_arc: Option<String>,
    #[serde(
        rename = "StoryArcNumber",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub story_arc_number: Option<String>,
    #[serde(
        rename = "SeriesGroup",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
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
    pub pages: ArrayOfComicPageInfo,
    #[serde(
        rename = "CommunityRating",
        default,
        serialize_with = "Rating::serializer",
        deserialize_with = "Rating::deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub community_rating: Option<Rating>,
    #[serde(
        rename = "MainCharacterOrTeam",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub main_character_or_team: Option<String>,
    #[serde(
        rename = "Review",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub review: Option<String>,
    #[serde(
        rename = "GTIN",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub gtin: Option<String>,

    #[serde(skip)]
    pages_field_hash: u32,
    #[serde(skip)]
    release_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Title(String);

impl Default for Title {
    fn default() -> Self {
        Title("Untitled".to_string())
    }
}

impl From<&str> for Title {
    fn from(value: &str) -> Self {
        Title(value.trim().to_string())
    }
}

impl Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Title {
    pub fn is_untitled(&self) -> bool {
        self.0 == "Untitled"
    }
    fn deserializer<'de, D>(deserializer: D) -> Result<Title, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.trim().to_string();
        if s.is_empty() {
            return Err(serde::de::Error::custom("empty string"));
        }
        Ok(Title(s))
    }

    fn serializer<S>(title: &Title, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&title.0)
    }
}

fn option_string_deserializer<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?.trim().to_string();
    if s.is_empty() {
        return Ok(None);
    }
    Ok(Some(s))
}

fn tags_deserializer<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?.trim().to_string();
    if s.is_empty() {
        return Ok(vec![]);
    }
    let mut tags: Vec<String> = s
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    tags.sort();
    tags.dedup();
    Ok(tags)
}

fn tags_serializer<S>(tags: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
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
    fn deserializer<'de, D>(deserializer: D) -> Result<YesNo, D::Error>
    where
        D: Deserializer<'de>,
    {
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
    #[serde(rename = "Page", default)]
    pub pages: Vec<ComicPageInfo>,
}

impl ArrayOfComicPageInfo {
    fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ComicPageInfo {
    #[serde(rename = "@Image")]
    pub image: i32,
    #[serde(
        rename = "@Type",
        default,
        skip_serializing_if = "ComicPageType::is_story"
    )]
    pub page_type: ComicPageType,
    #[serde(rename = "@DoublePage", default, skip_serializing_if = "is_false")]
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

    /// Yomuyume custom attributes
    #[serde(
        rename = "@ImagePath",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub image_path: Option<String>,
    #[serde(
        rename = "@Description",
        default,
        deserialize_with = "option_string_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Rating(f32);

impl Eq for Rating {
    fn assert_receiver_is_total_eq(&self) {
        assert!(self.eq(self))
    }
}

impl FromStr for Rating {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = s.parse::<f32>().map_err(|e| e.to_string())?;
        // if f < 0.0 || f > 5.0 {
        if !(0.0..=5.0).contains(&f) {
            return Err(format!(
                "Rating must be between 0.0 and 5.0 (inclusive), got {f}"
            ));
        }
        Ok(Rating((f * 10.0).round() / 10.0))
    }
}

impl Rating {
    fn deserializer<'de, D>(deserializer: D) -> Result<Option<Rating>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.trim().to_string();
        if s.is_empty() {
            return Err(serde::de::Error::custom("empty string"));
        }
        Rating::from_str(s.as_str())
            .map_err(serde::de::Error::custom)
            .map(Some)
    }

    fn serializer<S>(rating: &Option<Rating>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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

fn is_false(b: &bool) -> bool {
    !b
}

impl ComicInfo {
    /// Get the release date of the comic.
    pub fn get_release(&mut self) -> Result<DateTime<Utc>> {
        if let Some(ref release) = self.release_date {
            return Ok(*release);
        }
        let result = DateTime::parse_from_str(
            format!("{}-{}-{}", self.year, self.month, self.day).as_str(),
            "%Y-%m-%d",
        )
        .context("can't parse comic release date")
        .map(|d| d.with_timezone(&Utc));
        if let Ok(result) = result {
            self.release_date = Some(result);
        }
        result
    }

    /// Get the hash of all the pages in the Pages array.
    pub fn get_pages_field_hash(&mut self) -> Result<u32> {
        if self.pages.pages.is_empty() {
            return Ok(0);
        }
        if self.pages_field_hash != 0 {
            return Ok(self.pages_field_hash);
        }

        let raw = self
            .pages
            .pages
            .iter()
            .map(|p| {
                quick_xml::se::to_string(p)
                    .context("can't serialize page metadata")
                    .unwrap_or_default()
            })
            .collect::<Vec<String>>()
            .join("");
        match murmur3_32(&mut &raw.as_bytes()[..], 0) {
            Ok(hash) => {
                self.pages_field_hash = hash;
                Ok(hash)
            }
            Err(e) => Err(anyhow!("can't hash pages field: {}", e)),
        }
    }

    /// Get the description of a page file given its file name.
    pub fn get_page_description(&self, page_file_name: &str) -> Option<String> {
        // split by "." and remove just the last one
        let no_ext = PathBuf::from(page_file_name)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let with_ext = page_file_name.to_string();

        self.pages.pages.iter().find_map(|page| {
            if let Some(ref image_path) = page.image_path {
                if image_path == &with_ext || image_path == &no_ext {
                    return page.description.clone();
                }
            }
            None
        })
    }

    pub fn from_str(s: &str) -> Result<Self> {
        from_str(s).context("can't parse ComicInfo from string")
    }

    pub fn to_pretty_string(&self) -> Result<String> {
        let mut buffer = format!("{COMICINFO_SCHEMA}\n");
        let mut ser = quick_xml::se::Serializer::new(&mut buffer);
        ser.indent(' ', 4);
        self.serialize(ser)
            .context("can't serialize ComicInfo to pretty string")?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_string_some() {
        let xml = r#"<ComicInfo>
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
            </ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
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
        )
    }

    #[test]
    fn option_string_none() {
        let xml = r#"<ComicInfo>
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
        </ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
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
        let xml = r#"<ComicInfo></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(xml).unwrap();
        assert_eq!(comic_info.count, -1);
        assert_eq!(comic_info.volume, -1);
        assert_eq!(comic_info.alternate_count, -1);
        assert_eq!(comic_info.year, -1);
        assert_eq!(comic_info.month, -1);
        assert_eq!(comic_info.day, -1);
        assert_eq!(comic_info.page_count, 0);

        let xml = comic_info.to_pretty_string().unwrap();
        assert_eq!(xml, format!("{COMICINFO_SCHEMA}\n<ComicInfo/>"));

        let xml = r#"<ComicInfo>
            <Count>  </Count>
            <Volume>  </Volume>
            <AlternateCount>  </AlternateCount>
            <Year>  </Year>
            <Month>  </Month>
            <Day>  </Day>
            <PageCount>  </PageCount>
        </ComicInfo>"#;
        assert!(ComicInfo::from_str(&xml).is_err());

        let xml = r#"<ComicInfo>
            <Count>-1</Count>
            <Volume>-1</Volume>
            <AlternateCount>-1</AlternateCount>
            <Year>-1</Year>
            <Month>-1</Month>
            <Day>-1</Day>
            <PageCount>0</PageCount>
        </ComicInfo>"#;
        let xml = ComicInfo::from_str(&xml)
            .unwrap()
            .to_pretty_string()
            .unwrap();
        assert_eq!(xml, format!("{COMICINFO_SCHEMA}\n<ComicInfo/>"));
    }

    #[test]
    fn title() {
        let xml = r#"<ComicInfo><Title>  Lorem Ipsum Title  </Title></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(xml).unwrap();
        assert_eq!(comic_info.title, Title::from("Lorem Ipsum Title"));
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <Title>Lorem Ipsum Title</Title>\n</ComicInfo>"));
    }

    #[test]
    fn age_rating() {
        let xml = r#"<ComicInfo><AgeRating>  </AgeRating></ComicInfo>"#;
        assert!(ComicInfo::from_str(&xml).is_err());

        let xml = r#"<ComicInfo><AgeRating>  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml);
        assert!(comic_info.is_err());

        let xml = r#"<ComicInfo></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::Unknown);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let xml = r#"<ComicInfo><AgeRating>Unknown  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::Unknown);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let xml = r#"<ComicInfo><AgeRating>Adults Only 18+  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::AdultsOnly18);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Adults Only 18+</AgeRating>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><AgeRating>Early Childhood  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::EarlyChildhood);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Early Childhood</AgeRating>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><AgeRating>Everyone  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::Everyone);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Everyone</AgeRating>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><AgeRating>Everyone 10+  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::Everyone10);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Everyone 10+</AgeRating>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><AgeRating>G  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::G);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>G</AgeRating>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><AgeRating>Kids to Adults  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::KidsToAdults);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Kids to Adults</AgeRating>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><AgeRating>M  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::M);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>M</AgeRating>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><AgeRating>MA15+  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::MA15);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>MA15+</AgeRating>\n</ComicInfo>"
            )
        );

        let xml = r#"<ComicInfo><AgeRating>Mature 17+  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::Mature17);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Mature 17+</AgeRating>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><AgeRating>PG  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::PG);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>PG</AgeRating>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><AgeRating>R18+  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::R18);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>R18+</AgeRating>\n</ComicInfo>"
            )
        );

        let xml = r#"<ComicInfo><AgeRating>Rating Pending  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::RatingPending);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Rating Pending</AgeRating>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><AgeRating>Teen  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.age_rating, AgeRating::Teen);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <AgeRating>Teen</AgeRating>\n</ComicInfo>"
            )
        );

        let xml = r#"<ComicInfo><AgeRating>X18+  </AgeRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
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
        let xml = r#"<ComicInfo></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.pages, ArrayOfComicPageInfo { pages: vec![] });
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let xml = r#"<ComicInfo><Pages></Pages></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.pages, ArrayOfComicPageInfo { pages: vec![] });
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let xml = r#"<ComicInfo>
            <Pages>
                <Page Image="1" Type="Story" DoublePage="false" ImageSize="0" Key="" Bookmark="" ImageWidth="-1" ImageHeight="-1" ImagePath="" Description=""/>
            </Pages>
        </ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
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
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(
            comic_info.pages,
            ArrayOfComicPageInfo {
                pages: vec![ComicPageInfo {
                    image: 0,
                    page_type: ComicPageType::FrontCover,
                    double_page: false,
                    image_size: 0,
                    key: "1".to_string(),
                    bookmark: "2".to_string(),
                    image_width: 3,
                    image_height: 4,
                    image_path: Some("5".to_string()),
                    description: Some("6".to_string()),
                }]
            }
        );
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}
<ComicInfo>
    <Pages>
        <Page Image=\"0\" Type=\"FrontCover\" Key=\"1\" Bookmark=\"2\" ImageWidth=\"3\" ImageHeight=\"4\" ImagePath=\"5\" Description=\"6\"/>
    </Pages>
</ComicInfo>")
        );
    }

    #[test]
    fn manga() {
        let xml = r#"<ComicInfo><Manga>Yes  </Manga></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.manga, Manga::Yes);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <Manga>Yes</Manga>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><Manga>YesAndRightToLeft  </Manga></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.manga, Manga::YesAndRightToLeft);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <Manga>YesAndRightToLeft</Manga>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><Manga>No  </Manga></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.manga, Manga::No);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <Manga>No</Manga>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><Manga>Unknown  </Manga></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.manga, Manga::Unknown);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let xml = r#"<ComicInfo></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.manga, Manga::Unknown);

        let xml = r#"<ComicInfo><Manga></Manga></ComicInfo>"#;
        assert!(ComicInfo::from_str(&xml).is_err());
    }

    #[test]
    fn yesno() {
        let xml = r#"<ComicInfo><BlackAndWhite>Yes  </BlackAndWhite></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.black_and_white, YesNo::Yes);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <BlackAndWhite>Yes</BlackAndWhite>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><BlackAndWhite>No  </BlackAndWhite></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.black_and_white, YesNo::No);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <BlackAndWhite>No</BlackAndWhite>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><BlackAndWhite>Unknown  </BlackAndWhite></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.black_and_white, YesNo::Unknown);
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let xml = r#"<ComicInfo></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.black_and_white, YesNo::Unknown);

        let xml = r#"<ComicInfo><BlackAndWhite></BlackAndWhite></ComicInfo>"#;
        assert!(ComicInfo::from_str(&xml).is_err());
    }

    #[test]
    fn rating() {
        let xml = r#"<ComicInfo><CommunityRating>0.0  </CommunityRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.community_rating, Some(Rating(0.0)));
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <CommunityRating>0.0</CommunityRating>\n</ComicInfo>")
        );

        let xml = r#"<ComicInfo><CommunityRating>-1</CommunityRating></ComicInfo>"#;
        assert!(ComicInfo::from_str(&xml).is_err());
        let xml = r#"<ComicInfo><CommunityRating>6.0</CommunityRating></ComicInfo>"#;
        assert!(ComicInfo::from_str(&xml).is_err());
        let xml = r#"<ComicInfo><CommunityRating></CommunityRating></ComicInfo>"#;
        assert!(ComicInfo::from_str(&xml).is_err());

        let xml = r#"<ComicInfo><CommunityRating>4.12  </CommunityRating></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo>\n    <CommunityRating>4.1</CommunityRating>\n</ComicInfo>")
        );
    }

    #[test]
    fn tags() {
        let xml = r#"<ComicInfo><Tags>  </Tags></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(comic_info.tags, Vec::<String>::new());
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!("{COMICINFO_SCHEMA}\n<ComicInfo/>")
        );

        let xml = r#"<ComicInfo><Tags>tag1, tag2, tag3</Tags></ComicInfo>"#;
        let comic_info = ComicInfo::from_str(&xml).unwrap();
        assert_eq!(
            comic_info.tags,
            vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]
        );
        assert_eq!(
            comic_info.to_pretty_string().unwrap(),
            format!(
                "{COMICINFO_SCHEMA}\n<ComicInfo>\n    <Tags>tag1, tag2, tag3</Tags>\n</ComicInfo>"
            )
        );
    }
}
