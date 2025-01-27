use proc_macro::TokenStream;
use regex::Regex;

/// Replace continuous spaces, tabs and newlines in a `&str`` with a single space.
#[proc_macro]
pub fn smol(input: TokenStream) -> TokenStream {
    let mut input = input.to_string();
    let pattern = Regex::new(r"(\s|\t|\n)+").unwrap();
    input = pattern
        .replace_all(&input, " ")
        .replace("( ", "(")
        .replace(" )", ")");
    input.parse().unwrap()
}
