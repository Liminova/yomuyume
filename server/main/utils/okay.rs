use std::fmt::{Debug, Display};

pub trait MapErrorThenOk<T, E: Display + Debug> {
    fn okay<F: FnOnce(E)>(self, f: F) -> Option<T>;
}

impl<T, E: Display + Debug> MapErrorThenOk<T, E> for Result<T, E> {
    /// Same as [`Result::map_err`] + [`Result::ok`]
    fn okay<F: FnOnce(E)>(self, f: F) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                f(e);
                None
            }
        }
    }
}
