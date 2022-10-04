use std::{
    boxed::Box, collections::HashMap, collections::HashSet, fmt, mem::transmute_copy,
    num::Wrapping, ops, str, vec::Vec,
};

use crate::utils::{ToUnsigned, ToSigned};

/* Convenience Macros */
macro_rules! op_expr {
    ($first:expr $(, $op: expr, $operand: expr)*) => {
        {
            Operand::Expr(Box::new(Binary {
                first: $first,
                rest: vec![
                    $(
                        BinaryOperation {
                            operator: $op,
                            operand: $operand,
                        },
                    )*
                ],
            }))
        }
    };
}

macro_rules! op_unary {
    ($op: expr, $operand: expr) => {{
        Operand::Unary(Box::new(Unary {
            operator: $op,
            operand: $operand,
        }))
    }};
}



// Traits

pub trait Eval {
    fn eval(&self, mapping: fn(&str) -> Wrapping<u32>) -> Wrapping<u32>;

    fn eval_u32(&self, mapping: fn(&str) -> Wrapping<u32>) -> u32 {
        self.eval(mapping).0
    }

    fn eval_i32(&self, mapping: fn(&str) -> Wrapping<u32>) -> i32 {
        self.eval(mapping).to_i32()
    }
}

// C-like Enums
/// Unary Operators.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MonOp {
    PosOp,
    NegOp,
    BitNotOp,
}

impl str::FromStr for MonOp {
    type Err = ();

    fn from_str(s: &str) -> Result<MonOp, ()> {
        match s {
            "+" => Ok(MonOp::PosOp),
            "-" => Ok(MonOp::NegOp),
            "~" => Ok(MonOp::BitNotOp),
            _ => Err(()),
        }
    }
}

impl fmt::Display for MonOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            MonOp::PosOp => write!(f, "+"),
            MonOp::NegOp => write!(f, "-"),
            MonOp::BitNotOp => write!(f, "~"),
        }
    }
}

/// Binary Operators.
/// These do support C++ operator precedence
/// 
/// Currently at most 2^16 operators can share the same precedence.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum BinOp {
    TimesOp = 0x00,
    DivideOp = 0x01,
    // ---
    PlusOp = 0x100,
    MinusOp = 0x101,
    // ---
    BitAndOp = 0x200,
    // ---
    BitOrOp = 0x400,
}

impl BinOp {
    fn same_precedence(lhs: &BinOp, rhs: &BinOp) -> bool {
        const PRECEDENCE_MASK: u32 = 0xffffff00;
        (*lhs as u32 & PRECEDENCE_MASK) == (*rhs as u32 & PRECEDENCE_MASK)
    }
}

impl str::FromStr for BinOp {
    type Err = ();

    fn from_str(s: &str) -> Result<BinOp, ()> {
        match s {
            "+" => Ok(BinOp::PlusOp),
            "-" => Ok(BinOp::MinusOp),
            "*" => Ok(BinOp::TimesOp),
            "/" => Ok(BinOp::DivideOp),
            "&" => Ok(BinOp::BitAndOp),
            "|" => Ok(BinOp::BitOrOp),
            _ => Err(()),
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            BinOp::PlusOp => write!(f, "+"),
            BinOp::MinusOp => write!(f, "-"),
            BinOp::TimesOp => write!(f, "*"),
            BinOp::DivideOp => write!(f, "/"),
            BinOp::BitAndOp => write!(f, "&"),
            BinOp::BitOrOp => write!(f, "|"),
        }
    }
}

// Atomic expression types

/// Expression for applying a unary operation to a operand
#[derive(Debug, Clone)]
pub struct Unary {
    operator: MonOp,
    operand: Operand,
}

impl fmt::Display for Unary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.operator, self.operand)
    }
}

impl Eval for Unary {
    fn eval(&self, mapping: fn(&str) -> Wrapping<u32>) -> Wrapping<u32> {
        let value: Wrapping<u32> = self.operand.eval(mapping);

        match self.operator {
            MonOp::PosOp => value,
            MonOp::NegOp => -value,
            MonOp::BitNotOp => !value,
        }
    }
}

