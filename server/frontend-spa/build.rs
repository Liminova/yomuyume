use std::{
    collections::{HashMap, VecDeque, hash_map::Entry},
    env,
    fs::{DirEntry, File},
    io::{BufWriter, Write},
    path::Path,
};

const SPA_DIST_DIR: &str = "../../client/.output/public";

fn main() {
    include_spa();
}

fn ensure_minimum_files() {
    if !Path::new(SPA_DIST_DIR).exists() {
        std::fs::create_dir_all(SPA_DIST_DIR).unwrap();
    }

    let content = r#"<!DOCTYPE html>
            <html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Yomuyume</title>
</head>

<body>
    Hello World
    </body>

    </html>
    "#;

    if !Path::new(SPA_DIST_DIR).join("index.html").exists() {
        writeln!(
            File::create(Path::new(SPA_DIST_DIR).join("index.html")).unwrap(),
            "{}",
            content
        )
        .unwrap();
    }

    if !Path::new(SPA_DIST_DIR).join("404.html").exists() {
        writeln!(
            File::create(Path::new(SPA_DIST_DIR).join("404.html")).unwrap(),
            "{}",
            content
        )
        .unwrap();
    }
}

fn include_spa() {
    println!("cargo:rerun-if-changed={}", SPA_DIST_DIR);

    ensure_minimum_files();

    let mut spa_implicit_index_html = Vec::new();
    let mut spa_mime_type = HashMap::new();

    let mut spa_files = Vec::new();
    let mut spa_dirs = VecDeque::new();

    let mut handle_file = |dir: &DirEntry| {
        if dir.file_type().unwrap().is_dir() {
            return;
        }

        let fpath = dir.path();
        let fext = fpath
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_string();

        if let Entry::Vacant(e) = spa_mime_type.entry(fext.clone()) {
            match fext.as_str() {
                "html" => e.insert("text/html"),
                "css" => e.insert("text/css"),
                "js" => e.insert("application/javascript"),
                "json" => e.insert("application/json"),
                "png" => e.insert("image/png"),
                "jpg" => e.insert("image/jpeg"),
                "jpeg" => e.insert("image/jpeg"),
                "gif" => e.insert("image/gif"),
                "webp" => e.insert("image/webp"),
                "ico" => e.insert("image/x-icon"),
                "wasm" => e.insert("application/wasm"),
                "woff2" => e.insert("font/woff2"),
                "ttf" => e.insert("font/ttf"),
                "xml" => e.insert("application/xml"),
                "svg" => e.insert("image/svg+xml"),
                "" => e.insert("text/plain"),

                _ => panic!("unhandled file extension \"{}\" in build.rs", fext),
            };
        }

        if fpath.ends_with("index.html") {
            let mut tmp = fpath.clone();
            tmp.pop();
            spa_implicit_index_html.push(tmp.to_string_lossy().to_string());
        }

        spa_files.push(fpath);
    };

    for dir in Path::new(SPA_DIST_DIR).read_dir().unwrap() {
        let dir = dir.unwrap();
        if dir.file_type().unwrap().is_dir() {
            spa_dirs.push_back(dir);
        } else {
            handle_file(&dir);
        }
    }

    while let Some(dir_) = spa_dirs.pop_front() {
        for dir in dir_.path().read_dir().unwrap() {
            let dir = dir.unwrap();
            if dir.file_type().unwrap().is_dir() {
                spa_dirs.push_back(dir);
            } else {
                handle_file(&dir);
            }
        }
    }

    let spa_implicit_index_html = {
        let mut tmp = spa_implicit_index_html
            .into_iter()
            .map(|p| {
                p.trim_start_matches(SPA_DIST_DIR)
                    .trim_start_matches("/")
                    .to_string()
            })
            .filter(|p| !p.is_empty())
            .map(|p| format!(r#""{p}""#))
            .collect::<Vec<_>>();
        tmp.sort();
        tmp.dedup();
        tmp
    };

    let spa_mime_type = {
        let mut tmp = spa_mime_type
            .into_iter()
            .filter(|(ext, _)| ext.as_str() != "")
            .collect::<Vec<_>>();
        tmp.sort_by(|a, b| a.0.cmp(&b.0));
        tmp.dedup_by(|a, b| a.0 == b.0);
        tmp.into_iter()
            .map(|(ext, mime_type)| format!(r#""{ext}" => Some("{mime_type}"),"#))
            .collect::<Vec<_>>()
            .join("\n        ")
    };

    spa_files.sort();
    spa_files.dedup();
    let spa_files = spa_files
        .into_iter()
        .map(|path| {
            format!(
                "\"{}\" => Some(include_bytes!(\"../../../{}\")),",
                path.to_string_lossy()
                    .trim_start_matches(SPA_DIST_DIR)
                    .trim_start_matches("/"),
                path.to_string_lossy()
            )
        })
        .collect::<Vec<_>>()
        .join("\n        ");

    let new_content = format!(
        r#"// Contains paths that are supposed to be served by an `index.html` file, but the request doesn't have the `index.html` postfix.
const IMPLICIT_INDEX_HTML: [&str; {}] = [{}];

fn get_mime_type(ext: impl AsRef<str>) -> Option<&'static str> {{
    match ext.as_ref() {{
        {}
        _ => None,
    }}
}}

fn get_file_(path: impl AsRef<str>) -> Option<&'static [u8]> {{
    #[allow(clippy::match_same_arms)]
    match path.as_ref() {{
        {}
        _ => None,
    }}
}}"#,
        spa_implicit_index_html.len(),
        spa_implicit_index_html.join(", "),
        spa_mime_type,
        spa_files
    );

    let codegen_path = Path::new(&env::var("OUT_DIR").unwrap()).join("spa.rs");
    let mut codegen_file = BufWriter::new(File::create(&codegen_path).unwrap());

    let old_content = std::fs::read_to_string(&codegen_path).unwrap_or_default();
    if new_content != old_content {
        writeln!(&mut codegen_file, "{new_content}").unwrap();
    }
}
