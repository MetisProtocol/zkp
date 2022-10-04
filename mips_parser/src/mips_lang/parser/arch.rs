use super::super::types::arch::{FpReg, Reg};
use super::{
    ChangeErrorKind, ErrorKind, IResult, MIPSLangError, NomParsable, Span, Token, TokenValue,
};

use nom::bytes::complete::tag;
use nom::{branch::alt, combinator::all_consuming};
use nom_locate::position;
use std::str::FromStr;

impl NomParsable for Reg {
    /// Parses a register from the complete string.
    fn parse(s: Span) -> IResult<Token> {
        fn helper_parse_register(s: Span) -> IResult<Token> {
            let (remain, register_frag) = one_of_list_rev![
                all_consuming;
                "$zero", "$0", "$at", "$1", "$v0", "$2", "$v1", "$3", "$a0", "$4", "$a1", "$5", "$a2",
                "$6", "$a3", "$7", "$t0", "$8", "$t1", "$9", "$t2", "$10", "$t3", "$11", "$t4", "$12",
                "$t5", "$13", "$t6", "$14", "$t7", "$15", "$s0", "$16", "$s1", "$17", "$s2", "$18", "$s3",
                "$19", "$s4", "$20", "$s5", "$21", "$s6", "$22", "$s7", "$23", "$t8", "$24", "$t9", "$25",
                "$k0", "$26", "$k1", "$27", "$gp", "$28", "$sp", "$29", "$fp", "$30", "$ra", "$31"
            ](s)?;
            let (remain, pos) = position(remain)?;
            match Reg::from_str(register_frag.fragment()) {
                Ok(reg) => Ok((
                    remain,
                    Token {
                        position: pos,
                        value: TokenValue::Register(reg),
                    },
                )),
                Err(reason) => {
                    IError!(MIPSLangError::from_error_kind(
                        pos,
                        ErrorKind::InvalidRegister
                    ))
                }
            }
        }

        let res = helper_parse_register(s);

        match res {
            ok @ Ok(_) => ok,
            Err(mut nom_error) => {
                nom_error.change_error_kind(ErrorKind::InvalidRegister);
                Err(nom_error)
            }
        }
    }
}

impl NomParsable for FpReg {
    /// Parses a register from the complete string.
    fn parse(s: Span) -> IResult<Token> {
        fn helper_parse_register(s: Span) -> IResult<Token> {
            let (remain, register_frag) = one_of_list_rev![
                all_consuming;
                "$f0","$f1","$f2","$f3","$f4","$f5","$f6","$f7","$f8","$f9","$f10",
                "$f11","$f12","$f13","$f14","$f15","$f16","$f17","$f18","$f19","$f20",
                "$f21","$f22","$f23","$f24","$f25","$f26","$f27","$f28","$f29","$f30","$f31"
            ](s)?;
            let (remain, pos) = position(remain)?;
            match FpReg::from_str(register_frag.fragment()) {
                Ok(reg) => Ok((
                    remain,
                    Token {
                        position: pos,
                        value: TokenValue::FloatReg(reg),
                    },
                )),
                Err(reason) => {
                    IError!(MIPSLangError::from_error_kind(
                        pos,
                        ErrorKind::InvalidRegister
                    ))
                }
            }
        }

        let res = helper_parse_register(s);

        match res {
            ok @ Ok(_) => ok,
            Err(mut nom_error) => {
                nom_error.change_error_kind(ErrorKind::InvalidRegister);
                Err(nom_error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_REGISTERS: [&str; 64] = [
        "$zero", "$0", "$at", "$1", "$v0", "$2", "$v1", "$3", "$a0", "$4", "$a1", "$5", "$a2",
        "$6", "$a3", "$7", "$t0", "$8", "$t1", "$9", "$t2", "$10", "$t3", "$11", "$t4", "$12",
        "$t5", "$13", "$t6", "$14", "$t7", "$15", "$s0", "$16", "$s1", "$17", "$s2", "$18", "$s3",
        "$19", "$s4", "$20", "$s5", "$21", "$s6", "$22", "$s7", "$23", "$t8", "$24", "$t9", "$25",
        "$k0", "$26", "$k1", "$27", "$gp", "$28", "$sp", "$29", "$fp", "$30", "$ra", "$31",
    ];

    const VALID_FP_REGISTERS: [&str; 32] = [
        "$f0", "$f1", "$f2", "$f3", "$f4", "$f5", "$f6", "$f7", "$f8", "$f9", "$f10", "$f11",
        "$f12", "$f13", "$f14", "$f15", "$f16", "$f17", "$f18", "$f19", "$f20", "$f21", "$f22",
        "$f23", "$f24", "$f25", "$f26", "$f27", "$f28", "$f29", "$f30", "$f31",
    ];

    #[test]
    fn test_parse_register() {
        for (idx, reg_str) in VALID_REGISTERS.iter().enumerate() {
            let s = Span::new(reg_str);
            let (remain_str, token) = Reg::parse(s).unwrap();
            assert!("" == *remain_str.fragment());

            match token.value {
                TokenValue::Register(reg) => {
                    assert_eq!(reg, Reg::new((idx / 2) as u32));
                }
                _ => {
                    panic!("Unexpected token value type! Token: {:?}", token);
                }
            }
        }
    }

    #[test]
    fn fail_invalid_register() {
        let s = Span::new("$32");
        let res = Reg::parse(s);
        match res.unwrap_err() {
            nom::Err::Error(err) => {
                assert_eq!(ErrorKind::InvalidRegister, err.kind);
                assert_eq!(0, err.input.location_offset());
                assert_eq!(1, err.input.location_line());
                assert_eq!("$32", *err.input.fragment());
            }
            val @ _ => panic!("Result was not an error type {:?}", val),
        }
    }

    #[test]
    fn test_parse_fp_register() {
        for (idx, reg_str) in VALID_FP_REGISTERS.iter().enumerate() {
            let s = Span::new(reg_str);
            let (remain_str, token) = FpReg::parse(s).unwrap();
            assert!("" == *remain_str.fragment());

            match token.value {
                TokenValue::FloatReg(reg) => {
                    assert_eq!(reg, FpReg::new((idx) as u32));
                }
                _ => {
                    panic!("Unexpected token value type! Token: {:?}", token);
                }
            }
        }
    }

    #[test]
    fn fail_invalid_fp_register() {
        let s = Span::new("$f32");
        let res = FpReg::parse(s);
        match res.unwrap_err() {
            nom::Err::Error(err) => {
                assert_eq!(ErrorKind::InvalidRegister, err.kind);
                assert_eq!(0, err.input.location_offset());
                assert_eq!(1, err.input.location_line());
                assert_eq!("$f32", *err.input.fragment());
            }
            val @ _ => panic!("Result was not an error type {:?}", val),
        }
    }
}
