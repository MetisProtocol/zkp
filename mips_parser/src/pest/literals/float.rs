use crate::parser::{MIPSParser, Rule};
use crate::utils::{ToSigned, ToUnsigned};
use pest::{error::Error, Parser};
use std::num::Wrapping;

/// Sub parser for unsigned integers. It supports reading from
/// - Hex strings prefixed with 0X or 0x. Underscores are removed, so 0x_f_f is valid
/// - Binary strings prefixed with 0B or 0b. Underscores are removed, so 0x_f_f is valid
/// - Decimal strings.
/// - Characters
pub fn parse_float(pair: pest::iterators::Pair<Rule>) -> f32 {
    let float_str = pair.as_str();
    let cleaned_str = float_str.replace("_", "");
    str::parse::<f32>(&cleaned_str).unwrap()
}


#[cfg(test)]
mod tests {
    use super::*;

    parser_helper!(fn parse_float_str -> f32, pair: Rule::float, Rule::float => parse_float(pair));

    #[test]
    fn test_float () {
        assert_eq!(parse_float_str("3."), 3.);
        assert_eq!(parse_float_str("0.3"), 0.3);
        assert_eq!(parse_float_str("3.0E-5"), 3.0E-5);
        assert_eq!(parse_float_str("3.0e10"), 3.0e10);
    }

    #[test]
    #[should_panic]
    fn test_float_fail_int () {
        assert_eq!(parse_float_str("3"), 3.)
    }
    
    #[test]
    #[should_panic]
    fn test_float_fail_signed_int () {
        assert_eq!(parse_float_str("-."), 0.)
    }


    #[test]
    #[should_panic]
    fn test_float_fail_binary_notation () {
        assert_eq!(parse_float_str("0b101010e10"), 3.0e10);
    }
}
