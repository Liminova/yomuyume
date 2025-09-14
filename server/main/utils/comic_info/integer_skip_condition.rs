#![allow(clippy::trivially_copy_pass_by_ref)]

pub const fn int32_neg_one() -> i32 {
    -1
}

pub const fn int32_is_neg_one(i: &i32) -> bool {
    *i == -1
}

pub const fn int32_is_zero(i: &i32) -> bool {
    *i == 0
}

pub const fn int64_is_zero(i: &i64) -> bool {
    *i == 0
}
