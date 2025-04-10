use std::{
    collections::{HashSet, VecDeque},
    env,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use include_dir::{include_dir, Dir};

static FRONTEND_SPA: Dir = include_dir!("src-frontend/.output/public");

fn main() {
    let mut spa_implicit_index_html = phf_codegen::Set::new();
    let mut spa_dirs = FRONTEND_SPA
        .entries()
        .iter()
        .filter_map(|entry| entry.as_dir())
        .collect::<VecDeque<_>>();

    let mut mime_types = phf_codegen::Map::new();
    let mut added_mime_types = HashSet::new();

    while let Some(dir) = spa_dirs.pop_front() {
        for entry in dir.entries().iter() {
            if let Some(dir) = entry.as_dir() {
                spa_dirs.push_back(dir);
            } else if let Some(file) = entry.as_file() {
                let fpath = file.path().to_string_lossy().to_string();
                let fpath_ = PathBuf::from(fpath.clone());
                let fext = fpath_
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or_default();
                if added_mime_types.insert(fext.to_string()) {
                    match fext {
                        "html" => mime_types.entry("html", "\"text/html\""),
                        "css" => mime_types.entry("css", "\"text/css\""),
                        "js" => mime_types.entry("js", "\"application/javascript\""),
                        "json" => mime_types.entry("json", "\"application/json\""),
                        "png" => mime_types.entry("png", "\"image/png\""),
                        "jpg" => mime_types.entry("jpg", "\"image/jpeg\""),
                        "jpeg" => mime_types.entry("jpeg", "\"image/jpeg\""),
                        "gif" => mime_types.entry("gif", "\"image/gif\""),
                        "webp" => mime_types.entry("webp", "\"image/webp\""),
                        "ico" => mime_types.entry("ico", "\"image/x-icon\""),
                        "wasm" => mime_types.entry("wasm", "\"application/wasm\""),
                        "woff2" => mime_types.entry("woff2", "\"font/woff2\""),
                        "ttf" => mime_types.entry("ttf", "\"font/ttf\""),
                        "xml" => mime_types.entry("xml", "\"application/xml\""),
                        "svg" => mime_types.entry("svg", "\"image/svg+xml\""),
                        "" => mime_types.entry("", "\"text/plain\""),

                        _ => panic!("unhandled file extension \"{}\" in build.rs", fext),
                    };
                }

                if fpath.ends_with("index.html") {
                    spa_implicit_index_html
                        .entry(fpath.trim_end_matches("/index.html").to_string());
                }
            } else {
                panic!("unexpected entry type in frontend build directory")
            }
        }
    }

    let codegen_path = Path::new(&env::var("OUT_DIR").unwrap()).join("spa.rs");
    let mut codegen_file = BufWriter::new(File::create(&codegen_path).unwrap());

    writeln!(
        &mut codegen_file,
        r#"const FRONTEND_SPA_DIR: include_dir::Dir = include_dir::include_dir!("src-frontend/.output/public");
const FRONTEND_SPA_MIME_TYPES: phf::Map<&'static str, &'static str> = {};
const FRONTEND_SPA_IMPLICIT_INDEX_HTML: phf::Set<&'static str> = {};"#,
        mime_types.build(),
        spa_implicit_index_html.build()
    )
    .unwrap();
}
