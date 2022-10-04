/* This file exists to contain literals specific to the MIPS32 architecture */

use crate::parser::{MIPSParser, Rule};
use crate::utils::{ToSigned, ToUnsigned};
use pest::{Parser, error::{Error, ErrorVariant}};
use std::fmt::Display;
use std::result::Result;
use std::str::FromStr;

const NUM_REGISTERS: u32 = 32;

/// A general purpose register
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Reg {
    pub reg_no: u32,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "${}", self.reg_no)
    }
}

impl FromStr for Reg {
    type Err = &'static str;
    fn from_str(reg_str: &str) -> std::result::Result<Self, <Self as FromStr>::Err> {
        match reg_str {
            "$zero" | "$0" => Ok(Reg::new(0)),
            "$at" | "$1" => Ok(Reg::new(1)),
            "$v0" | "$2" => Ok(Reg::new(2)),
            "$v1" | "$3" => Ok(Reg::new(3)),
            "$a0" | "$4" => Ok(Reg::new(4)),
            "$a1" | "$5" => Ok(Reg::new(5)),
            "$a2" | "$6" => Ok(Reg::new(6)),
            "$a3" | "$7" => Ok(Reg::new(7)),
            "$t0" | "$8" => Ok(Reg::new(8)),
            "$t1" | "$9" => Ok(Reg::new(9)),
            "$t2" | "$10" => Ok(Reg::new(10)),
            "$t3" | "$11" => Ok(Reg::new(11)),
            "$t4" | "$12" => Ok(Reg::new(12)),
            "$t5" | "$13" => Ok(Reg::new(13)),
            "$t6" | "$14" => Ok(Reg::new(14)),
            "$t7" | "$15" => Ok(Reg::new(15)),
            "$s0" | "$16" => Ok(Reg::new(16)),
            "$s1" | "$17" => Ok(Reg::new(17)),
            "$s2" | "$18" => Ok(Reg::new(18)),
            "$s3" | "$19" => Ok(Reg::new(19)),
            "$s4" | "$20" => Ok(Reg::new(20)),
            "$s5" | "$21" => Ok(Reg::new(21)),
            "$s6" | "$22" => Ok(Reg::new(22)),
            "$s7" | "$23" => Ok(Reg::new(23)),
            "$t8" | "$24" => Ok(Reg::new(24)),
            "$t9" | "$25" => Ok(Reg::new(25)),
            "$k0" | "$26" => Ok(Reg::new(26)),
            "$k1" | "$27" => Ok(Reg::new(27)),
            "$gp" | "$28" => Ok(Reg::new(28)),
            "$sp" | "$29" => Ok(Reg::new(29)),
            "$fp" | "$30" => Ok(Reg::new(30)),
            "$ra" | "$31" => Ok(Reg::new(31)),
            _ => Err("Invalid Register String"),
        }
    }
}

impl Reg {
    fn new(idx: u32) -> Self {
        if idx >= NUM_REGISTERS {
            panic!("Register number {} is too large!", idx);
        }

        Reg { reg_no: idx }
    }
}

/// A general purpose register
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FpReg {
    pub reg_no: u32,
}

impl Display for FpReg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "$f{}", self.reg_no)
    }
}

impl FromStr for FpReg {
    type Err = &'static str;
    fn from_str(reg_str: &str) -> std::result::Result<Self, <Self as FromStr>::Err> {
        match reg_str {
            // Return values
            "$f0" => Ok(FpReg::new(0)),
            "$f1" => Ok(FpReg::new(1)),
            "$f2" => Ok(FpReg::new(2)),
            "$f3" => Ok(FpReg::new(3)),
            // Temporary registers (not preserved)
            "$f4" => Ok(FpReg::new(4)),
            "$f5" => Ok(FpReg::new(5)),
            "$f6" => Ok(FpReg::new(6)),
            "$f7" => Ok(FpReg::new(7)),
            "$f8" => Ok(FpReg::new(8)),
            "$f9" => Ok(FpReg::new(9)),
            "$f10" => Ok(FpReg::new(10)),
            "$f11" => Ok(FpReg::new(11)),
            // Arguments to subprogram (not preserved)
            "$f12" => Ok(FpReg::new(12)),
            "$f13" => Ok(FpReg::new(13)),
            "$f14" => Ok(FpReg::new(14)),
            "$f15" => Ok(FpReg::new(15)),
            // Temporary registers (not preserved)
            "$f16" => Ok(FpReg::new(16)),
            "$f17" => Ok(FpReg::new(17)),
            "$f18" => Ok(FpReg::new(18)),
            "$f19" => Ok(FpReg::new(19)),
            // Saved registers
            "$f20" => Ok(FpReg::new(20)),
            "$f21" => Ok(FpReg::new(21)),
            "$f22" => Ok(FpReg::new(22)),
            "$f23" => Ok(FpReg::new(23)),
            "$f24" => Ok(FpReg::new(24)),
            "$f25" => Ok(FpReg::new(25)),
            "$f26" => Ok(FpReg::new(26)),
            "$f27" => Ok(FpReg::new(27)),
            "$f28" => Ok(FpReg::new(28)),
            "$f29" => Ok(FpReg::new(29)),
            "$f30" => Ok(FpReg::new(30)),
            "$f31" => Ok(FpReg::new(31)),
            _ => Err("Invalid Float Register String"),
        }
    }
}

impl FpReg {
    fn new(idx: u32) -> Self {
        if idx >= NUM_REGISTERS {
            panic!("Float Register number {} is too large!", idx);
        }

        FpReg { reg_no: idx }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convenient_names() {
        let names = [
            "$zero", "$at", "$v0", "$v1", "$a0", "$a1", "$a2", "$a3", "$t0", "$t1", "$t2", "$t3",
            "$t4", "$t5", "$t6", "$t7", "$s0", "$s1", "$s2", "$s3", "$s4", "$s5", "$s6", "$s7",
            "$t8", "$t9", "$k0", "$k1", "$gp", "$sp", "$fp", "$ra",
        ];

        for (i, name) in (0..=31).zip(names.iter()) {
            assert_eq!(str::parse::<Reg>(name).unwrap().reg_no, i);
        }
    }

    #[test]
    fn test_reg_string_conversion() {
        for i in 0..=31 {
            assert_eq!(
                str::parse::<Reg>(&Reg::new(i).to_string()).unwrap(),
                Reg::new(i)
            )
        }
    }

    #[test]
    fn test_fp_reg_string_conversion() {
        for i in 0..=31 {
            assert_eq!(
                str::parse::<FpReg>(&FpReg::new(i).to_string()).unwrap(),
                FpReg::new(i)
            )
        }
    }

    #[test]
    #[should_panic]
    fn test_invalid_fpreg() {
        str::parse::<FpReg>("$f32").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_reg() {
        str::parse::<FpReg>("$32").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_common_name_reg() {
        str::parse::<FpReg>("$s10").unwrap();
    }
}