/// This helper struct represents the right hand side of an operation
/// So, if we have `1 + 2 -3`, then `+2` and `-3` are the binary operations applied to the left hand side
#[derive(Debug, Clone)]
struct BinaryOperation {
    operator: BinOp,
    operand: Operand,
}

impl fmt::Display for BinaryOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.operator, self.operand)
    }
}

/// An general-case binary expression. We assume that all expressions on the same level have equal precedence.
/// Operations are appplied left to right and it can support chained binary operations.
#[derive(Debug, Clone)]
pub struct Binary {
    first: Operand,
    rest: Vec<BinaryOperation>,
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.first)?;
        for operation in self.rest.iter() {
            write!(f, "{}", operation)?;
        }
        Ok(())
    }
}

impl Eval for Binary {
    fn eval(&self, mapping: fn(&str) -> Wrapping<u32>) -> Wrapping<u32> {
        let mut value = self.first.eval(mapping);
        for oper in self.rest.iter() {
            match oper.operator {
                BinOp::PlusOp => value += oper.operand.eval(mapping),
                BinOp::MinusOp => value -= oper.operand.eval(mapping),
                BinOp::TimesOp => {
                    let mut int = value.to_i32w();
                    int *= oper.operand.eval(mapping).to_i32w();
                    value = int.to_u32w();
                }
                BinOp::DivideOp => {
                    let mut int = value.to_i32w();
                    int /= oper.operand.eval(mapping).to_i32w();
                    value = int.to_u32w();
                }
                BinOp::BitAndOp => value &= oper.operand.eval(mapping),
                BinOp::BitOrOp => value |= oper.operand.eval(mapping),
            };
        }
        value
    }
}

/// Operand for wrapping all types of expressions.
/// 
/// Just like an operand, we can use algebraic operators on it with a few caveats
/// 
/// - The beginning of an expression needs to be seeded with an operand. E.g. in a * b + b * d, both a and c need to be operands due to precedence
/// - All integral types are represented by u32 with wrapping arithmetic. The only exception is multiplication and division where we explicitly convert
/// use wrapping i32 arithmetic
#[derive(Debug, Clone)]
pub enum Operand {
    Var(String),
    Num(Wrapping<u32>),
    Unary(Box<Unary>),
    Expr(Box<Binary>),
}

impl Operand {
    // Atomic factory methods
    pub fn unsigned(num: u32) -> Operand {
        Operand::Num(Wrapping(num))
    }

    /// Construct a variable type operand
    pub fn var(name: &str) -> Operand {
        Operand::Var(String::from(name))
    }

    /// Construct a integer type operand
    pub fn int(num: i32) -> Operand {
        Operand::Num(Wrapping(unsafe { transmute_copy::<i32, u32>(&num) }))
    }

    /// Combine the left and right hand sides. If the combination is of the same precedence as the lhs, then it will merge the rhs without creating a new level.
    fn combine(lhs: Operand, operator: BinOp, rhs: Operand) -> Operand {
        match lhs {
            Operand::Expr(mut expr) => {
                if expr.rest.len() == 0
                    || !BinOp::same_precedence(&operator, &expr.rest[0].operator)
                {
                    op_expr![Operand::Expr(expr), operator, rhs]
                } else {
                    expr.rest.push(BinaryOperation {
                        operator: operator,
                        operand: rhs,
                    });
                    Operand::Expr(expr)
                }
            }
            _ => op_expr![lhs, operator, rhs],
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Operand::Var(name) => write!(f, "{}", name)?,
            Operand::Num(value) => write!(f, "{}", value)?,
            Operand::Unary(unary) => write!(f, "{}", unary)?,
            Operand::Expr(expr) => write!(f, "({})", expr)?,
        }

        Ok(())
    }
}

impl Eval for Operand {
    fn eval(&self, mapping: fn(&str) -> Wrapping<u32>) -> Wrapping<u32> {
        match &self {
            Operand::Var(name) => mapping(name),
            Operand::Num(value) => *value,
            Operand::Unary(unary) => (*unary).eval(mapping),
            Operand::Expr(expr) => (*expr).eval(mapping),
        }
    }
}

