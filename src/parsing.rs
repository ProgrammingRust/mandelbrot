use std::str::FromStr;
use rug::{Complex, Float};
use rug::ops::CompleteRound;
use crate::{MyError, PREC};

/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5"`.
///
/// Specifically, `s` should have the form <left><sep><right>, where <sep> is
/// the character given by the `separator` argument, and <left> and <right> are both
/// strings that can be parsed by `T::from_str`.
///
/// If `s` has the proper form, return `Some<(x, y)>`. If it doesn't parse
/// correctly, return `None`.
pub(crate) fn parse_pair<T: Parseable>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}


/// Parse a pair of floating-point numbers separated by a comma as a complex
/// number.
pub(crate) fn parse_complex(s: &str) -> Option<Complex> {
    parse_pair::<Float>(s, ',')
        .map(|(re, im)| Complex::with_val(PREC, (re, im)))
}

pub(crate) trait Parseable {
    fn from_str(s: &str) -> Result<Self, MyError> where Self: Sized;
}

impl Parseable for Float {
    fn from_str(s: &str) -> Result<Self, MyError> where Self: Sized {
        let incomplete = Float::parse(s);

        match incomplete {
            Ok(incomplete) => { Ok(incomplete.complete(PREC)) }
            Err(err) => { Err(MyError::from(err)) }
        }
    }
}

impl Parseable for usize {
    fn from_str(s: &str) -> Result<Self, MyError> where Self: Sized {
        <usize as FromStr>::from_str(s).map_err(MyError::from)
    }
}

impl Parseable for i32 {
    fn from_str(s: &str) -> Result<Self, MyError> where Self: Sized {
        <i32 as FromStr>::from_str(s).map_err(MyError::from)
    }
}

impl Parseable for f64 {
    fn from_str(s: &str) -> Result<Self, MyError> where Self: Sized {
        <f64 as FromStr>::from_str(s).map_err(MyError::from)
    }
}