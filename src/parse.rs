use crate::{
    ast::{Constant, Data, Instruction, Program, Statement},
    lex::Token,
};
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
        .then(parse_text_section().collect())
        .then_ignore(newlines())
        .map(|(data, text)| Program { data, text });

    parser.parse(token_stream)
}

fn parse_data_section<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Vec<Data<'a>>, Extra<'a>> {
    just(Token::Ident(".data"))
        .labelled("data directive")
        .ignore_then(newlines().at_least(1))
        .ignore_then(
            parse_label(Some("data"))
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

fn parse_label<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>(
    label_kind: Option<&str>,
) -> impl Parser<'a, I, &'a str, Extra<'a>> {
    select!(Token::Ident(i) => i)
        .labelled("identifier")
        .then_ignore(just(Token::Colon))
        .labelled(format!(
            "{}label",
            label_kind.map(|k| k.to_string() + " ").unwrap_or_default()
        ))
}

fn parse_text_section<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl IterParser<'a, I, Statement<'a>, Extra<'a>> {
    just(Token::Ident(".text"))
        .ignore_then(newlines().at_least(1))
        .ignore_then(
            select!(Token::Ident(i) => i)
                .labelled("identifier")
                .then_ignore(just(Token::Colon))
                .map(Statement::Label)
                .or(parse_instruction().map(Statement::Instruction)),
        )
        .separated_by(newlines().at_least(1))
}

fn parse_instruction<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> impl Parser<'a, I, Instruction, Extra<'a>> {
    todo()
}

fn newlines<'a, I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>>()
-> Repeated<impl Parser<'a, I, (), Extra<'a>>, (), I, Extra<'a>> {
    just(Token::Newline).ignored().repeated()
}
