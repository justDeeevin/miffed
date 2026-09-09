use std::{borrow::Cow, str::FromStr};
use thiserror::Error;

#[derive(Debug)]
pub struct Program<'a> {
    pub data: Vec<Data<'a>>,
    pub text: Vec<Statement<'a>>,
}

#[derive(Debug)]
pub enum Statement<'a> {
    Label(&'a str),
    Instruction(Instruction),
}

#[derive(Debug)]
pub enum Instruction {
    R {
        rs: Register,
        rt: Register,
        rd: Register,
        shamt: u8,
        funct: Funct,
    },
    I {
        op: IOp,
        rs: Register,
        rt: Register,
        imm: i16,
    },
    J {
        op: JOp,
        offset: i32,
    },
    Nop,
}

#[derive(Debug)]
pub enum JOp {
    J = 2,
    Jal,
}

#[derive(Debug)]
pub enum IOp {
    /// There are many with the opcode 1 :<
    One = 1,
    Beq = 4,
    Bne,
    Blez,
    Bgtz,
    Addi,
    Addiu,
    Slti,
    Sltiu,
    Andi,
    Ori,
    Xori,
    Lui,
    Lb = 0x20,
    Lh,
    Lwl,
    Lw,
    Lbu,
    Lhu,
    Lwr,
    Sb = 0x28,
    Sh,
    Swl,
    Sw,
    Swr = 0x2e,
}

#[derive(Debug)]
pub enum Funct {
    Sll,
    Srl = 2,
    Sra,
    Sllv,
    Srlv = 6,
    Srav,
    Jr,
    Jalr,
    Movz,
    Movn,
    Syscall,
    Break,
    Mfhi = 16,
    Mthi,
    Mflo,
    Mtlo,
    Mult = 24,
    Multu,
    Div,
    Divu,
    Add = 32,
    Addu,
    Sub,
    Subu,
    And,
    Or,
    Xor,
    Nor,
    Slt = 42,
    Sltu,
    Tge = 48,
    Tgeu,
    Tlt,
    Tltu,
    Teq = 54,
}

#[derive(Debug)]
pub struct Data<'a> {
    pub label: Option<&'a str>,
    pub constant: Constant<'a>,
}

#[derive(Debug)]
pub enum Constant<'a> {
    Bytes(Vec<i8>),
    Halves(Vec<i16>),
    Words(Vec<i32>),
    Space(usize),
    String { contents: Cow<'a, str>, z: bool },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    Zero,
    At,
    V0,
    V1,
    A0,
    A1,
    A2,
    A3,
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    T8,
    T9,
    K0,
    K1,
    Gp,
    Sp,
    Fp,
    Ra,
}

#[derive(Error, Debug)]
pub enum RegisterParseError {
    #[error("Expected \"$\" prefix")]
    NoPrefix,
    #[error("Invalid register \"{0}\"")]
    InvalidRegister(String),
}

impl FromStr for Register {
    type Err = RegisterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix('$').ok_or(RegisterParseError::NoPrefix)? {
            "zero" => Ok(Self::Zero),
            "at" => Ok(Self::At),
            "v0" => Ok(Self::V0),
            "v1" => Ok(Self::V1),
            "a0" => Ok(Self::A0),
            "a1" => Ok(Self::A1),
            "a2" => Ok(Self::A2),
            "a3" => Ok(Self::A3),
            "t0" => Ok(Self::T0),
            "t1" => Ok(Self::T1),
            "t2" => Ok(Self::T2),
            "t3" => Ok(Self::T3),
            "t4" => Ok(Self::T4),
            "t5" => Ok(Self::T5),
            "t6" => Ok(Self::T6),
            "t7" => Ok(Self::T7),
            "s0" => Ok(Self::S0),
            "s1" => Ok(Self::S1),
            "s2" => Ok(Self::S2),
            "s3" => Ok(Self::S3),
            "s4" => Ok(Self::S4),
            "s5" => Ok(Self::S5),
            "s6" => Ok(Self::S6),
            "s7" => Ok(Self::S7),
            "t8" => Ok(Self::T8),
            "t9" => Ok(Self::T9),
            "k0" => Ok(Self::K0),
            "k1" => Ok(Self::K1),
            "gp" => Ok(Self::Gp),
            "sp" => Ok(Self::Sp),
            "fp" => Ok(Self::Fp),
            "ra" => Ok(Self::Ra),
            _ => Err(RegisterParseError::InvalidRegister(s.to_string())),
        }
    }
}
