macro_rules! mental_new_type {
    ($($name:ident: $inner:ty),+) => {
        $(#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name($inner);

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl $name {
            pub fn as_ref(&self) -> $inner {
                self.0
            }
        })+
    };
}

mental_new_type!(
    TitleID: i64,
    CategoryID: i64,
    UserID: i64,
    SessionID: i64
);
