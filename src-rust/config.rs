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

    pub jwt_secret: String,
    pub jwt_maxage_day: chrono::Duration,

    pub smtp_host: Option<String>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: Option<String>,
    pub smtp_from_name: Option<String>,

    // Internal variables
    pub cover_filestems: Vec<&'static str>,
    pub supported_img_formats: Vec<&'static str>,
}

impl Config {
    fn get_env(key: &str, default: Option<&str>) -> String {
        match default {
            Some(val) => std::env::var(key).unwrap_or(val.to_string()),
            None => std::env::var(key).unwrap_or_else(|_| panic!("{} must be set.", key)),
        }
    }

    fn may_get(key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    pub fn init() -> Self {
        Self {
            app_name: Self::get_env("APP_NAME", Some("Yomuyume")),
            server_address: Self::get_env("SERVER_ADDRESS", Some("0.0.0.0")),
            server_port: Self::get_env("SERVER_PORT", Some("3000"))
                .parse()
                .unwrap_or(3000),
            library_path: Self::get_env("LIBRARY_PATH", Some("/library")),
            database_url: Self::get_env("DATABASE_URL", Some("sqlite:./sqlite.db?mode=rwc")),

            jwt_secret: Self::get_env("JWT_SECRET", None),
            jwt_maxage_day: chrono::Duration::try_days(
                Self::get_env("JWT_MAXAGE_DAY", Some("30"))
                    .parse()
                    .unwrap_or(30),
            )
            .expect("JWT_MAXAGE_DAY was not set"),

            smtp_host: Self::may_get("SMTP_HOST"),
            smtp_username: Self::may_get("SMTP_USERNAME"),
            smtp_password: Self::may_get("SMTP_PASSWORD"),
            smtp_from_email: Self::may_get("SMTP_FROM_EMAIL"),
            smtp_from_name: Self::may_get("SMTP_FROM_NAME"),

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
            let version_name = format!(
                "{} - {}",
                semver,
                VERSION_NAMES[(semver.major + semver.minor - 1) as usize]
            );
            version_name
        } else {
            tracing::warn!(
                "couldn't parse a semver out of Cargo.toml? defaulting to 0.0.0-unknown."
            );
            String::from("0.0.0-unknown - No Version Name")
        }
    }
}
