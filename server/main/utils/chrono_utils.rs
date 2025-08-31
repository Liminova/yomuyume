use chrono::{DateTime, Duration, Utc};

pub trait ChronoUtils {
    fn inside(&self, center: &DateTime<Utc>, range: &Duration) -> bool;
    fn outside(&self, center: &DateTime<Utc>, range: &Duration) -> bool;
}

impl ChronoUtils for DateTime<Utc> {
    /// Check if the current date is within `center-range..=center+range`
    fn inside(&self, center: &DateTime<Utc>, range: &Duration) -> bool {
        (*self - center).abs() <= *range
    }
    /// Same as `within` but inverted
    fn outside(&self, center: &DateTime<Utc>, range: &Duration) -> bool {
        !self.inside(center, range)
    }
}
