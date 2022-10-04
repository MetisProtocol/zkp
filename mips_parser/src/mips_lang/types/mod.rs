use nom_locate::LocatedSpan;
use crate::mips_lang::error::MIPSLangError;

pub type Span<'a> = LocatedSpan<&'a str>;
pub type IResult<'a, O> = nom::IResult<Span<'a>, O, MIPSLangError<'a>>;

pub mod arch;
pub mod expression;