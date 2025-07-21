use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0},
    combinator::{map, recognize},
    multi::{many0, many1},
    sequence::{delimited, pair, preceded},
    IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Import,
    Loop,
    If,
    Else,
    
    // Identifiers and literals
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    
    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Equals,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    
    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Dot,
    Assignment,
    
    // Special
    Newline,
    Eof,
}

pub fn tokenize(input: &str) -> IResult<&str, Vec<Token>> {
    many0(preceded(multispace0, token))(input)
}

fn token(input: &str) -> IResult<&str, Token> {
    alt((
        keyword,
        boolean,
        float,
        integer,
        string_literal,
        identifier,
        operator,
        punctuation,
    ))(input)
}

fn keyword(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("import"), |_| Token::Import),
        map(tag("loop"), |_| Token::Loop),
        map(tag("if"), |_| Token::If),
        map(tag("else"), |_| Token::Else),
    ))(input)
}

fn boolean(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("true"), |_| Token::Boolean(true)),
        map(tag("false"), |_| Token::Boolean(false)),
    ))(input)
}

fn integer(input: &str) -> IResult<&str, Token> {
    map(
        recognize(many1(nom::character::complete::digit1)),
        |s: &str| Token::Integer(s.parse().unwrap()),
    )(input)
}

fn float(input: &str) -> IResult<&str, Token> {
    map(
        recognize(pair(
            many1(nom::character::complete::digit1),
            pair(char('.'), many1(nom::character::complete::digit1)),
        )),
        |s: &str| Token::Float(s.parse().unwrap()),
    )(input)
}

fn string_literal(input: &str) -> IResult<&str, Token> {
    map(
        delimited(
            char('"'),
            take_while1(|c| c != '"'),
            char('"'),
        ),
        |s: &str| Token::String(s.to_string()),
    )(input)
}

fn identifier(input: &str) -> IResult<&str, Token> {
    map(
        recognize(pair(
            alpha1,
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| Token::Identifier(s.to_string()),
    )(input)
}

fn operator(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("=="), |_| Token::Equals),
        map(tag("!="), |_| Token::NotEqual),
        map(tag("<="), |_| Token::LessThanOrEqual),
        map(tag(">="), |_| Token::GreaterThanOrEqual),
        map(tag("<"), |_| Token::LessThan),
        map(tag(">"), |_| Token::GreaterThan),
        map(tag("+"), |_| Token::Plus),
        map(tag("-"), |_| Token::Minus),
        map(tag("*"), |_| Token::Multiply),
        map(tag("/"), |_| Token::Divide),
        map(tag("="), |_| Token::Assignment),
    ))(input)
}

fn punctuation(input: &str) -> IResult<&str, Token> {
    alt((
        map(char('('), |_| Token::LeftParen),
        map(char(')'), |_| Token::RightParen),
        map(char('{'), |_| Token::LeftBrace),
        map(char('}'), |_| Token::RightBrace),
        map(char(','), |_| Token::Comma),
        map(char(':'), |_| Token::Colon),
        map(char('.'), |_| Token::Dot),
    ))(input)
}