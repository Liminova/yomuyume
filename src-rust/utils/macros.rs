macro_rules! bail_if_empty {
    ($x:expr, $return:expr) => {
        if $x.is_empty() {
            return $return;
        }
    };
}

pub(crate) use bail_if_empty;
