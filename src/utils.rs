#![allow(unused_macros)]
#![allow(dead_code)]

use chrono::{DateTime, Local};

// Dish out to gloo::console since it doesn't format the inputs.
macro_rules! log {
    ($($arg:tt)+) => (
        gloo::console::log!(format!($($arg)+));
    );
}

macro_rules! assert_true {
    ($exp: expr) => {
        assert_eq!($exp, true);
    };
}
macro_rules! assert_false {
    ($exp: expr) => {
        assert_eq!($exp, false);
    };
}

pub(crate) use assert_false;
pub(crate) use assert_true;
pub(crate) use log;

pub fn ring_iter<'a, T: 'a>(
    v: impl Iterator<Item = T> + Clone,
    start: usize,
) -> impl Iterator<Item = T> {
    v.clone().skip(start).chain(v.take(start))
}

pub fn minmax<T: std::cmp::PartialOrd>(lhs: T, rhs: T) -> (T, T) {
    if lhs <= rhs {
        (lhs, rhs)
    } else {
        (rhs, lhs)
    }
}

pub fn maybe_min<T: std::cmp::PartialOrd>(lhs: T, rhs: T) -> bool {
    lhs < rhs
}

pub fn min_in_place<T: std::cmp::PartialOrd>(lhs: &mut T, rhs: T) -> bool {
    if *lhs >= rhs {
        *lhs = rhs;
        return true;
    }
    false
}

pub fn median<T: std::cmp::PartialOrd + Copy>(mut d: Vec<T>) -> T {
    d.sort_by(|a, b| {
        if a == b {
            std::cmp::Ordering::Equal
        } else if a < b {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
    d[d.len() / 2]
}

pub fn now_ms() -> i64 {
    Local::now().timestamp_millis()
}
