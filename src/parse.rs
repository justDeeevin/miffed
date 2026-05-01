use crate::{
    ast::{
        Addr, Address, BinCond, Binop, Condition, Index, Instruction, MemOp, Program, Register,
        UnaryCond, Unaryop, Value,
    },
    lex::Token,
};
use chumsky::{
    IterParser, ParseResult, Parser,
    error::Rich,
    extra::Err,
    input::{Input, Stream, ValueInput},
    primitive::{choice, just, none_of, one_of},
    select,
    span::SimpleSpan,
};
use logos::Logos;
use std::collections::HashMap;

type Extra<'a> = Err<Rich<'a, Token<'a>>>;

pub fn parse_program(input: &str) -> ParseResult<Program<'_>, Rich<'_, Token<'_>>> {
    let lexer = Token::lexer(input)
        .spanned()
        .map(|(token, span)| (token.unwrap_or(Token::Error), SimpleSpan::from(span)));

    let token_stream = Stream::from_iter(lexer).map((0..input.len()).into(), |t| t);

    let parser = just(Token::Newline)
        .or_not()
        .ignore_then(parse_data_section())
        .then_ignore(just(Token::Newline))
        .then(parse_text_section())
        .then_ignore(just(Token::Newline).or_not())
        .map(|(data, text)| Program { data, text });

    #[cfg(feature = "debug")]
    let _ = std::fs::write("parser.svg", parser.debug().to_railroad_svg().to_string());

    parser.parse(token_stream)
}

fn parse_data_section<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, HashMap<&'a str, Vec<u8>>, Extra<'a>> {
    just(Token::Dot)
        .ignore_then(just(Token::Ident("data")))
        .labelled("data directive")
        .ignore_then(just(Token::Newline))
        .ignore_then(
            parse_label(Some("data"))
                .then(parse_data())
                .separated_by(just(Token::Newline))
                .collect(),
        )
        .labelled("data section")
}

fn parse_label<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>(
    label_kind: Option<&'a str>,
) -> impl Parser<'a, I, &'a str, Extra<'a>> {
    select!(Token::Ident(i) => i)
        .labelled("identifier")
        .then_ignore(just(Token::Colon))
        .labelled(format!(
            "{}label",
            label_kind.map(|k| k.to_string() + " ").unwrap_or_default()
        ))
}

fn parse_data<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Vec<u8>, Extra<'a>> {
    just(Token::Dot)
        .ignore_then(choice((
            just(Token::Ident("word"))
                .labelled("word directive")
                .ignore_then(
                    parse_number()
                        .map(i32::to_le_bytes)
                        .repeated()
                        .fold(Vec::new(), |mut acc, bytes| {
                            acc.extend(bytes);
                            acc
                        })
                        .labelled("words"),
                ),
            just(Token::Ident("asciiz"))
                .labelled("asciiz directive")
                .ignore_then(select!(Token::String(s) => parse_string().parse(s).unwrap())),
        )))
        .repeated()
        .fold(Vec::new(), |mut acc, bytes| {
            acc.extend(bytes);
            acc
        })
        .labelled("data")
}

fn parse_text_section<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Vec<(Option<&'a str>, Instruction<'a>)>, Extra<'a>> {
    just(Token::Dot)
        .ignore_then(just(Token::Ident("text")))
        .labelled("text directive")
        .ignore_then(just(Token::Newline))
        .ignore_then(
            parse_label(Some("text"))
                .or_not()
                .then(parse_instruction())
                .separated_by(just(Token::Newline))
                .collect()
                .labelled("text"),
        )
        .labelled("text section")
}

