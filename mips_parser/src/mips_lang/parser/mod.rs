use super::{IResult, Span, Token, TokenValue};
use super::error::{ChangeErrorKind, ErrorKind, MIPSLangError};

pub trait NomParsable {
    fn parse(s: Span) -> IResult<Token>;
}

pub mod arch;
pub mod expression;
pub mod primitives;