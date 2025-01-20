use std::path::PathBuf;

use crate::types::absolute_path::AbsolutePath;

pub const SUPPORTED_ARCHIVE_FORMATS: &[&str] = &["zip", "cbz", "rar", "cbr", "7z"];
pub const SUPPORTED_IMAGE_FORMATS: &[&str] = &[
    "avif", "bmp", "gif", "jpeg", "jpg", "png", "tif", "tiff", "webp",
];

pub const COMICINFO_SCHEMA: &str =
    r#"<?xml-model href="https://delnegend.com/comicinfo-2.1-extended.xsd"?>"#;
pub const COMICINFO_FILENAME: &str = "ComicInfo.xml";
pub const CATEGORY_INFO_SCHEMA: &str =
    r#"<?xml-model href="https://delnegend.com/categoryinfo-1.0.xsd"?>"#;
pub const CATEGORY_INFO_FILENAME: &str = "CategoryInfo.xml";

pub(super) const SECURE_ID_LENGTH: usize = 32;
pub(super) const SECURE_ID_CHARSET: &[char; 64] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '-', '_',
];
pub(super) const SECURE_ID_MASK: usize = SECURE_ID_CHARSET.len().next_power_of_two() - 1;

pub const SESSION_TOKEN_LAST_USED_AT_UPDATE_INTERVAL: i64 = 5 * 60; // 5 minutes

const VERSION_NAMES: [&str; 31] = [
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

#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,
    pub library_path: AbsolutePath,

    pub listen_address: String,
    pub server_port: u16,
    pub database_url: String,
    pub reverse_proxy_ip_header: Option<String>,

    pub smtp_host: Option<String>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: Option<String>,
    pub smtp_from_name: Option<String>,
}

macro_rules! must {
    ($env:expr) => {
        std::env::var($env).expect(concat!($env, " must be set"))
    };
}

macro_rules! optional {
    ($env:expr) => {
        std::env::var($env)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

    ($env:expr, $default:expr) => {
        std::env::var($env)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| $default.to_string())
    };

    (num: $env:expr, $default:expr) => {
        std::env::var($env)
            .unwrap_or_else(|_| $default.to_string())
            .parse()
            .unwrap_or_else(|_| $default)
    };
}

impl Config {
    pub fn init() -> Self {
        let library_path = AbsolutePath::from(&PathBuf::from(must!("LIBRARY_PATH")), None)
            .expect("can't convert LIBRARY_PATH to absolute");

        assert!(library_path.as_ref().is_dir());

        Self {
            app_name: optional!("APP_NAME", "Yomuyume"),
            library_path,
            listen_address: optional!("LISTEN_ADDRESS", "0.0.0.0"),
            server_port: optional!(num: "SERVER_PORT", 3000),
            database_url: must!("DATABASE_URL"),
            reverse_proxy_ip_header: optional!("REVERSE_PROXY_IP_HEADER"),

            smtp_host: optional!("SMTP_HOST"),
            smtp_username: optional!("SMTP_USERNAME"),
            smtp_password: optional!("SMTP_PASSWORD"),
            smtp_from_email: optional!("SMTP_FROM_EMAIL"),
            smtp_from_name: optional!("SMTP_FROM_NAME"),
        }
    }

    pub fn get_version(&self) -> String {
        let semver = env!("CARGO_PKG_VERSION").parse::<semver::Version>();

        if let Ok(semver) = semver {
            format!(
                "{} - {}",
                semver,
                VERSION_NAMES[(semver.major + semver.minor - 1) as usize]
            )
        } else {
            tracing::warn!(
                "couldn't parse a semver out of Cargo.toml? defaulting to 0.0.0-unknown"
            );
            String::from("0.0.0-unknown - No Version Name")
        }
    }
}
