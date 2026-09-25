use crate::ast::{Register, RegisterParseError};
#[cfg(doc)]
use Token::*;
use logos::{Lexer, Logos};
use strum::Display;

#[derive(Default, Clone, PartialEq, Debug)]
pub enum LexError {
    UnknownRegister,
    #[default]
    UnknownToken,
}

impl From<RegisterParseError> for LexError {
    fn from(_value: RegisterParseError) -> Self {
        LexError::UnknownRegister
    }
}

#[derive(Logos, Debug, Clone, PartialEq, Display)]
#[logos(skip(r"[ \t\v]+"))]
#[logos(skip("#.*", allow_greedy = true))]
#[logos(error = LexError)]
/// Math operations are signed by default and will crash on overflow. Unsigned variants (denoted
/// with a "u" prefix) will wrap around on overflow.
pub enum Token<'a> {
    // --- Symbols ---
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[regex(r"[\r\n\f]")]
    Newline,
    // ---------------

    // --- Operations ---
    #[token("abs")]
    /// Peudo-instruction expansion:
    /// ```asm
    /// addu $1, $2, $zero # Expansion of move
    /// bgez $2, 8 # skip sub
    /// sub $1, $zero, $2
    /// ```
    Abs,
    #[token("add")]
    Add,
    #[token("addu")]
    Addu,
    #[token("addi")]
    Addi,
    #[token("addiu")]
    Addiu,
    #[token("and")]
    And,
    #[token("andi")]
    Andi,
    #[token("clo")]
    /// Count leading ones
    Clo,
    #[token("clz")]
    /// Count leading zeros
    Clz,
    #[token("div")]
    /// Three-operand form pseudo-instruction expansion:
    /// ```asm
    /// tnei $3, zero
    /// div $2, $3
    /// mflo $1
    /// ```
    Div,
    #[token("divu")]
    /// See [`Div`] for the three-operand form pseudo-instruction expansion
    Divu,
    #[token("mult")]
    /// Double-word
    Mult,
    #[token("multu")]
    /// Double-word
    Multu,
    #[token("mul")]
    /// **No overflow**
    Mul,
    #[token("mulo")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// mult $2, $3
    /// mflo $1
    /// mfhi $at
    /// sra $1, $1, 31
    /// teq $1, $at
    /// mflo $1
    /// ```
    Mulo,
    #[token("mulou")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// multu $2, $3
    /// mflo $1
    /// mfhi $at
    /// teq $at, $zero
    /// ```
    Mulou,
    #[token("madd")]
    Madd,
    #[token("maddu")]
    Maddu,
    #[token("msub")]
    Msub,
    #[token("msubu")]
    Msubu,
    #[token("neg")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// sub $1, $zero, $2
    /// ```
    Neg,
    #[token("negu")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// subu $1, $zero, $2
    /// ```
    Negu,
    #[token("nor")]
    Nor,
    #[token("not")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// nor $1, $2, $zero
    /// ```
    Not,
    #[token("or")]
    Or,
    #[token("ori")]
    Ori,
    #[token("rem")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// div $2, $3
    /// mfhi $1
    /// ```
    Rem,
    #[token("remu")]
    /// See [`Rem`] for pseudo-instruction expansion
    Remu,
    #[token("sll")]
    Sll,
    #[token("sllv")]
    Sllv,
    #[token("sra")]
    Sra,
    #[token("srav")]
    Srav,
    #[token("srl")]
    Srl,
    #[token("srlv")]
    Srlv,
    #[token("rol")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// # immediate shamt
    /// srl $at, $2, (32 - shamt)
    /// sll $1, $2, shamt
    /// or $1, $1, $at
    ///
    /// # register shamt
    /// subu $at, $zero, $3
    /// slrv $at, $2, $at
    /// sllv $1, $2, $3
    /// or $1, $1, $at
    /// ```
    Rol,
    #[token("ror")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// # immediate shamt
    /// srl $at, $2, shamt
    /// sll $1, $2, (32 - shamt)
    /// or $1, $1, $at
    ///
    /// # register shamt
    /// subu $at, $zero, $3
    /// slrv $1, $2, $3
    /// sllv $at, $2, $at
    /// or $1, $1, $at
    /// ```
    Ror,
    #[token("sub")]
    Sub,
    #[token("subu")]
    Subu,
    #[token("xor")]
    Xor,
    #[token("xori")]
    Xori,

    #[token("lui")]
    Lui,
    #[token("li")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// lui $1, (imm >> 16)
    /// ori $1, $1, (imm & 0xffff)
    /// ```
    Li,

    #[token("slt")]
    Slt,
    #[token("sltu")]
    Sltu,
    #[token("slti")]
    Slti,
    #[token("sltiu")]
    Sltiu,
    #[token("seq")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// subu $1, $2, $3
    /// ori $at, $zero, 1
    /// sltu $1, $1, $at
    /// ```
    Seq,
    #[token("sge")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// slt $1, $2, $3
    /// xori $1, $1, 1
    /// ```
    Sge,
    #[token("sgeu")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// sltu $1, $2, $3
    /// xori $1, $1, 1
    /// ```
    Sgeu,
    #[token("sgt")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// slt $1, $3, $2
    /// ```
    Sgt,
    #[token("sgtu")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// sltu $1, $3, $2
    /// ```
    Sgtu,
    #[token("sle")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// slt $at, $3, $2
    /// ori $1, $zero, 1
    /// subu $1, $1, $at
    /// ```
    Sle,
    #[token("sleu")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// sltu $at, $3, $2
    /// xori $1, $at, 1
    /// ```
    Sleu,
    #[token("sne")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// subu $at, $2, $3
    /// sltu $1, $zero, $at
    /// ```
    Sne,

    #[token("b")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// beq $zero, $zero, offset
    /// ```
    B,
    #[token("beq")]
    Beq,
    #[token("bgez")]
    Bgez,
    #[token("bgezal")]
    Bgezal,
    #[token("bgtz")]
    Bgtz,
    #[token("blez")]
    Blez,
    #[token("bltzal")]
    Bltzal,
    #[token("bltz")]
    Bltz,
    #[token("bne")]
    Bne,
    #[token("beqz")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// beq $1, $zero, offset
    /// ```
    Beqz,
    #[token("bge")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// slt $at, $1, $2
    /// beq $at, $zero, offset
    /// ```
    Bge,
    #[token("bgeu")]
    /// See [`Bge`] for pseudo-instruction expansion
    Bgeu,
    #[token("bgt")]
    /// Pseeudo-instruction expansion:
    /// ```asm
    /// slt $at, $1, $2
    /// bne $at, $zero, offset
    /// ```
    Bgt,
    #[token("bgtu")]
    /// See [`Bgt`] for pseudo-instruction expansion
    Bgtu,
    #[token("ble")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// slt $at, $2, $1
    /// beq $at, $zero, offset
    /// ```
    Ble,
    #[token("bleu")]
    /// See [`Ble`] for pseudo-instruction expansion
    Bleu,
    #[token("blt")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// slt $at, $2, $1
    /// bne $at, $zero, offset
    /// ```
    Blt,
    #[token("bltu")]
    /// See [`Blt`] for pseudo-instruction expansion
    Bltu,
    #[token("bnez")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// bne $1, $zero, offset
    /// ```
    Bnez,

    #[token("j")]
    J,
    #[token("jal")]
    Jal,
    #[token("jalr")]
    Jalr,
    #[token("jr")]
    Jr,

    #[token("teq")]
    Teq,
    #[token("teqi")]
    Teqi,
    #[token("tne")]
    Tne,
    #[token("tnei")]
    Tnei,
    #[token("tge")]
    Tge,
    #[token("tgeu")]
    Tgeu,
    #[token("tgei")]
    Tgei,
    #[token("tgeiu")]
    Tgeiu,
    #[token("tlt")]
    Tlt,
    #[token("tltu")]
    Tltu,
    #[token("tlti")]
    Tlti,
    #[token("tltiu")]
    Tltiu,

    #[token("la")]
    /// See [`Li`] for pseudo-instruction expansion
    La,
    #[token("lb")]
    Lb,
    #[token("lbu")]
    Lbu,
    #[token("lh")]
    Lh,
    #[token("lhu")]
    Lhu,
    #[token("lw")]
    Lw,
    #[token("lwl")]
    Lwl,
    #[token("lwr")]
    Lwr,
    #[token("ld")]
    /// Pseudo-instruction
    Ld,
    // Skipped: ulh, ulhu, ulw, ll
    #[token("sb")]
    Sb,
    #[token("sh")]
    Sh,
    #[token("sw")]
    Sw,
    #[token("swl")]
    Swl,
    #[token("swr")]
    Swr,
    #[token("sd")]
    /// Pseudo-instruction
    Sd,
    // Skipped: ush, usw, sc
    #[token("move")]
    /// Pseudo-instruction expansion:
    /// ```asm
    /// addu $1, $2, $zero
    /// ```
    Move,
    #[token("mfhi")]
    Mfhi,
    #[token("mflo")]
    Mflo,
    #[token("mthi")]
    Mthi,
    #[token("mtlo")]
    Mtlo,
    #[token("movn")]
    Movn,
    #[token("movz")]
    Movz,

    #[token("syscall")]
    Syscall,
    #[token("break")]
    Break,
    #[token("nop")]
    Nop,
    // ------------------

    // --- Etc ---
    #[regex(r"-?([0-9]+|0x[0-9a-fA-F]+)", parse_number)]
    Number(i32),

    #[regex(r"\$[0-9a-zA-Z]+", |lex| lex.slice().parse::<Register>())]
    Register(Register),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().strip_circumfix('"', '"').unwrap())]
    String(&'a str),

    #[regex(r"[a-zA-Z_.][0-9a-zA-Z_.]*", Lexer::slice, priority = 1)]
    Ident(&'a str),

    Error(LexError),
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
