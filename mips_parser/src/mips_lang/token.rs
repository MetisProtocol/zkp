use std::str::FromStr;

use crate::mips_lang::{
    types::arch::*,
    types::expression::Operand,
    IResult, Span
};

#[derive(Debug, Clone)]
pub enum TokenValue {
    Register(Reg),
    FloatReg(FpReg),
    Expression(Operand)
}

#[derive(Debug)]
pub struct Token<'a> {
    pub position: Span<'a>,
    pub value: TokenValue,
}

