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

pub(super) const SECURE_ID_LENGTH: usize = 32;
pub(super) const SECURE_ID_CHARSET: &[char; 64] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '-', '_',
];
pub(super) const SECURE_ID_MASK: usize = SECURE_ID_CHARSET.len().next_power_of_two() - 1;

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
