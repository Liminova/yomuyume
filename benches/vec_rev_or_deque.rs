use std::collections::VecDeque;

use anyhow::{anyhow, Result};
use regex::Regex;

const STRING: &str = "chap123abc001";

fn vec_and_rev() -> Result<(String, String)> {
    let mut base_name = Vec::with_capacity(STRING.len());
    let mut chapter_number = Vec::with_capacity(STRING.len());
    for ch in STRING.chars().rev() {
        if chapter_number.is_empty() && !ch.is_ascii_digit() {
            return Err(anyhow!("found non-digit at the end"));
        }
        if base_name.is_empty() && ch.is_ascii_digit() {
            chapter_number.push(ch);
            continue;
        }
        if ch.is_ascii_alphanumeric() {
            base_name.push(ch.to_ascii_lowercase());
        }
    }
    Ok((
        base_name.iter().rev().collect::<String>(),
        chapter_number.iter().rev().collect::<String>(),
    ))
}

fn vec_deque() -> Result<(String, String)> {
    let mut base_name = VecDeque::with_capacity(STRING.len());
    let mut chapter_number = VecDeque::with_capacity(STRING.len());
    for ch in STRING.chars().rev() {
        if chapter_number.is_empty() && !ch.is_ascii_digit() {
            return Err(anyhow!("found non-digit at the end"));
        }
        if base_name.is_empty() && ch.is_ascii_digit() {
            chapter_number.push_front(ch);
            continue;
        }
        base_name.push_front(ch);
    }
    Ok((
        base_name.into_iter().collect::<String>(),
        chapter_number.into_iter().collect::<String>(),
    ))
}

fn use_regex() -> Result<(String, String)> {
    let re = Regex::new(r"(.*?)(\d+)$").unwrap();
    let caps = re.captures(STRING).unwrap();
    Ok((
        caps.get(1).unwrap().as_str().to_string(),
        caps.get(2).unwrap().as_str().to_string(),
    ))
}

pub fn main(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("vec_rev_or_deque");

    group.bench_function("vec_and_rev", |b| {
        b.iter(|| {
            let _ = vec_and_rev().unwrap();
        })
    });

    group.bench_function("vec_deque", |b| {
        b.iter(|| {
            let _ = vec_deque().unwrap();
        })
    });

    group.bench_function("use_regex", |b| {
        b.iter(|| {
            let _ = use_regex().unwrap();
        })
    });
}
