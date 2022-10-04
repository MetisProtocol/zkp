use super::super::types::expression::{BinOp, Binary, Eval, MonOp, Operand, Unary};
use super::{
    ChangeErrorKind, ErrorKind, IResult, MIPSLangError, NomParsable, Span, Token, TokenValue,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    character::complete::{digit1 as digit, space0 as space},
    combinator::{map_res, map},
    multi::fold_many0,
    sequence::{delimited, pair},
};

// Parser definition

use std::str::FromStr;

// We parse any expr surrounded by parens, ignoring all whitespaces around those
fn parens(s: Span) -> IResult<Operand> {
    delimited(space, delimited(tag("("), top_level_expr, tag(")")), space)(s)
}

// We transform an integer string into a i64, ignoring surrounding whitespaces
// We look for a digit suite, and try to convert it.
// If either str::from_utf8 or FromStr::from_str fail,
// we fallback to the parens parser defined above

//     atomic_expr = { unsigned | ident | "(" ~ expr ~ ")" }
fn atomic_expr(s: Span) -> IResult<Operand> {
    map(delimited(space, digit, space), |val: Span| Operand::var(val.fragment()))(s)//, // Unsigned
    // alt((
    //     map(delimited(space, digit, space), |val| val), // Ident
    //     parens,
    // ))(s)
}

//     unary_expr = { (op_unary ~ atomic_expr) | atomic_expr }

fn unary_expr(s: Span) -> IResult<Operand> {
    alt((
        map(pair(alt((char('+'), char('-'), char('~'))), atomic_expr), 
        |(op, val): (char, Operand)| {
            match op {
                '-' => -val,
                '~' => !val,
                _ => val
            }
        }),
        atomic_expr,
    ))(s)
}

// We read an initial factor and for each time we find
// a * or / operator followed by another factor, we do
// the math by folding everything
fn mul_expr(s: Span) -> IResult<Operand> {
    let (i, init) = unary_expr(s)?;

    fold_many0(
        pair(alt((char('*'), char('/'))), unary_expr),
        init,
        |acc, (op, val): (char, Operand)| {
            if op == '*' {
                acc * val
            } else {
                acc / val
            }
        },
    )(i)
}

fn add_expr(s: Span) -> IResult<Operand> {
    let (i, init) = mul_expr(s)?;

    fold_many0(
        pair(alt((char('+'), char('-'))), mul_expr),
        init,
        |acc, (op, val): (char, Operand)| {
            if op == '+' {
                acc + val
            } else {
                acc - val
            }
        },
    )(i)
}

fn bit_and_expr(s: Span) -> IResult<Operand> {
    let (i, init) = add_expr(s)?;

    fold_many0(
        pair(char('&'), add_expr),
        init,
        |acc, (_op, val): (char, Operand)| acc & val,
    )(i)
}

fn bit_or_expr(s: Span) -> IResult<Operand> {
    let (i, init) = bit_and_expr(s)?;

    fold_many0(
        pair(char('|'), bit_and_expr),
        init,
        |acc, (_op, val): (char, Operand)| { acc | val },
    )(i)
}

fn top_level_expr(s: Span) -> IResult<Operand> {
    bit_or_expr(s)
}

pub fn expr(s: Span) -> IResult<Operand> {
    top_level_expr(s)
}

// expr = { bit_or_expr }
//     bit_or_expr = { bit_and_expr ~ (op_bit_or ~ bit_and_expr)* } -
//     bit_and_expr = {add_expr ~ (op_bit_and ~ add_expr)* } -
//     add_expr = { mul_expr ~ (op_additive ~ mul_expr)* }
//     mul_expr = { unary_expr ~ (op_multiplicative ~ unary_expr)* }
//     unary_expr = { (op_unary ~ atomic_expr) | atomic_expr }
//     atomic_expr = { unsigned | ident | "(" ~ expr ~ ")" }
