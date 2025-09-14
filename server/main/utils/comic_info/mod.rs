//! ComicInfo Version 2.1
//!
//! Schema: https://github.com/anansi-project/comicinfo/blob/0b6e01/drafts/v2.1/ComicInfo.xsd

mod age_rating;
pub mod comic_page_info;
mod integer_skip_condition;
mod manga;
mod rating;
mod tags;
mod yes_no;

use serde::{Deserialize, Deserializer, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::constants::COMICINFO_SCHEMA;
use age_rating::AgeRating;
use comic_page_info::{ArrayOfComicPageInfo, ComicPageInfo};
use integer_skip_condition::{int32_is_neg_one, int32_is_zero, int32_neg_one};
use manga::Manga;
use rating::Rating;
use tags::{tags_deserializer, tags_serializer};
use yes_no::YesNo;

pub fn option_string_deserializer<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let s = String::deserialize(deserializer)?.trim().to_string();
    if s.is_empty() { Ok(None) } else { Ok(Some(s)) }
}

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
    #[serde(
        rename = "Manga",
        default,
        skip_serializing_if = "Manga::is_unknown",
        deserialize_with = "Manga::deserializer"
    )]
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
        deserialize_with = "AgeRating::deserializer",
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

impl ComicInfo {
    pub const fn pages(&self) -> &Vec<ComicPageInfo> {
        &self.pages_.pages_
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

    #[allow(unused)]
    pub fn to_pretty_string(
        &self,
        include_schema: bool,
    ) -> Result<String, quick_xml::errors::serialize::SeError> {
        let mut buffer = if include_schema {
            format!("{COMICINFO_SCHEMA}\n")
        } else {
            String::new()
        };
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
    use crate::utils::comic_info::ComicInfo;

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
            comic_info.to_pretty_string(false).unwrap(),
            format!(
                "<ComicInfo>
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
            comic_info.to_pretty_string(false).unwrap(),
            format!("<ComicInfo/>")
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
            comic_info.to_pretty_string(false).unwrap(),
            format!("<ComicInfo/>")
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
            .to_pretty_string(false)
            .unwrap(),
            format!("<ComicInfo/>")
        );
    }

    #[test]
    fn title() {
        assert_eq!(
            ComicInfo::from_str(r"<ComicInfo><Title>  Foo  </Title></ComicInfo>")
                .unwrap()
                .to_pretty_string(false)
                .unwrap(),
            format!("<ComicInfo>\n    <Title>Foo</Title>\n</ComicInfo>")
        );

        assert_eq!(
            ComicInfo::from_str(r"<ComicInfo><Title /></ComicInfo>")
                .unwrap()
                .to_pretty_string(false)
                .unwrap(),
            format!("<ComicInfo/>")
        );
    }
}
