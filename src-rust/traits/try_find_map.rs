use anyhow::{anyhow, Result};

pub trait IteratorExt: Iterator {
    /// Same as [`find_map`] but use [`Result<B>`] instead of [`Option<B>`]
    ///
    /// [`find_map`]: Iterator::find_map
    fn try_find_map<B, F>(self, mut f: F) -> Result<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Result<B>,
    {
        let mut error = anyhow!("Item not found");
        self.filter_map(|i| {
            f(i).map_err(|e| {
                error = e;
            })
            .ok()
        })
        .next()
        .ok_or(error)
    }
}

impl<I: Iterator> IteratorExt for I {}
