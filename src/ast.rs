use std::{borrow::Cow, str::FromStr};
use thiserror::Error;

#[derive(Debug)]
pub struct Program<'a> {
    pub data: Vec<Data<'a>>,
    pub text: Vec<(Option<&'a str>, Instruction<'a>)>,
}

#[derive(Debug, Clone)]
pub enum Instruction<'a> {
    Binop {
        op: Binop,
        dst: Register,
        lhs: Register,
        rhs: Value,
    },
    Unop {
        op: Unop,
        dst: Register,
        src: Value,
    },
    MoveHiLo {
        reg: Register,
        hi: bool,
        to: bool,
    },
    La {
        dst: Register,
        addr: Address<'a>,
    },
    Mem {
        op: MemOp,
        reg: Register,
        offset: Address<'a>,
        addr: Option<Register>,
        width: Width,
    },
    Branch {
        cond: Condition,
        target: Address<'a>,
        link: bool,
    },
    Jump {
        target: Value<Address<'a>>,
        link: bool,
    },
    Trap {
        cond: BinCond,
        lhs: Register,
        rhs: Value,
    },
    Syscall,
    Nop,
    Break,
}

#[derive(Debug, Clone)]
pub enum MemOp {
    Load,
    Store,
}

#[derive(Debug, Clone)]
pub enum Width {
    Byte,
    ByteUnaligned,
    Half,
    HalfUnaligned,
    Word,
    WordLeft,
    WordRight,
    Double,
}

#[derive(Debug, Clone)]
pub enum Address<'a, L = usize> {
    Label(&'a str),
    Literal(L),
}

#[derive(Debug, Clone)]
pub enum Condition {
    Binary {
        cond: BinCond,
        lhs: Register,
        rhs: Value,
    },
    Unary {
        cond: UnCond,
        src: Value,
    },
}

#[derive(Debug, Clone)]
pub enum UnCond {
    Gez,
    Gtz,
    Lez,
    Ltz,
    Eqz,
    Nez,
}

#[derive(Debug, Clone)]
pub enum BinCond {
    Eq,
    Ne,
    Ge,
    Geu,
    Gt,
    Gtu,
    Le,
    Leu,
    Lt,
    Ltu,
}

#[derive(Debug, Clone)]
pub enum Unop {
    Abs,
    Clo,
    Clz,
    Move,
    Neg,
    Negu,
    Not,
    Lui,
    Li,
}

#[derive(Debug, Clone)]
pub enum Binop {
    Add,
    Addu,
    Addi,
    Addiu,
    And,
    Andi,
    Div,
    Divu,
    Movn,
    Movz,
    Mult,
    Multu,
    Mul,
    Mulo,
    Mulou,
    Madd,
    Maddu,
    Msub,
    Msubu,
    Nor,
    Or,
    Ori,
    Rem,
    Remu,
    Sll,
    Sllv,
    Sra,
    Srav,
    Srl,
    Srlv,
    Rol,
    Ror,
    Sub,
    Subu,
    Xor,
    Xori,
    Slt,
    Sltu,
    Slti,
    Sltiu,
    Seq,
    Sge,
    Sgeu,
    Sgt,
    Sgtu,
    Sle,
    Sleu,
    Sne,
}

#[derive(Debug, Clone)]
pub enum Value<C = i32, D = Register> {
    Constant(C),
    Dynamic(D),
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
