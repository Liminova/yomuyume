use std::env::var;

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
    pub server_address: String,
    pub server_port: u16,
    pub database_url: String,
    pub library_path: String,

    pub smtp_host: Option<String>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: Option<String>,
    pub smtp_from_name: Option<String>,

    pub reverse_proxy_ip_header: Option<String>,
    pub developing: bool,

    // Internal variables
    pub cover_filestems: Vec<&'static str>,
    pub supported_img_formats: Vec<&'static str>,
}

impl Config {
    pub fn init() -> Self {
        Self {
            app_name: var("APP_NAME").unwrap_or_else(|_| "Yomuyume".to_string()),
            server_address: var("SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            library_path: var("LIBRARY_PATH").expect("LIBRARY_PATH must be set."),
            database_url: var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./sqlite.db?mode=rwc".to_string()),

            reverse_proxy_ip_header: var("REVERSE_PROXY_IP_HEADER").ok(),
            developing: var("DEVELOPING").unwrap_or_else(|_| "false".to_string()) == "true",

            smtp_host: var("SMTP_HOST").ok(),
            smtp_username: var("SMTP_USERNAME").ok(),
            smtp_password: var("SMTP_PASSWORD").ok(),
            smtp_from_email: var("SMTP_FROM_EMAIL").ok(),
            smtp_from_name: var("SMTP_FROM_NAME").ok(),

            // for the cover image finding stradegy, prioritize
            // files containing any of these strings
            cover_filestems: vec!["cover", "thumbnail", "folder"],
            supported_img_formats: vec![
                "avif", "bmp", "gif", "jpeg", "jpg", "png", "tif", "tiff", "webp",
            ],
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
                "couldn't parse a semver out of Cargo.toml? defaulting to 0.0.0-unknown."
            );
            String::from("0.0.0-unknown - No Version Name")
        }
    }
}