/* Add Operations */
impl ops::Add<i32> for Operand {
    type Output = Operand;

    fn add(self, rhs: i32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::PlusOp, Operand::int(rhs)),
            _ => op_expr![self, BinOp::PlusOp, Operand::int(rhs)],
        }
    }
}

impl ops::Add<u32> for Operand {
    type Output = Operand;

    fn add(self, rhs: u32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::PlusOp, Operand::unsigned(rhs)),
            _ => op_expr![self, BinOp::PlusOp, Operand::unsigned(rhs)],
        }
    }
}

impl ops::Add<&str> for Operand {
    type Output = Operand;

    fn add(self, rhs: &str) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::PlusOp, Operand::var(rhs)),
            _ => op_expr![self, BinOp::PlusOp, Operand::var(rhs)],
        }
    }
}

impl ops::Add<Operand> for Operand {
    type Output = Operand;

    fn add(self, rhs: Operand) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::PlusOp, rhs),
            _ => op_expr![self, BinOp::PlusOp, rhs],
        }
    }
}

/* Sub Operations */
impl ops::Sub<i32> for Operand {
    type Output = Operand;

    fn sub(self, rhs: i32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::MinusOp, Operand::int(rhs)),
            _ => op_expr![self, BinOp::MinusOp, Operand::int(rhs)],
        }
    }
}

impl ops::Sub<u32> for Operand {
    type Output = Operand;

    fn sub(self, rhs: u32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::MinusOp, Operand::unsigned(rhs)),
            _ => op_expr![self, BinOp::MinusOp, Operand::unsigned(rhs)],
        }
    }
}

impl ops::Sub<&str> for Operand {
    type Output = Operand;

    fn sub(self, rhs: &str) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::MinusOp, Operand::var(rhs)),
            _ => op_expr![self, BinOp::MinusOp, Operand::var(rhs)],
        }
    }
}

impl ops::Sub<Operand> for Operand {
    type Output = Operand;

    fn sub(self, rhs: Operand) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::MinusOp, rhs),
            _ => op_expr![self, BinOp::MinusOp, rhs],
        }
    }
}

/* Mul operations */
impl ops::Mul<i32> for Operand {
    type Output = Operand;

    fn mul(self, rhs: i32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::TimesOp, Operand::int(rhs)),
            _ => op_expr![self, BinOp::TimesOp, Operand::int(rhs)],
        }
    }
}

impl ops::Mul<u32> for Operand {
    type Output = Operand;

    fn mul(self, rhs: u32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::TimesOp, Operand::unsigned(rhs)),
            _ => op_expr![self, BinOp::TimesOp, Operand::unsigned(rhs)],
        }
    }
}

impl ops::Mul<&str> for Operand {
    type Output = Operand;

    fn mul(self, rhs: &str) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::TimesOp, Operand::var(rhs)),
            _ => op_expr![self, BinOp::TimesOp, Operand::var(rhs)],
        }
    }
}

impl ops::Mul<Operand> for Operand {
    type Output = Operand;

    fn mul(self, rhs: Operand) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::TimesOp, rhs),
            _ => op_expr![self, BinOp::TimesOp, rhs],
        }
    }
}
/* Divide Operations */

impl ops::Div<i32> for Operand {
    type Output = Operand;

    fn div(self, rhs: i32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::DivideOp, Operand::int(rhs)),
            _ => op_expr![self, BinOp::DivideOp, Operand::int(rhs)],
        }
    }
}

impl ops::Div<u32> for Operand {
    type Output = Operand;

    fn div(self, rhs: u32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::DivideOp, Operand::unsigned(rhs)),
            _ => op_expr![self, BinOp::DivideOp, Operand::unsigned(rhs)],
        }
    }
}

impl ops::Div<&str> for Operand {
    type Output = Operand;

    fn div(self, rhs: &str) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::DivideOp, Operand::var(rhs)),
            _ => op_expr![self, BinOp::DivideOp, Operand::var(rhs)],
        }
    }
}

