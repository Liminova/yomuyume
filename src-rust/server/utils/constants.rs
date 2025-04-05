pub const SUPPORTED_ARCHIVE_FORMATS: &[&str] = &["zip", "cbz", "rar", "cbr", "7z"];
pub const SUPPORTED_IMAGE_FORMATS: &[&str] = &[
    "avif", "bmp", "gif", "jpeg", "jpg", "png", "tif", "tiff", "webp", "jxl",
];

pub const COMICINFO_SCHEMA: &str =
    r#"<?xml-model href="https://delnegend.com/comicinfo-2.1-extended.xsd"?>"#;
pub const COMICINFO: &str = "ComicInfo.xml";
pub const CATEGORY_INFO_SCHEMA: &str =
    r#"<?xml-model href="https://delnegend.com/categoryinfo-1.0.xsd"?>"#;
pub const CATEGORYINFO: &str = "CategoryInfo.xml";

pub const SESSION_TOKEN_UPDATE_LAST_USED_AT_INTERVAL: i64 = 5 * 60;
pub const SESSION_TOKEN_EXPIRED_AFTER: i64 = 60 * 60 * 24 * 7; // 7 days

pub const TEMP_CODE_REQUEST_RATE_LIMIT: i64 = 60 * 5; // 5 minutes per request
pub const TEMP_CODE_EXPIRED_AFTER: i64 = 60 * 5; // 5 minutes after request

pub(super) const VERSION_NAMES: [&str; 31] = [
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

pub const SESSION_ID_COOKIE_NAME: &str = "session-id";
pub const SESSION_SECRET_COOKIE_NAME: &str = "session-secret";

// START: API paths - DO NOT MODIFY THIS LINE

pub const LOGOUT_PATH: &str = "/api/auth/logout";
pub const LOGIN_PATH: &str = "/api/auth/login";
pub const REGISTER_PATH: &str = "/api/auth/register";
pub const FORGOT_PATH: &str = "/api/auth/forgot";

pub const GET_CATEGORIES_PATH: &str = "/api/content/categories";
pub const GET_TITLE_PATH: &str = "/api/content/title/{title_id}";
pub const GET_PAGES_PATH: &str = "/api/content/pages/{chapter_id}";
pub const GET_TAGS_PATH: &str = "/api/content/tags";
pub const SEARCH_PATH: &str = "/api/content/search";

pub const GET_PAGE_FILE_PATH: &str = "/api/file/page/{page_id}";
pub const GET_COVER_FILE_PATH: &str = "/api/file/cover/{title_id}";

pub const FAVORITE_PATH: &str = "/api/user/favorite/{title_id}";
pub const BOOKMARK_PATH: &str = "/api/user/bookmark/{title_id}";
pub const WHOAMI_PATH: &str = "/api/user/whoami";
pub const USER_MODIFY_PATH: &str = "/api/user/modify";
pub const USER_SENSITIVE_PATH: &str = "/api/user/sensitive";
pub const USER_PROGRESS_PATH: &str = "/api/user/progress";

pub const GET_SCANNING_PROGRESS_PATH: &str = "/api/admin/scanning_progress";
pub const LIVE_CONFIG_PATH: &str = "/api/admin/live_config";

pub const GET_STATUS_PATH: &str = "/api/status";

// END: API paths - DO NOT MODIFY THIS LINE
