use semver::Version;

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

pub fn get_version() -> String {
    let semver = env!("CARGO_PKG_VERSION").parse::<Version>();

    if let Ok(semver) = semver {
        let version_name = format!(
            "{} - {}",
            semver,
            VERSION_NAMES[(semver.major + semver.minor - 1) as usize]
        );
        version_name
    } else {
        tracing::warn!("couldn't parse a semver out of Cargo.toml? defaulting to 0.0.0-unknown.");
        String::from("0.0.0-unknown - No Version Name")
    }
}
