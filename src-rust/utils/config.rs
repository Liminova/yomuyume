use std::{env::var, path::PathBuf};

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

impl Config {
    pub fn init() -> Self {
        let library_path = AbsolutePath::from(
            &PathBuf::from(var("LIBRARY_PATH").expect("LIBRARY_PATH must be set")),
            None,
        )
        .expect("can't convert LIBRARY_PATH to absolute");

        if !library_path.as_ref().is_dir() {
            panic!("LIBRARY_PATH was set but isn't a directory");
        }

        Self {
            app_name: var("APP_NAME").unwrap_or_else(|_| "Yomuyume".to_string()),
            library_path,
            listen_address: var("LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            database_url: var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./sqlite.db?mode=rwc".to_string()),
            reverse_proxy_ip_header: var("REVERSE_PROXY_IP_HEADER").ok(),

            smtp_host: var("SMTP_HOST").ok(),
            smtp_username: var("SMTP_USERNAME").ok(),
            smtp_password: var("SMTP_PASSWORD").ok(),
            smtp_from_email: var("SMTP_FROM_EMAIL").ok(),
            smtp_from_name: var("SMTP_FROM_NAME").ok(),
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
