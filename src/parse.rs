use crate::{ast::*, lex::Token};
use chumsky::{
    combinator::Repeated,
    extra::Err,
    input::{Stream, ValueInput},
    prelude::*,
};
use logos::Logos;
use std::{borrow::Cow, fmt::Display};

type Extra<'a> = Err<Rich<'a, Token<'a>>>;

pub fn parse_program(input: &str) -> ParseResult<Program<'_>, Rich<'_, Token<'_>>> {
    let lexer = Token::lexer(input)
        .spanned()
        .map(|(token, span)| (token.unwrap_or_else(Token::Error), SimpleSpan::from(span)));

    let token_stream = Stream::from_iter(lexer).map((input.len()..input.len()).into(), |t| t);

    let parser = newlines()
        .ignore_then(parse_data_section())
        .then_ignore(newlines())
        .then(parse_text_section())
        .then_ignore(newlines())
        .map(|(data, text)| Program { data, text });

    #[cfg(feature = "debug")]
    let _ = std::fs::write("parser.svg", parser.debug().to_railroad_svg().to_string());

    parser.parse(token_stream)
}

fn parse_data_section<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Vec<Data<'a>>, Extra<'a>> {
    just(Token::Ident(".data"))
        .ignore_then(newlines().at_least(1))
        .ignore_then(
            parse_label()
                .or_not()
                .then(parse_constant())
                .map(|(label, constant)| Data { label, constant })
                .separated_by(newlines().at_least(1))
                .collect(),
        )
        .labelled("data section")
}

fn parse_constant<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Constant<'a>, Extra<'a>> {
    choice((
        just(Token::Ident(".ascii")).ignore_then(parse_string(false)),
        just(Token::Ident(".asciiz")).ignore_then(parse_string(true)),
        just(Token::Ident(".byte")).ignore_then(parse_numbers(Constant::Bytes)),
        just(Token::Ident(".half")).ignore_then(parse_numbers(Constant::Halves)),
        just(Token::Ident(".word")).ignore_then(parse_numbers(Constant::Words)),
        just(Token::Ident(".space")).ignore_then(parse_number().map(Constant::Space)),
    ))
    .labelled("constant")
}

fn parse_string<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>(
    z: bool,
) -> impl Parser<'a, I, Constant<'a>, Extra<'a>> {
    select!(Token::String(s) => s)
        .labelled("string")
        .try_map(move |s, span: SimpleSpan| {
            let mut contents = Cow::Borrowed(s);
            let mut chars = s.char_indices();

            while let Some((i, c)) = chars.next() {
                if c == '\\' {
                    let string = match &mut contents {
                        Cow::Borrowed(s) => {
                            contents = Cow::Owned(String::with_capacity(s.len()) + &s[..i]);
                            contents.to_mut()
                        }
                        Cow::Owned(s) => s,
                    };

                    let escape = chars.next().unwrap().1;
                    let start = span.start() + 1;
                    match escape {
                        'n' => string.push('\n'),
                        't' => string.push('\t'),
                        'r' => string.push('\r'),
                        '\\' => string.push('\\'),
                        '"' => string.push('"'),
                        _ => {
                            return Err(Rich::custom(
                                (start + i
                                    ..start + chars.next().map(|(i, _)| i).unwrap_or(s.len()))
                                    .into(),
                                "Unknown escape character",
                            ));
                        }
                    }
                } else {
                    if let Cow::Owned(s) = &mut contents {
                        s.push(c);
                    }
                }
            }

            Ok(Constant::String { contents, z })
        })
}

fn parse_numbers<
    'a,
    N: TryFrom<i32, Error: Display>,
    I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>,
>(
    wrap: impl Fn(Vec<N>) -> Constant<'a>,
) -> impl Parser<'a, I, Constant<'a>, Extra<'a>> {
    parse_number()
        .separated_by(just(Token::Comma))
        .collect()
        .map(wrap)
        .labelled("comma-separated numbers")
}

fn parse_number<
    'a,
    N: TryFrom<i32, Error: Display>,
    I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>,
>() -> impl Parser<'a, I, N, Extra<'a>> {
    select!(Token::Number(n) => n)
        .labelled("number")
        .try_map(|n, span| N::try_from(n).map_err(|e| Rich::custom(span, e.to_string())))
}

fn parse_label<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, &'a str, Extra<'a>> {
    parse_identifier()
        .then_ignore(just(Token::Colon))
        .labelled("label")
}

fn parse_text_section<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Vec<(Option<&'a str>, Instruction<'a>)>, Extra<'a>> {
    just(Token::Ident(".text"))
        .ignore_then(newlines().at_least(1))
        .ignore_then(
            parse_label()
                .or_not()
                .then_ignore(newlines())
                .then(parse_instruction())
                .separated_by(newlines().at_least(1))
                .collect(),
        )
        .labelled("text section")
}

