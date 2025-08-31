use regex::{NoExpand, Regex};
use std::fs::{File, read_to_string, write};
use std::io::{self, BufRead, BufReader};

fn underscore_to_camel(s: &str) -> String {
    let mut iter = s.split('_');
    let mut result = iter.next().unwrap_or("").to_string();
    for part in iter {
        let mut chrs = part.chars();
        if let Some(first) = chrs.next() {
            result.push_str(&first.to_uppercase().to_string());
            result.push_str(chrs.as_str());
        }
    }
    result
}

const SERVER_CONSTANT_FILE_PATH: &str = "../main/utils/constants.rs";
const CLIENT_COMMON_FILE_PATH: &str = "../../client/composables/api/common.ts";

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", SERVER_CONSTANT_FILE_PATH);

    let marker = "API paths - DO NOT MODIFY THIS LINE";
    let start_marker = format!("// START: {}", marker);
    let end_marker = format!("// END: {}", marker);

    let const_re = Regex::new(r#"^pub const (\w+): &str = "(.*?)";$"#).unwrap();
    let path_param_re = Regex::new(r#"\{(\w+)\}"#).unwrap();

    // Read backend constants and convert lines between markers
    let infile = File::open(SERVER_CONSTANT_FILE_PATH)?;
    let reader = BufReader::new(infile);
    let mut results = Vec::new();
    let mut in_section = false;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with(&start_marker) {
            in_section = true;
            continue;
        }
        if line.starts_with(&end_marker) {
            if !in_section {
                panic!("Missing start marker in {}", SERVER_CONSTANT_FILE_PATH);
            }
            break;
        }
        if !in_section {
            continue;
        }

        let trimmed = line.trim();
        if let Some(caps) = const_re.captures(trimmed) {
            let endpoint_name = &caps[1];
            let mut endpoint_url = caps[2].to_string();
            let params: Vec<_> = path_param_re
                .captures_iter(&endpoint_url)
                .map(|c| c[1].to_string())
                .collect();

            if params.is_empty() {
                results.push(format!(
                    r#"export const {} = '{}';"#,
                    endpoint_name, endpoint_url
                ));
            } else {
                let ts_params = params
                    .iter()
                    .map(|p| format!("{}: string", underscore_to_camel(p)))
                    .collect::<Vec<_>>()
                    .join(", ");

                for p in &params {
                    let camel = underscore_to_camel(p);

                    let a = format!("{{{}}}", p);
                    let b = format!("${{{}}}", camel);
                    endpoint_url = endpoint_url.replace(&a, &b);
                }

                let ts_endpoint_const = format!(
                    "export const {} = ({}) => `{}`;",
                    endpoint_name, ts_params, endpoint_url
                );
                println!("Updated ts constant: {}", ts_endpoint_const);
                results.push(ts_endpoint_const);
            }
        }
    }

    // Prepare the new block
    let block = results.join("\n");
    let existing = read_to_string(CLIENT_COMMON_FILE_PATH).unwrap_or_default();

    if existing.contains(&start_marker) && existing.contains(&end_marker) {
        let re = Regex::new(&format!(
            "{}[\\s\\S]*?{}",
            regex::escape(&start_marker),
            regex::escape(&end_marker)
        ))
        .unwrap();
        let replacement = format!("{}\n{}\n{}", start_marker, block, end_marker);
        // Use NoExpand to treat `$` literally in replacement (prevent backreference expansion)
        let new_content = re.replace(&existing, NoExpand(replacement.as_str()));
        if new_content != existing {
            write(CLIENT_COMMON_FILE_PATH, new_content.as_ref())?;
            println!("Updated \"{}\"", CLIENT_COMMON_FILE_PATH);
        } else {
            println!("No changes needed for \"{}\"", CLIENT_COMMON_FILE_PATH);
        }
    } else {
        let mut new_content = existing;
        if !new_content.is_empty() {
            new_content.push('\n');
        }
        new_content.push_str(&format!("{}\n{}\n{}\n", start_marker, block, end_marker));
        write(CLIENT_COMMON_FILE_PATH, new_content)?;
        println!("Created \"{}\"", CLIENT_COMMON_FILE_PATH);
    }

    Ok(())
}
