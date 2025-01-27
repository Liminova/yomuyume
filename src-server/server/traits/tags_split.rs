pub trait TagsSplit {
    fn tags_split(self) -> Option<Vec<(String, String)>>;
}

impl TagsSplit for Option<Vec<String>> {
    /// ```
    /// Some(["foo-bar", "baz-qux"]) => Some([("foo", "bar"), ("baz", "qux")])
    /// ```
    fn tags_split(self) -> Option<Vec<(String, String)>> {
        self.map(|tags| {
            tags.into_iter()
                .filter_map(|s| {
                    let mut parts = s.splitn(2, '-');
                    Some((
                        parts.next().filter(|s| !s.is_empty())?.to_string(),
                        parts.next().filter(|s| !s.is_empty())?.to_string(),
                    ))
                })
                .collect()
        })
        .filter(|tags: &Vec<(String, String)>| !tags.is_empty())
    }
}
