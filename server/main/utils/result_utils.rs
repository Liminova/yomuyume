pub trait ResultUtils<T, E> {
    fn okay<F: FnOnce(E)>(self, f: F) -> Option<T>;
}

impl<T, E> ResultUtils<T, E> for Result<T, E> {
    /// Syntax sugar for [`Result::map_err`] + [`Result::ok`]
    fn okay<F: FnOnce(E)>(self, f: F) -> Option<T> {
        self.map_err(f).ok()
    }
}

pub trait OptionUtils<T> {
    fn inspect_err<F: FnOnce()>(self, f: F) -> Option<T>;
}

impl<T> OptionUtils<T> for Option<T> {
    /// Provide a closure for tracing calls then forward it
    fn inspect_err<F: FnOnce()>(self, f: F) -> Option<T> {
        self.or_else(|| {
            f();
            None
        })
    }
}
