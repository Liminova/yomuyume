use chrono::Duration;

pub const SUPPORTED_ARCHIVE_FORMATS: [&str; 5] = ["zip", "cbz", "rar", "cbr", "7z"];
pub const SUPPORTED_IMAGE_FORMATS: [&str; 9] = [
    "avif", "bmp", "gif", "jpeg", "jpg", "png", "tif", "tiff", "webp",
];

pub const COMIC_INFO_SCHEMA: &str = r#"<?xml-model href="https://raw.githubusercontent.com/anansi-project/comicinfo/0b6e01/drafts/v2.1/ComicInfo.xsd"?>"#;
pub const COMIC_INFO: &str = "ComicInfo.xml";
pub const CATEGORY_INFO: &str = "CategoryInfo.xml";

pub enum ForgotPasswordLimit {
    Cooldown,
    ExpiredAfter,
}

impl From<ForgotPasswordLimit> for Duration {
    fn from(value: ForgotPasswordLimit) -> Duration {
        match value {
            ForgotPasswordLimit::Cooldown | ForgotPasswordLimit::ExpiredAfter => {
                Duration::minutes(5)
            }
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

// START: API paths - DO NOT MODIFY THIS LINE
pub const SESSION_SECRET_COOKIE_NAME: &str = "session-secret";

pub const LOGOUT_PATH: &str = "/api/auth/logout";
pub const LOGIN_PATH: &str = "/api/auth/login";
pub const REGISTER_PATH: &str = "/api/auth/register";
pub const FORGOT_PATH: &str = "/api/auth/forgot";

pub const GET_NAVIGATION_FEED_PATH: &str = "/api/opds/navigation/{series_or_category}/{id}";
pub const GET_ACQUISITION_FEED_PATH: &str = "/api/opds/acquisition/{title_id}";

pub const GET_PAGE_FILE_PATH: &str = "/api/file/title/{title_id}/{chapter_id}/{page_number}";
pub const GET_COVER_FILE_PATH: &str = "/api/file/cover/{title_or_chapter_id}";

pub const GET_WHOAMI_PATH: &str = "/api/user/whoami";
pub const POST_USER_MODIFY_PATH: &str = "/api/user/modify";
pub const PUT_READ_PROGRESS_PATH: &str = "/api/user/progress";

pub const PUT_COLLECTION_PATH: &str = "/api/user/collection/new/{name}";
pub const TITLE_IN_COLLECTION_PATH: &str = "/api/user/collection/{collection_id}/{title_id}";
pub const DELETE_COLLECTION_PATH: &str = "/api/user/collection/delete/{collection_id}";

pub const GET_STATUS_PATH: &str = "/api/status";
// END: API paths - DO NOT MODIFY THIS LINE