impl ops::Div<Operand> for Operand {
    type Output = Operand;

    fn div(self, rhs: Operand) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::DivideOp, rhs),
            _ => op_expr![self, BinOp::DivideOp, rhs],
        }
    }
}

/* Bitwise And */

impl ops::BitAnd<i32> for Operand {
    type Output = Operand;

    fn bitand(self, rhs: i32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::BitAndOp, Operand::int(rhs)),
            _ => op_expr![self, BinOp::BitAndOp, Operand::int(rhs)],
        }
    }
}

impl ops::BitAnd<u32> for Operand {
    type Output = Operand;

    fn bitand(self, rhs: u32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::BitAndOp, Operand::unsigned(rhs)),
            _ => op_expr![self, BinOp::BitAndOp, Operand::unsigned(rhs)],
        }
    }
}

impl ops::BitAnd<&str> for Operand {
    type Output = Operand;

    fn bitand(self, rhs: &str) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::BitAndOp, Operand::var(rhs)),
            _ => op_expr![self, BinOp::BitAndOp, Operand::var(rhs)],
        }
    }
}

impl ops::BitAnd<Operand> for Operand {
    type Output = Operand;

    fn bitand(self, rhs: Operand) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::BitAndOp, rhs),
            _ => op_expr![self, BinOp::BitAndOp, rhs],
        }
    }
}

/* Bitwise Or */

impl ops::BitOr<i32> for Operand {
    type Output = Operand;

    fn bitor(self, rhs: i32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::BitOrOp, Operand::int(rhs)),
            _ => op_expr![self, BinOp::BitOrOp, Operand::int(rhs)],
        }
    }
}

impl ops::BitOr<u32> for Operand {
    type Output = Operand;

    fn bitor(self, rhs: u32) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::BitOrOp, Operand::unsigned(rhs)),
            _ => op_expr![self, BinOp::BitOrOp, Operand::unsigned(rhs)],
        }
    }
}

impl ops::BitOr<&str> for Operand {
    type Output = Operand;

    fn bitor(self, rhs: &str) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::BitOrOp, Operand::var(rhs)),
            _ => op_expr![self, BinOp::BitOrOp, Operand::var(rhs)],
        }
    }
}

impl ops::BitOr<Operand> for Operand {
    type Output = Operand;

    fn bitor(self, rhs: Operand) -> Operand {
        match &self {
            Operand::Expr(_) => Operand::combine(self, BinOp::BitOrOp, rhs),
            _ => op_expr![self, BinOp::BitOrOp, rhs],
        }
    }
}

/* Unary operations */
impl ops::Neg for Operand {
    type Output = Operand;

    fn neg(self) -> Operand {
        op_unary![MonOp::NegOp, self]
    }
}

impl ops::Not for Operand {
    type Output = Operand;

    fn not(self) -> Operand {
        op_unary![MonOp::BitNotOp, self]
    }
}

impl Operand {
    /// Applies the positive operator on the number. Note that Rust does not support + which is why we define our own
    pub fn positive(self) -> Operand {
        // Do nothing. The + operator is pretty useless except as a symmetry to negative.
        // Since it is so useless, we just compress it entirely.
        self
    }

    // TODO: Add simplify
}

#[cfg(test)]
mod tests {
    use super::*;

