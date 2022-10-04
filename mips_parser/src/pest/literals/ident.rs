use crate::parser::{MIPSParser, Rule};
use pest::{error::Error, Parser};

/// Parses an ident from the given rule. This method assumes that the given pair already matches Rule::ident
pub fn parse_ident (pair: pest::iterators::Pair<Rule>) -> &str {
    pair.as_str()
}


#[cfg(test)]
mod tests {
    use super::*;
    
    parser_helper!(fn parse_ident_str -> &str, pair: Rule::ident, Rule::ident => parse_ident(pair));

    #[test]
    fn test_ident() {
        let case = "abc";
        let expected = "abc";
        assert_eq!(expected, parse_ident_str(case));
    }

    #[test]
    fn test_special() {
        let case = "_$$";
        let expected = "_$$";
        assert_eq!(expected, parse_ident_str(case));
    }

    #[test]
    #[should_panic]
    fn fail_ident() {
        let case = "0ab";
        parse_ident_str(case);
    }

    #[test]
    #[should_panic]
    fn fail_ident_reserved_keyword() {
        let case = "$s0";
        parse_ident_str(case);
    }
}