use crate::parser::{MIPSParser, Rule};
use pest::{error::Error, Parser};
use nom::{
    bytes::complete::escaped_transform,
    character::complete::{none_of, one_of},
    IResult,
};

/// Replaces escaped characters with their corresponding value in an arbitrary string.
///
/// So "ab\tcd" => "ab  cd"
fn unescape(escaped_str: &str) -> String {
    // This is really annoying to do in rust, so we use nom to parse this properly
    const HEX_DIGIT: &str = "0123456789abcdefABCDEF";
    let escape_string: IResult<&str, String, ()> =
        escaped_transform(none_of("\\"), r"\".chars().next().unwrap(), |i: &str| {
            alt!(i,
              tag!("\\")      => { |_| { '\\' }}
            | tag!("\"")      => { |_| { '\"' }}
            | tag!("n")       => { |_| { '\n' }}
            | tag!("r")       => { |_| { '\r' }}
            | tag!("t")       => { |_| { '\t' }}
            | tag!("0")       => { |_| { '\0' }}
            | tag!("\'")      => { |_| { '\'' }}
            | preceded!(char!('x'), tuple!(one_of!(HEX_DIGIT), one_of!(HEX_DIGIT))) => {
                |(c1, c2): (char, char)| {
                    match (c1.to_digit(16), c2.to_digit(16)) {
                    (Some(d1), Some(d2)) => {
                        (((d1 as u8) << 4) + d2 as u8) as char
                    },
                    _ => unreachable!(),
                }
                }
            }
            )
        })(escaped_str);
    let (remaining, output) = escape_string.unwrap();
    assert_eq!("", remaining);
    output
}

/// Parses a quote_string
pub fn parse_quote_string(pair: pest::iterators::Pair<Rule>) -> String {
    let quote_str = pair.as_str();
    if quote_str == r#""""# {
        return String::from("");
    }
    if quote_str.len() <= 2 {
        panic!("Invalid quote string")
    }

    unescape(&quote_str[1..quote_str.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    parser_helper!(fn parse_quotes_helper -> String, pair: Rule::quote_string, Rule::quote_string => parse_quote_string(pair));

    #[test]
    fn test_quote_string() {
        let actual = parse_quotes_helper(r#""""#);
        assert_eq!("", actual);

        let actual = parse_quotes_helper(r#""\x41""#);
        assert_eq!("A", actual);

        let actual = parse_quotes_helper(r#""\r""#);
        assert_eq!("\r", actual);

        let actual = parse_quotes_helper(r#""\n""#);
        assert_eq!("\n", actual);

        let actual = parse_quotes_helper(r#""\n\r\t\\\0\"\'\x41assad""#);
        assert_eq!("\n\r\t\\\0\"\'\x41assad", actual);

        let actual = parse_quotes_helper(r#""assad""#);
        assert_eq!("assad", actual);
    }
}