    lazy_static! {
        static ref ENV: HashMap<&'static str, Wrapping<u32>> = hashmap! {
            "abc" => Wrapping(123),
            "deadc0de" => Wrapping(0xdeadc0de),
            "deadbeef" => Wrapping(0xdeadbeef),
            "one" => Wrapping(1),
        };
    }

    fn mapping(name: &str) -> Wrapping<u32> {
        *ENV.get(name).unwrap()
    }

    #[test]
    fn test_constant() {
        let expr = Operand::Num(Wrapping(32));
        assert_eq!(32, expr.eval(|_| { Wrapping(0) }).0);
    }

    #[test]
    fn test_label() {
        let expr = Operand::Var(String::from("one"));
        assert_eq!(Wrapping(1), expr.eval(mapping))
    }

    #[test]
    fn test_unary_constant() {
        let num = Wrapping(0xffffffff);

        let expr = Operand::Unary(Box::new(Unary {
            operator: MonOp::PosOp,
            operand: Operand::Num(num),
        }));
        assert_eq!(Wrapping(0xffffffff), expr.eval(mapping));

        let expr = Operand::Unary(Box::new(Unary {
            operator: MonOp::NegOp,
            operand: Operand::Num(num),
        }));
        assert_eq!(Wrapping(1), expr.eval(mapping));

        let expr = Operand::Unary(Box::new(Unary {
            operator: MonOp::BitNotOp,
            operand: Operand::Num(num),
        }));
        assert_eq!(Wrapping(0), expr.eval(mapping));
    }

    #[test]
    fn test_unary_label() {
        let var = String::from("abc");
        let expr = op_unary![MonOp::PosOp, Operand::Var(var.clone())];
        assert_eq!(Wrapping(123), expr.eval(mapping));

        let expr = op_unary![MonOp::NegOp, Operand::Var(var.clone())];
        assert_eq!(Wrapping(0xffffff85), expr.eval(mapping));

        let expr = op_unary![MonOp::BitNotOp, Operand::Var(var.clone())];
        assert_eq!(Wrapping(0xffffff84), expr.eval(mapping));
    }

    #[test]
    fn test_expr_single() {
        let expr = op_expr![Operand::unsigned(12)];
        assert_eq!(Wrapping(12), expr.eval(mapping));

        let expr = op_expr![Operand::var("abc")];
        assert_eq!(Wrapping(123), expr.eval(mapping));
    }

    #[test]
    fn test_expr_valid_ops() {
        // 12 + -13
        let expr = op_expr![Operand::unsigned(12), BinOp::PlusOp, Operand::int(-13)];
        assert_eq!(-1, expr.eval(mapping).to_i32());

        // 12 - 13
        let expr = op_expr![Operand::unsigned(12), BinOp::MinusOp, Operand::int(13)];
        assert_eq!(-1, expr.eval(mapping).to_i32());

        // 12 * -1
        let expr = op_expr![Operand::unsigned(12), BinOp::TimesOp, Operand::int(-1)];
        assert_eq!(-12, expr.eval(mapping).to_i32());

        // -4 / -2
        let expr = op_expr![Operand::int(-4), BinOp::DivideOp, Operand::int(-2)];
        assert_eq!(2, expr.eval(mapping).to_i32());

        // 4 / -2
        let expr = op_expr![Operand::int(4), BinOp::DivideOp, Operand::int(-2)];
        assert_eq!(-2, expr.eval(mapping).to_i32());

        // -4 / 2
        let expr = op_expr![Operand::int(-4), BinOp::DivideOp, Operand::int(2)];
        assert_eq!(-2, expr.eval(mapping).to_i32());

        // 2 / 4
        let expr = op_expr![Operand::int(2), BinOp::DivideOp, Operand::int(4)];
        assert_eq!(0, expr.eval(mapping).to_i32());

        // 0xffff0000 & -1
        let expr = op_expr![
            Operand::unsigned(0xffff0000),
            BinOp::BitAndOp,
            Operand::int(-1)
        ];
        assert_eq!(0xffff0000, expr.eval(mapping).to_u32());

        // 0xffff0000 | -1
        let expr = op_expr![
            Operand::unsigned(0xffff0000),
            BinOp::BitOrOp,
            Operand::int(-1)
        ];
        assert_eq!(0xffffffff, expr.eval(mapping).to_u32());
    }

    #[test]
    fn test_expr_nested() {
        // 12 - 13 = -1
        let expr_neg1 = op_expr![Operand::unsigned(12), BinOp::PlusOp, Operand::int(-13)];
        assert_eq!(Operand::int(-1).eval(mapping), expr_neg1.eval(mapping));

        // Compound instruction
        // abc + abc + abc = 123 * 3 = 369
        let expr_369 = op_expr![
            Operand::var("abc"),
            BinOp::PlusOp,
            Operand::var("abc"),
            BinOp::PlusOp,
            Operand::var("abc")
        ];
        assert_eq!(369, expr_369.eval(mapping).to_u32());

        // Compound instruction
        // one * deadbeef / abc = 30373402
        let expr_30373402 = op_expr![
            Operand::var("one"),
            BinOp::TimesOp,
            Operand::var("deadbeef"),
            BinOp::DivideOp,
            Operand::var("abc")
        ];
        assert_eq!(-4545030, expr_30373402.eval(mapping).to_i32());

        let complex = op_expr![
            expr_neg1,
            BinOp::TimesOp,
            expr_30373402,
            BinOp::DivideOp,
            expr_369
        ];
        assert_eq!(12317, complex.eval(mapping).to_u32())
    }

    #[test]
    fn test_additive_precedence() {
        use BinOp::*;

        let set = hashset! {TimesOp, DivideOp, PlusOp, MinusOp, BitAndOp, BitOrOp};
        for op in set.iter() {
            let same_precedence = BinOp::same_precedence(&PlusOp, op);
            match op {
                PlusOp => assert_eq!(true, same_precedence),
                MinusOp => assert_eq!(true, same_precedence),
                _ => assert_eq!(false, same_precedence),
            };
        }

        for op in set.iter() {
            let same_precedence = BinOp::same_precedence(&MinusOp, op);
            match op {
                PlusOp => assert_eq!(true, same_precedence),
                MinusOp => assert_eq!(true, same_precedence),
                _ => assert_eq!(false, same_precedence),
            };
        }
    }

    #[test]
    fn test_multiplicative_precedence() {
        use BinOp::*;

        let set = hashset! {TimesOp, DivideOp, PlusOp, MinusOp, BitAndOp, BitOrOp};
        for op in set.iter() {
            let same_precedence = BinOp::same_precedence(&DivideOp, op);
            match op {
                TimesOp => assert_eq!(true, same_precedence),
                DivideOp => assert_eq!(true, same_precedence),
                _ => assert_eq!(false, same_precedence),
            };
        }

        for op in set.iter() {
            let same_precedence = BinOp::same_precedence(&DivideOp, op);
            match op {
                TimesOp => assert_eq!(true, same_precedence),
                DivideOp => assert_eq!(true, same_precedence),
                _ => assert_eq!(false, same_precedence),
            };
        }
    }

    #[test]
    fn test_combine_same_precedence() {
        let a = op_expr![Operand::int(-1), BinOp::PlusOp, Operand::unsigned(2)];
        let b = op_expr![Operand::int(3)];
        let c = Operand::combine(a, BinOp::MinusOp, b);

        if let Operand::Expr(expr) = c {
            assert_eq!(2, expr.rest.len());
            assert_eq!(BinOp::MinusOp, expr.rest[1].operator);
            assert_eq!(3, expr.rest[1].operand.eval(mapping).to_i32());
        } else {
            panic!("Operand is not an expression")
        }
    }

    #[test]
    fn test_combine_different_precedence() {
        let a = op_expr![Operand::int(-1), BinOp::PlusOp, Operand::unsigned(2)];
        let b = op_expr![Operand::int(3)];
        let c = Operand::combine(a, BinOp::TimesOp, b);

        if let Operand::Expr(expr) = c {
            assert_eq!(1, expr.rest.len());
            assert_eq!(BinOp::TimesOp, expr.rest[0].operator);
            assert_eq!(3, expr.rest[0].operand.eval(mapping).to_i32());
        } else {
            panic!("Operand is not an expression")
        }
    }

    /* Algebraic notation test */
    #[test]
    fn test_algebraic() {
        let a = Operand::unsigned(1);
        let res: Operand = a + 2 + 3;
        assert_eq!(6, res.eval(mapping).to_i32());

        let a = Operand::unsigned(1);
        let res: Operand = a + 2 * 3 + "abc";
        assert_eq!(130, res.eval(mapping).to_i32());

        let abc = Operand::var("abc");
        let abc2 = Operand::var("abc");
        let res: Operand = Operand::unsigned(1) + abc * abc2 + 42;
        assert_eq!(15172, res.eval(mapping).to_i32());
    }
}
