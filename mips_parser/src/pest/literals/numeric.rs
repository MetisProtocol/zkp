use crate::parser::{MIPSParser, Rule};
use crate::utils::{ToSigned, ToUnsigned};
use pest::{error::Error, Parser};
use std::num::Wrapping;

/// Sub parser for unsigned integers. It supports reading from
/// - Hex strings prefixed with 0X or 0x. Underscores are removed, so 0x_f_f is valid
/// - Binary strings prefixed with 0B or 0b. Underscores are removed, so 0x_f_f is valid
/// - Decimal strings.
/// - Characters
pub fn parse_unsigned(pair: pest::iterators::Pair<Rule>) -> u32 {
    match pair.as_rule() {
        Rule::unsigned => {
            let mut pairs = pair.into_inner();
            let inner = pairs.next().unwrap();

            match inner.as_rule() {
                Rule::bin => u32::from_str_radix(&inner.as_str()[2..].replace("_", ""), 2).unwrap(),

                Rule::hex => {
                    u32::from_str_radix(&inner.as_str()[2..].replace("_", ""), 16).unwrap()
                }

                Rule::dec => u32::from_str_radix(&inner.as_str().replace("_", ""), 10).unwrap(),
                Rule::char => {
                    let character_str = inner.as_str().trim_matches('\'');
                    let character = match &character_str[..] {
                        r"\n" => '\n' as u8,
                        r"\t" => '\t' as u8,
                        r"\\" => '\\' as u8,
                        r"\0" => '\0' as u8,
                        r#"\0""# => '"' as u8,
                        r"\'" => '\'' as u8,
                        &_ => {
                            if &character_str[..1] != r"\" {
                                // Plain character
                                *(&character_str[..1].chars().next().unwrap()) as u8
                            } else if &character_str[..2] == r"\x" {
                                // Hex encoding
                                u8::from_str_radix(&character_str[2..], 16).unwrap()
                            } else {
                                panic!("Unrecognized escape")
                            }
                        }
                    };
                    character as u32
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

/// Sub parser for integers. This mostly exists as a convenience feature
/// - Hex strings prefixed with 0X or 0x. Underscores are removed, so 0x_f_f is valid
/// - Binary strings prefixed with 0B or 0b. Underscores are removed, so 0x_f_f is valid
/// - Decimal strings.
/// - Characters
pub fn parse_int(pair: pest::iterators::Pair<Rule>) -> i32 {
    let mut negative: i32 = 1;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::op_additive => {
                if pair.as_str() == "-" {
                    negative *= -1;
                }
            }
            Rule::unsigned => {
                let i32_wrapped = Wrapping(parse_unsigned(pair)).to_i32w();
                return (Wrapping(negative) * i32_wrapped).to_i32();
            }
            _ => unreachable!(),
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    parser_helper!(fn parse_unsigned_str -> u32, pair: Rule::unsigned, Rule::unsigned => parse_unsigned(pair));
    parser_helper!(fn parse_int_str -> i32, pair: Rule::int, Rule::int => parse_int(pair));

    #[test]
    fn test_unsigned() {
        assert_eq!(5, parse_unsigned_str("0B_10_1"));
        assert_eq!(3, parse_unsigned_str("0b_01_1"));
        assert_eq!(510, parse_unsigned_str("0x_1f_e"));
        assert_eq!(239, parse_unsigned_str("0X_E_f"));
        assert_eq!(239, parse_unsigned_str("239"));
        assert_eq!(99, parse_unsigned_str("'c'"));
        assert_eq!(10, parse_unsigned_str(r"'\n'"));
        assert_eq!(0, parse_unsigned_str(r#"'\0'"#));
        assert_eq!(255, parse_unsigned_str(r#"'\xff'"#));
    }

    #[test]
    fn test_int() {
        assert_eq!(5, parse_int_str("0B_10_1"));
        assert_eq!(-3, parse_int_str("-0b_01_1"));
        assert_eq!(510, parse_int_str("0x_1f_e"));
        assert_eq!(-239, parse_int_str("-0X_E_f"));
        assert_eq!(239, parse_int_str("239"));
        assert_eq!(-99, parse_int_str("-'c'"));
        assert_eq!(10, parse_int_str(r"'\n'"));
        assert_eq!(0, parse_int_str(r#"'\0'"#));
        assert_eq!(-255, parse_int_str(r#"-'\xff'"#));
    }

    #[test]
    #[should_panic]
    fn test_invalid_unsigned() {
        assert_eq!(5, parse_unsigned_str("'as'"));
    }
}
