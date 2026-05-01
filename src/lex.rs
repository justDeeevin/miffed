use crate::ast::Register;
use logos::{Lexer, Logos};

#[derive(Logos, Clone, PartialEq, Eq, Debug)]
#[logos(skip(r"[ \t\v]+"))]
#[logos(skip(r"#.*", allow_greedy = true))]
pub enum Token<'a> {
    // --- Symbols ---
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[regex(r"[\r\n\f]+")]
    Newline,
    // ------

    // --- Instructions ---
    #[token("add")]
    Add,
    #[token("sub")]
    Sub,
    #[token("mul")]
    Mul,
    #[token("div")]
    Div,
    #[token("rem")]
    Rem,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("sll")]
    Sll,
    #[token("slr")]
    Slr,

    #[token("not")]
    Not,
    #[token("abs")]
    Abs,
    #[token("neg")]
    Neg,

    #[token("beq")]
    Beq,
    #[token("bne")]
    Bne,
    #[token("bge")]
    Bge,
    #[token("bgt")]
    Bgt,
    #[token("ble")]
    Ble,
    #[token("blt")]
    Blt,

    #[token("beqz")]
    Beqz,
    #[token("bnez")]
    Bnez,
    #[token("bgez")]
    Bgez,
    #[token("bgtz")]
    Bgtz,
    #[token("blez")]
    Blez,
    #[token("bltz")]
    Bltz,

    #[token("j")]
    Jmp,

    #[token("li")]
    Li,

    #[token("sw")]
    Sw,
    #[token("lw")]
    Lw,
    #[token("la")]
    La,

    #[token("syscall")]
    Syscall,

    #[token("nop")]
    Nop,

    #[token("move")]
    Move,
    // ------

    // --- Etc ---
    #[regex(r"-?([0-9]+|0x[0-9a-fA-F]+)", parse_number)]
    Number(i32),

    #[regex(r"\$(t[0-9]|a[0-3]|v0)", |lex| lex.slice().parse::<Register>().unwrap())]
    Register(Register),

    #[regex(r#""([^"\\]|\\([abfnrtv"\\]|[0-7]{1,3}|x[a-fA-F0-9]+))*""#, |lex| lex.slice().strip_prefix('"').unwrap().strip_suffix('"').unwrap())]
    String(&'a str),

    #[regex(r"[a-z]+", |lex| lex.slice(), priority = 1)]
    Ident(&'a str),

    Error,
}

fn parse_number<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> i32 {
    let mut slice = lexer.slice();
    let signum = if let Some(neg) = slice.strip_prefix('-') {
        slice = neg;
        -1
    } else {
        1
    };
    let radix = if let Some(hex) = slice.strip_prefix("0x") {
        slice = hex;
        16
    } else {
        10
    };

    signum * i32::from_str_radix(slice, radix).unwrap()
}
