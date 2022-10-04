use crate::mips_lang::Span;
use nom::error::ParseError;
use nom::Err;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Default(nom::error::ErrorKind),
    // Rust parse error
    InvalidBinaryString,
    InvalidHexString,

    // Custom Error Types
    InvalidRegister,
    UnrecognizedInstruction,
    InvalidCharacterEscape
}

#[derive(Debug)]
pub struct MIPSLangError<'a> {
    pub kind: ErrorKind,

    /// Location that caused the error
    pub input: Span<'a>,
}

impl<'a> MIPSLangError<'a> {
    pub fn from_error_kind(input: Span<'a>, kind: ErrorKind) -> Self {
        MIPSLangError {
            kind: kind,
            input: input,
        }
    }

    pub fn append(input: Span<'a>, kind: ErrorKind, other: Self) -> Self {
        todo!()
    }
}

impl<'a> ParseError<Span<'a>> for MIPSLangError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        MIPSLangError {
            kind: ErrorKind::Default(kind),
            input: input,
        }
    }

    fn append(input: Span<'a>, kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

pub trait ChangeErrorKind {
    fn change_error_kind(&mut self, error_kind: ErrorKind);
}

impl<'a> ChangeErrorKind for nom::Err<MIPSLangError<'a>> {
    fn change_error_kind(&mut self, error_kind: ErrorKind) {
        if let Err::Error(internal_error) = self {
            internal_error.kind = error_kind;
        }
    }

}
