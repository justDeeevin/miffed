use std::{borrow::Cow, str::FromStr};
use thiserror::Error;

#[derive(Debug)]
pub struct Program<'a> {
    pub data: Vec<Data<'a>>,
    pub text: Vec<(Option<&'a str>, Instruction<'a>)>,
    pub delay_slot: bool,
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
        index: Index<'a>,
    },
    Mem {
        op: MemOp,
        reg: Register,
        index: Index<'a>,
        width: Width,
    },
    Branch {
        cond: Condition,
        target: Address<'a>,
    },
    Jump {
        target: Value<Address<'a>>,
        link: Option<Register>,
    },
    Trap {
        cond: TrapCond,
        lhs: Register,
        rhs: Value,
    },
    Syscall,
    Nop,
    Break(i32),
}

#[derive(Debug, Clone)]
pub struct Index<'a> {
    pub offset: Address<'a>,
    pub addr: Option<Register>,
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum TrapCond {
    Ge,
    Geu,
    Lt,
    Ltu,
    Eq,
    Ne = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemOp {
    Load,
    Store,
}

#[derive(Debug, Clone, Copy)]
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
        src: Register,
        link: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum UnCond {
    Gez,
    Gtz,
    Lez,
    Ltz,
    Eqz,
    Nez,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum Unop {
    Abs,
    Clo,
    Clz,
    Div,
    Divu,
    Madd,
    Maddu,
    Move,
    Msub,
    Msubu,
    Mult,
    Multu,
    Neg,
    Negu,
    Not,
    Lui,
    Li,
}

#[derive(Debug, Clone, Copy)]
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
    Mul,
    Mulo,
    Mulou,
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
pub enum Value<I = i32, D = Register> {
    Immediate(I),
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

impl Constant<'_> {
    pub fn size(&self) -> usize {
        match self {
            Self::Bytes(bytes) => bytes.len(),
            Self::Halves(halves) => halves.len() * 2,
            Self::Words(words) => words.len() * 4,
            Self::Space(size) => *size,
            Self::String { contents, z } => contents.len() + *z as usize,
        }
    }

    pub fn bytes(self) -> Vec<u8> {
        match self {
            Self::Bytes(bytes) => bytes.into_iter().map(|b| b as u8).collect(),
            Self::Halves(halves) => halves.into_iter().flat_map(|h| h.to_ne_bytes()).collect(),
            Self::Words(words) => words.into_iter().flat_map(|w| w.to_ne_bytes()).collect(),
            Self::Space(size) => vec![0; size],
            Self::String { contents, z } => {
                let mut out = contents.into_owned().into_bytes();
                if z {
                    out.push(0);
                }
                out
            }
        }
    }
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
    #[error("$at is reserved")]
    AtUsed,
}

impl FromStr for Register {
    type Err = RegisterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix('$').ok_or(RegisterParseError::NoPrefix)? {
            "zero" => Ok(Self::Zero),
            "at" => Err(RegisterParseError::AtUsed),
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
