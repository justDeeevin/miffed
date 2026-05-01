use std::collections::HashMap;

#[derive(Debug)]
pub struct Program<'a> {
    pub data: HashMap<&'a str, Vec<u8>>,
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
    Unaryop {
        op: Unaryop,
        dst: Register,
        rhs: Register,
    },
    Branch {
        cond: Condition,
        dst: Address<'a>,
    },
    Move {
        dst: Register,
        src: Register,
    },
    Li {
        dst: Register,
        src: i32,
    },
    Mem {
        op: MemOp,
        reg: Register,
        addr: Value<Address<'a>, Index>,
    },
    Syscall,
    Nop,
}

#[derive(Debug)]
pub enum Syscall {
    PrintInt = 1,
    PrintStr = 4,
    ReadInt,
    ReadString = 8,
    Exit = 10,
    PrintChar,
    ReadChar,
    Exit2 = 17,
}

#[derive(Debug, Clone, Copy)]
pub enum Binop {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Sll,
    Slr,
}

#[derive(Debug, Clone, Copy)]
pub enum Unaryop {
    Not,
    Abs,
    Neg,
}

#[derive(Debug, Clone)]
pub enum Value<L = i32, R = Register> {
    Const(L),
    Dynamic(R),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Register {
    T0 = 8,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8 = 24,
    T9,
    V0,
    A0,
    A1,
    A2,
    A3,
}

impl std::str::FromStr for Register {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix('$').ok_or(())? {
            "t0" => Ok(Self::T0),
            "t1" => Ok(Self::T1),
            "t2" => Ok(Self::T2),
            "t3" => Ok(Self::T3),
            "t4" => Ok(Self::T4),
            "t5" => Ok(Self::T5),
            "t6" => Ok(Self::T6),
            "t7" => Ok(Self::T7),
            "t8" => Ok(Self::T8),
            "t9" => Ok(Self::T9),
            "v0" => Ok(Self::V0),
            "a0" => Ok(Self::A0),
            "a1" => Ok(Self::A1),
            "a2" => Ok(Self::A2),
            "a3" => Ok(Self::A3),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Condition {
    Binary {
        cond: BinCond,
        lhs: Register,
        rhs: Value,
    },
    Unary {
        cond: UnaryCond,
        rhs: Register,
    },
    Always,
}

#[derive(Debug, Clone, Copy)]
pub enum BinCond {
    Eq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryCond {
    Eqz,
    Nez,
    Gez,
    Gtz,
    Lez,
    Ltz,
}

#[derive(Debug, Clone, Copy)]
pub enum MemOp {
    Load,
    Store,
    LoadAddr,
}

pub type Addr = usize;

#[derive(Debug, Clone)]
pub enum Address<'a> {
    Literal(Addr),
    Label(&'a str),
}

#[derive(Debug, Clone)]
pub struct Index {
    pub offset: Addr,
    pub addr: Register,
}
