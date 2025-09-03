use std::fmt::{Debug, Display};

pub trait ResultUtils<T, E: Display + Debug> {
    fn okay<F: FnOnce(E)>(self, f: F) -> Option<T>;
    fn log_err<F: FnOnce(&E)>(self, f: F) -> Result<T, E>;
}

impl<T, E: Display + Debug> ResultUtils<T, E> for Result<T, E> {
    /// Provide an owned error then map to Option
    ///
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

    /// Provide a reference error for tracing calls then forward it
    fn log_err<F: FnOnce(&E)>(self, f: F) -> Result<T, E> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                f(&e);
                Err(e)
            }
        }
    }
}
