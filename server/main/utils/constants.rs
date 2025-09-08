use chrono::Duration;

pub const SUPPORTED_ARCHIVE_FORMATS: [&str; 5] = ["zip", "cbz", "rar", "cbr", "7z"];
pub const SUPPORTED_IMAGE_FORMATS: [&str; 9] = [
    "avif", "bmp", "gif", "jpeg", "jpg", "png", "tif", "tiff", "webp",
];

pub const COMICINFO_SCHEMA: &str = r#"<?xml-model href="https://raw.githubusercontent.com/anansi-project/comicinfo/0b6e01/drafts/v2.1/ComicInfo.xsd"?>"#;
pub const COMICINFO: &str = "ComicInfo.xml";
pub const CATEGORY_INFO_SCHEMA: &str = r#"<?xml-model href="https://raw.githubusercontent.com/Delnegend/Delnegend/c0114e8/categoryinfo-1.0.xsd"?>"#;
pub const CATEGORYINFO: &str = "CategoryInfo.xml";

pub enum ForgotPasswordLimit {
    Cooldown,
    ExpiredAfter,
}

impl From<ForgotPasswordLimit> for Duration {
    fn from(value: ForgotPasswordLimit) -> Self {
        match value {
            ForgotPasswordLimit::Cooldown => Duration::minutes(5),
            ForgotPasswordLimit::ExpiredAfter => Duration::minutes(5),
        }
    }
}

pub const VERSION_NAMES: [&str; 31] = [
    "Highly Responsive to Prayers",
    "Story of Eastern Wonderland",
    "Phantasmagoria of Dim.Dream",
    "Lotus Land Story",
    "Mystic Square",
    "Embodiment of Scarlet Devil",
    "Perfect Cherry Blossom",
    "Immaterial and Missing Power",
    "Imperishable Night",
    "Phantasmagoria of Flower View",
    "Shoot the Bullet",
    "Mountain of Faith",
    "Scarlet Weather Rhapsody",
    "Subterranean Animism",
    "Undefined Fantastic Object",
    "Unperceiving of Natural Law",
    "Double Spoiler",
    "Fairy Wars",
    "Ten Desires",
    "Hopeless Masquerade",
    "Double Dealing Character",
    "Urban Legend in Limbo",
    "Legacy of Lunatic Kingdom",
    "Antinomy of Common Flowers",
    "Hidden Star in Four Seasons",
    "Violet Detector",
    "Wily Beast and Weakest Creature",
    "Sunken Fossil World",
    "Unconnected Marketeers",
    "100th Black Market",
    "Unfinished Dream of All Living Ghost",
];

pub enum CookieName {
    UserID,
    SessionSecret,
}

impl AsRef<str> for CookieName {
    fn as_ref(&self) -> &str {
        match self {
            CookieName::UserID => "user-id",
            CookieName::SessionSecret => "session-secret",
        }
    }
}

// START: API paths - DO NOT MODIFY THIS LINE
pub const LOGOUT_PATH: &str = "/api/auth/logout";
pub const LOGIN_PATH: &str = "/api/auth/login";
pub const REGISTER_PATH: &str = "/api/auth/register";
pub const FORGOT_PATH: &str = "/api/auth/forgot";

pub const GET_CATEGORIES_PATH: &str = "/api/content/categories";
pub const GET_CATEGORY_PATH: &str = "/api/content/{category_id}";
pub const GET_TITLE_PATH: &str = "/api/content/{title_id}";
pub const GET_PAGES_PATH: &str = "/api/content/{chapter_id}";
pub const GET_TAGS_PATH: &str = "/api/content/tags";
pub const GET_SEARCH_PATH: &str = "/api/content/search";

pub const GET_PAGE_FILE_PATH: &str = "/api/file/page/{page_id}";
pub const GET_COVER_FILE_PATH: &str = "/api/file/cover/{title_id}";

pub const GET_WHOAMI_PATH: &str = "/api/user/whoami";
pub const POST_USER_MODIFY_PATH: &str = "/api/user/modify";
pub const PUT_READ_PROGRESS_PATH: &str = "/api/user/progress";
pub const GET_COLLECTIONS_PATH: &str = "/api/user/collections";
pub const PUT_COLLECTION_PATH: &str = "/api/user/collection";
pub const PUT_TITLE_IN_COLLECTION_PATH: &str = "/api/user/collection/{collection_id}/{title_id}";
pub const DELETE_TITLE_FROM_COLLECTION_PATH: &str = PUT_TITLE_IN_COLLECTION_PATH;
pub const DELETE_COLLECTION_PATH: &str = "/api/user/collection/{collection_id}";

pub const GET_STATUS_PATH: &str = "/api/status";
// END: API paths - DO NOT MODIFY THIS LINE
