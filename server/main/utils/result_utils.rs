pub trait ResultUtils<T, E> {
    fn okay<F: FnOnce(E)>(self, f: F) -> Option<T>;
    fn log_err<F: FnOnce(&E)>(self, f: F) -> Result<T, E>;
}

impl<T, E> ResultUtils<T, E> for Result<T, E> {
    /// Syntax sugar for [`Result::map_err`] + [`Result::ok`]
    fn okay<F: FnOnce(E)>(self, f: F) -> Option<T> {
        self.map_err(f).ok()
    }

    /// Provide a reference error for tracing calls then forward it
    fn log_err<F: FnOnce(&E)>(self, f: F) -> Result<T, E> {
        self.map_err(|e| {
            f(&e);
            e
        })
    }
}

pub trait OptionUtils<T> {
    fn log_err<F: FnOnce()>(self, f: F) -> Option<T>;
}

impl<T> OptionUtils<T> for Option<T> {
    /// Provide a closure for tracing calls then forward it
    fn log_err<F: FnOnce()>(self, f: F) -> Option<T> {
        self.or_else(|| {
            f();
            None
        })
    }
}
