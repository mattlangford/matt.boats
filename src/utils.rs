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
        assert_eq!($exp, true);
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