fn parse_instruction<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Instruction<'a>, Extra<'a>> {
    macro_rules! token_map {
        ($op:ident [$token1:ident $(, $token:ident)* $(,)?]) => {{
            match $op::$token1 {
                $op::$token1 | $($op::$token)|* => {}
            }
            select!(Token::$token1 => $op::$token1, $(Token::$token => $op::$token),*)
        }}
    }

    choice((
        token_map!(Binop [
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
        ])
        .labelled("binary operation")
        .then(parse_register())
        .then_ignore(just(Token::Comma))
        .then(parse_register())
        .then_ignore(just(Token::Comma))
        .then(parse_value())
        .map(|(((op, dst), lhs), rhs)| Instruction::Binop { op, dst, lhs, rhs }),
        token_map!(Unop [
            Abs,
            Clo,
            Clz,
            Move,
            Neg,
            Negu,
            Not,
            Lui,
            Li,
        ])
        .labelled("unary operation")
        .then(parse_register())
        .then_ignore(just(Token::Comma))
        .then(parse_value())
        .map(|((op, dst), src)| Instruction::Unop { op, dst, src }),
        select! {
            Token::Mfhi => (true, false),
            Token::Mflo => (false, false),
            Token::Mthi => (true, true),
            Token::Mtlo => (true, false)
        }
        .labelled("move to/from hi/lo")
        .then(parse_register())
        .map(|((hi, to), reg)| Instruction::MoveHiLo { reg, hi, to }),
        just(Token::La)
            .ignore_then(parse_register())
            .then_ignore(just(Token::Comma))
            .then(parse_address())
            .map(|(dst, addr)| Instruction::La { dst, addr }),
        select! {
            Token::Lb => Width::Byte,
            Token::Lbu => Width::ByteUnaligned,
            Token::Lh => Width::Half,
            Token::Lhu => Width::HalfUnaligned,
            Token::Lw => Width::Word,
            Token::Lwl => Width::WordLeft,
            Token::Lwr => Width::WordRight,
            Token::Ld => Width::Double,
        }
        .labelled("load operation")
        .map(|width| (MemOp::Load, width))
        .or(select! {
            Token::Sb => Width::Byte,
            Token::Sh => Width::Half,
            Token::Sw => Width::Word,
            Token::Sd => Width::Double,
        }
        .labelled("store operation")
        .map(|width| (MemOp::Store, width)))
        .then(parse_register())
        .then_ignore(just(Token::Comma))
        .then(parse_address())
        .then(
            just(Token::LParen)
                .ignore_then(parse_register())
                .then_ignore(just(Token::RParen))
                .or_not(),
        )
        .map(|((((op, width), reg), offset), addr)| Instruction::Mem {
            op,
            reg,
            offset,
            addr,
            width,
        }),
        select! {
            Token::Beq => BinCond::Eq,
            Token::Bne => BinCond::Ne,
            Token::Bge => BinCond::Ge,
            Token::Bgeu => BinCond::Geu,
            Token::Bgt => BinCond::Gt,
            Token::Bgtu => BinCond::Gtu,
            Token::Ble => BinCond::Le,
            Token::Bleu => BinCond::Leu,
            Token::Blt => BinCond::Lt,
            Token::Bltu => BinCond::Ltu,
        }
        .labelled("binary branch operation")
        .then(parse_register())
        .then_ignore(just(Token::Comma))
        .then(parse_value())
        .map(|((cond, lhs), rhs)| (Condition::Binary { cond, lhs, rhs }, false))
        .or(select! {
            Token::Bgez => UnCond::Gez,
            Token::Bgtz => UnCond::Gtz,
            Token::Blez => UnCond::Lez,
            Token::Bltz => UnCond::Ltz,
            Token::Beqz => UnCond::Eqz,
            Token::Bnez => UnCond::Nez,
        }
        .labelled("unary branch operation")
        .map(|cond| (false, cond))
        .or(select! {
            Token::Bgezal => UnCond::Gez,
            Token::Bltzal => UnCond::Ltz,
        }
        .labelled("linking unary branch operation")
        .map(|cond| (true, cond)))
        .then(parse_value())
        .map(|((link, cond), src)| (Condition::Unary { cond, src }, link)))
        .then_ignore(just(Token::Comma))
        .then(parse_address())
        .map(|((cond, link), target)| Instruction::Branch { cond, target, link }),
        select!(Token::J => false, Token::Jal => true)
            .labelled("static jump operation")
            .then(parse_address().map(Value::Constant))
            .or(select!(Token::Jr => false, Token::Jalr => true)
                .labelled("dynamic jump operation")
                .then(parse_register().map(Value::Dynamic)))
            .map(|(link, target)| Instruction::Jump { target, link }),
        just(Token::Syscall).to(Instruction::Syscall),
        just(Token::Nop).to(Instruction::Nop),
        just(Token::Break).to(Instruction::Break),
        // TODO: trap
    ))
    .labelled("instruction")
}

fn parse_address<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Address<'a>, Extra<'a>> {
    parse_number()
        .map(Address::Literal)
        .or(parse_identifier().map(Address::Label))
        .labelled("address")
}

fn parse_register<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Register, Extra<'a>> {
    select!(Token::Register(reg) => reg).labelled("register")
}

fn parse_value<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Value, Extra<'a>> {
    select!(Token::Number(n) => Value::Constant(n))
        .labelled("number")
        .or(parse_register().map(Value::Dynamic))
        .labelled("value")
}

fn parse_identifier<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, &'a str, Extra<'a>> {
    select!(Token::Ident(i) => i).labelled("identifier")
}

fn newlines<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> Repeated<impl Parser<'a, I, (), Extra<'a>>, (), I, Extra<'a>> {
    just(Token::Newline).ignored().repeated()
}
