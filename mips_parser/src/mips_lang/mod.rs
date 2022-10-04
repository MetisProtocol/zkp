#[macro_use]
macro_rules! IError {
    ($error: expr) => {
        Err(nom::Err::Error($error))
    };
}


#[macro_use]
macro_rules! one_of_list {
    // TODO Add more unrolling either through another macro or manually
    // Basic variant
    ($str1: expr, $str2: expr, $str3: expr, $($p2: expr),+) => {
        alt((
            tag($str1),
            tag($str2),
            tag($str3),
            one_of_list![$($p2),+],
        ))
    };
    ($str1: expr, $str2: expr, $str3: expr) => {
        alt((
            tag($str1),
            tag($str2),
            tag($str3),
        ))
    };
    ($str1: expr, $str2: expr) => {
        alt((
            tag($str1),
            tag($str2),
        ))
    };
    ($str1: expr) => {
        tag($str1)
    };
    // Mapped version
    ($fn: expr; $str1: expr, $str2: expr, $str3: expr, $($p2: expr),+) => {
        alt((
            $fn(tag($str1)),
            $fn(tag($str2)),
            $fn(tag($str3)),
            one_of_list![$fn: expr; $($p2),+],
        ))
    };
    ($fn: expr; $str1: expr, $str2: expr, $str3: expr) => {
        alt((
            $fn(tag($str1)),
            $fn(tag($str2)),
            $fn(tag($str3)),
        ))
    };
    ($fn: expr; $str1: expr, $str2: expr) => {
        alt((
            $fn(tag($str1)),
            $fn(tag($str2)),
        ))
    };
    ($fn: expr; $str1: expr) => {
        $fn(tag($str1))
    };
}

#[macro_use]
macro_rules! one_of_list_rev {
    // Plain variant
    ($str1: expr, $str2: expr, $str3: expr, $($p2: expr),+) => {
        alt((
            one_of_list_rev![$($p2),+],
            tag($str3),
            tag($str2),
            tag($str1),
        ))
    };
    // TODO Add more unrolling either through another macro or manually
    ($str1: expr, $str2: expr, $str3: expr) => {
        alt((
            tag($str3),
            tag($str2),
            tag($str1),
        ))
    };
    ($str1: expr, $str2: expr) => {
        alt((
            tag($str2),
            tag($str1),
        ))
    };
    ($str1: expr) => {
        tag($str1)
    };

    // Function map
    ($fn: expr; $str1: expr, $str2: expr, $str3: expr, $($p2: expr),+) => {
        alt((
            one_of_list_rev![$fn; $($p2),+],
            $fn(tag($str3)),
            $fn(tag($str2)),
            $fn(tag($str1)),
        ))
    };
    // TODO Add more unrolling either through another macro or manually
    ($fn: expr; $str1: expr, $str2: expr, $str3: expr) => {
        alt((
            $fn(tag($str3)),
            $fn(tag($str2)),
            $fn(tag($str1)),
        ))
    };
    ($fn: expr; $str1: expr, $str2: expr) => {
        alt((
            $fn(tag($str2)),
            $fn(tag($str1)),
        ))
    };
    ($fn: expr; $str1: expr) => {
        $fn(tag($str1))
    };
}

pub mod error;
pub mod types;
pub mod number;
pub mod token;

pub mod parser;

use self::types::{Span, IResult};
use self::token::{Token, TokenValue};