fn parse_instruction<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Instruction<'a>, Extra<'a>> {
    use Token::*;
    choice((
        select! {
            Add => Binop::Add,
            Sub=> Binop::Sub,
            Mul => Binop::Mul,
            Div => Binop::Div,
            Rem => Binop::Rem,
            And => Binop::And,
            Or => Binop::Or,
            Sll => Binop::Sll,
            Slr => Binop::Slr,
        }
        .labelled("binary operator")
        .then(parse_register())
        .then_ignore(just(Token::Comma))
        .then(parse_register())
        .then_ignore(just(Token::Comma))
        .then(parse_value())
        .map(|(((op, dst), lhs), rhs)| Instruction::Binop { op, dst, lhs, rhs }),
        select! {
            Not => Unaryop::Not,
            Abs => Unaryop::Abs,
            Neg => Unaryop::Neg,
        }
        .labelled("unary operator")
        .then(parse_register())
        .then_ignore(just(Token::Comma))
        .then(parse_register())
        .map(|((op, dst), rhs)| Instruction::Unaryop { op, dst, rhs }),
        select! {
            Beq => BinCond::Eq,
            Bne => BinCond::Ne,
            Bge => BinCond::Ge,
            Bgt => BinCond::Gt,
            Ble => BinCond::Le,
            Blt => BinCond::Lt,
        }
        .labelled("binary condition")
        .then(parse_register())
        .then_ignore(just(Token::Comma))
        .then(parse_value())
        .map(|((cond, lhs), rhs)| Condition::Binary { cond, lhs, rhs })
        .or(select!(
            Beqz => UnaryCond::Eqz,
            Bnez => UnaryCond::Nez,
            Bgez => UnaryCond::Gez,
            Bgtz => UnaryCond::Gtz,
            Blez => UnaryCond::Lez,
            Bltz => UnaryCond::Ltz,
        )
        .labelled("unary condition")
        .then(parse_register())
        .then_ignore(just(Token::Comma))
        .map(|(cond, rhs)| Condition::Unary { cond, rhs }))
        .or(just(Token::Jmp).to(Condition::Always))
        .then(parse_address())
        .map(|(cond, dst)| Instruction::Branch { cond, dst }),
        just(Token::Move)
            .ignore_then(parse_register())
            .then_ignore(just(Token::Comma))
            .then(parse_register())
            .map(|(dst, src)| Instruction::Move { dst, src }),
        just(Token::Li)
            .ignore_then(parse_register())
            .then_ignore(just(Token::Comma))
            .then(parse_number())
            .map(|(dst, src)| Instruction::Li { dst, src }),
        select! {
            Token::Lw => MemOp::Load,
            Token::Sw => MemOp::Store,
            Token::La => MemOp::LoadAddr
        }
        .labelled("load/store operation")
        .then(parse_register())
        .then_ignore(just(Token::Comma))
        .then(
            parse_address().map(Value::Const).or(parse_number()
                .or_not()
                .then_ignore(just(Token::LParen))
                .then(parse_register())
                .then_ignore(just(Token::RParen))
                .map(|(offset, addr)| {
                    Value::Dynamic(Index {
                        offset: offset.unwrap_or_default() as Addr,
                        addr,
                    })
                })),
        )
        .map(|((op, reg), addr)| Instruction::Mem { op, reg, addr }),
        just(Token::Syscall).to(Instruction::Syscall),
        just(Token::Nop).to(Instruction::Nop),
    ))
    .labelled("instruction")
}

fn parse_address<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Address<'a>, Extra<'a>> {
    select!(
        Token::Number(n) => Address::Literal(n as usize),
        Token::Ident(i) => Address::Label(i)
    )
    .labelled("address")
}

fn parse_number<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, i32, Extra<'a>> {
    select!(Token::Number(n) => n).labelled("number")
}

fn parse_value<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Value, Extra<'a>> {
    select!(Token::Number(n) => Value::Const(n), Token::Register(r) => Value::Dynamic(r))
        .labelled("value")
}

fn parse_register<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Register, Extra<'a>> {
    select!(Token::Register(r) => r).labelled("register")
}

fn parse_string<'a>() -> impl Parser<'a, &'a str, Vec<u8>> {
    none_of("\\")
        .repeated()
        .collect::<String>()
        .then_ignore(just('\\'))
        .then(
            choice((
                select!(
                    'a' => b'\x07',
                    'b' => b'\x08',
                    'f' => b'\x0c',
                    'n' => b'\n',
                    'r' => b'\r',
                    't' => b'\t',
                    'v' => b'\x0b',
                ),
                one_of("01234567")
                    .repeated()
                    .at_least(1)
                    .at_most(3)
                    .collect::<String>()
                    .map(|c| c.parse().unwrap()),
                just('x').ignore_then(
                    one_of("0123456789abcdefABCDEF")
                        .repeated()
                        .at_least(1)
                        .collect::<String>()
                        .map(|s| u8::from_str_radix(&s, 16).unwrap()),
                ),
            ))
            .labelled("escape sequence"),
        )
        .repeated()
        .fold(Vec::new(), |mut acc, (string, byte)| {
            acc.extend(string.bytes());
            acc.push(byte);
            acc
        })
        .map(|mut bytes| {
            bytes.push(0);
            bytes
        })
        .labelled("string")
}
