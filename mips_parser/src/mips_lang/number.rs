use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::one_of;
use nom::combinator::value;
use nom::named;
use nom::{Err, IResult, Needed};
use std::fmt::Display;

use nom_locate::{position, LocatedSpan};

pub type Span<'a> = LocatedSpan<&'a str>;


#[derive(Debug)]
pub struct Token<'a> {
    pub position: Span<'a>,
    pub foo: &'a str,
    pub bar: &'a str,
}

pub const NUM_REGISTERS: u32 = 32;

// fn alternative<'a>(input: &'a [u8], alternatives: &Vec<String>) -> IResult<&'a [u8], &'a [u8]> {
//     for alternative in alternatives {
//         match tag!(input, alternative.as_bytes()) {
//             done@IResult::Ok(..) => return done,
//             _ => () // continue
//         }
//     }
//     IResult::Err(nom::ErrorKind::Tag) // nothing found.
// }
/// A general purpose register
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Reg {
    pub reg_no: u32,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "${}", self.reg_no)
    }
}

impl Reg {
    pub fn new(idx: u32) -> Self {
        if idx >= NUM_REGISTERS {
            panic!("Register number {} is too large!", idx);
        }

        Reg { reg_no: idx }
    }
}

// fn parse_foobar(s: Span) -> IResult<Span, Token> {
//     let (newline, line) = take_until("\n")(s)?;
//     let (comment, line) = take_until("#")(s)?;

//     let (s, _) = take_until("foo")(s)?;
//     let (s, pos) = position(s)?;
//     let (s, foo) = tag("foo")(s)?;
//     let (s, bar) = tag("bar")(s)?;

//     Ok((
//         s,
//         Token {
//             position: pos,
//             foo: foo.fragment(),
//             bar: bar.fragment(),
//         },
//     ))
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date() {
        // let months = vec![
        //     "January",
        //     "February",
        //     "March",
        //     "April",
        //     "May",
        //     "June",
        //     "July",
        //     "August",
        //     "September",
        //     "October",
        //     "November",
        //     "December",
        // ]
        // .iter()
        // .map(|s| s.to_string())
        // .collect();
        // let parser = generate_alternative_parser(months);
        // named!(alternative, call!(parser));
        // println!("{:?}", alternative("May".as_bytes()));
    }
}
