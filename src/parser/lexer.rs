use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0},
    combinator::{map, map_res, recognize},
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
    Match,
    Every,
    After,
    While,
    For,
    In,
    Func,
    Class,
    Struct,
    Enum,
    Let,
    Mut,
    Return,
    Break,
    Continue,
    Main,
    As,
    Content,
    Style,
    
    // Identifiers and literals
    Identifier(String),
    Integer(i64),
    Float(f64),
    Percentage(f64),
    String(String),
    InterpolatedString(Vec<crate::parser::ast::StringPart>),
    Boolean(bool),
    Unit(String),
    ArrayLiteral(Vec<Token>),
    
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
    LogicalAnd,
    LogicalOr,
    Range,
    RangeInclusive,
    Pipe,
    BiDirectionalPipe,
    Branch(u8),
    
    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Dot,
    Assignment,
    Arrow,
    Underscore,
    Semicolon,
    Hash,
    Dollar,
    Pipe2,
    
    // Special
    Newline,
    Eof,
}

pub fn tokenize(input: &str) -> IResult<&str, Vec<Token>> {
    many0(preceded(skip_whitespace_comments, token))(input)
}

fn skip_whitespace_comments(input: &str) -> IResult<&str, ()> {
    let (mut input, _) = multispace0(input)?;
    
    // Handle comments
    loop {
        if let Ok((remaining, _)) = tag::<&str, &str, nom::error::Error<&str>>("//")(input) {
            // Found a // comment, skip to end of line
            let (remaining, _) = take_while(|c| c != '\n' && c != '\r')(remaining)?;
            let (remaining, _) = multispace0(remaining)?;
            input = remaining;
        } else if let Ok((remaining, _)) = tag::<&str, &str, nom::error::Error<&str>>("#")(input) {
            // Found a # comment, skip to end of line
            let (remaining, _) = take_while(|c| c != '\n' && c != '\r')(remaining)?;
            let (remaining, _) = multispace0(remaining)?;
            input = remaining;
        } else {
            break;
        }
    }
    
    Ok((input, ()))
}

fn token(input: &str) -> IResult<&str, Token> {
    alt((
        keyword,
        boolean,
        percentage,
        float_with_unit,
        integer_with_unit,
        float,
        integer,
        interpolated_string,
        string_literal,
        array_literal,
        identifier,
        operator,
        punctuation,
    ))(input)
}

fn keyword(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("import"), |_| Token::Import),
        map(tag("loop"), |_| Token::Loop),
        map(tag("match"), |_| Token::Match),
        map(tag("every"), |_| Token::Every),
        map(tag("after"), |_| Token::After),
        map(tag("while"), |_| Token::While),
        map(tag("for"), |_| Token::For),
        map(tag("in"), |_| Token::In),
        map(tag("if"), |_| Token::If),
        map(tag("else"), |_| Token::Else),
        map(tag("func"), |_| Token::Func),
        map(tag("class"), |_| Token::Class),
        map(tag("struct"), |_| Token::Struct),
        map(tag("enum"), |_| Token::Enum),
        map(tag("let"), |_| Token::Let),
        map(tag("mut"), |_| Token::Mut),
        map(tag("return"), |_| Token::Return),
        map(tag("break"), |_| Token::Break),
        map(tag("continue"), |_| Token::Continue),
        map(tag("main"), |_| Token::Main),
        map(tag("as"), |_| Token::As),
        map(tag("content"), |_| Token::Content),
        map(tag("style"), |_| Token::Style),
    ))(input)
}

fn boolean(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("true"), |_| Token::Boolean(true)),
        map(tag("false"), |_| Token::Boolean(false)),
    ))(input)
}

fn integer(input: &str) -> IResult<&str, Token> {
    map_res(
        recognize(many1(nom::character::complete::digit1)),
        |s: &str| {
            s.parse::<i64>()
                .map(Token::Integer)
                .map_err(|_| nom::error::ErrorKind::Digit)
        },
    )(input)
}

fn float(input: &str) -> IResult<&str, Token> {
    map_res(
        recognize(pair(
            many1(nom::character::complete::digit1),
            pair(char('.'), many1(nom::character::complete::digit1)),
        )),
        |s: &str| {
            s.parse::<f64>()
                .map(Token::Float)
                .map_err(|_| nom::error::ErrorKind::Float)
        },
    )(input)
}

fn percentage(input: &str) -> IResult<&str, Token> {
    let (input, number) = alt((
        recognize(pair(
            many1(nom::character::complete::digit1),
            pair(char('.'), many1(nom::character::complete::digit1)),
        )),
        recognize(many1(nom::character::complete::digit1)),
    ))(input)?;
    let (input, _) = char('%')(input)?;
    let value: f64 = number.parse()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Float)))?;
    Ok((input, Token::Percentage(value / 100.0))) // Convert to 0.0-1.0 range
}

fn string_literal(input: &str) -> IResult<&str, Token> {
    map(
        delimited(
            char('"'),
            take_while1(|c| c != '"' && c != '$'),
            char('"'),
        ),
        |s: &str| Token::String(s.to_string()),
    )(input)
}

fn interpolated_string(input: &str) -> IResult<&str, Token> {
    let (input, _) = char('"')(input)?;
    let mut parts = Vec::new();
    let mut remaining = input;
    
    loop {
        // Parse text part
        let (rest, text) = take_while(|c| c != '$' && c != '"')(remaining)?;
        if !text.is_empty() {
            parts.push(crate::parser::ast::StringPart::Text(text.to_string()));
        }
        
        if let Ok((rest, _)) = char::<&str, nom::error::Error<&str>>('"')(rest) {
            // End of string
            return Ok((rest, Token::InterpolatedString(parts)));
        } else if let Ok((rest, _)) = tag::<&str, &str, nom::error::Error<&str>>("${")(rest) {
            // Start of interpolation
            let (rest, expr) = take_while(|c| c != '}')(rest)?;
            let (rest, _) = char('}')(rest)?;
            // For now, we'll store the interpolation as a simple identifier expression
            // The parser will need to parse this properly
            let interpolation_expr = crate::parser::ast::Expression::Identifier(expr.to_string());
            parts.push(crate::parser::ast::StringPart::Interpolation(interpolation_expr));
            remaining = rest;
        } else {
            // Unexpected character, treat as regular string
            let (rest, _) = char('"')(remaining)?;
            return Ok((rest, Token::String("".to_string())));
        }
    }
}

fn array_literal(input: &str) -> IResult<&str, Token> {
    let (input, _) = char('[')(input)?;
    let (input, _) = multispace0(input)?;
    
    let mut elements = Vec::new();
    let mut remaining = input;
    
    while !remaining.is_empty() {
        if let Ok((rest, _)) = char::<&str, nom::error::Error<&str>>(']')(remaining) {
            return Ok((rest, Token::ArrayLiteral(elements)));
        }
        
        // Parse element
        if let Ok((rest, token)) = token(remaining) {
            elements.push(token);
            let (rest, _) = multispace0(rest)?;
            
            if let Ok((rest, _)) = char::<&str, nom::error::Error<&str>>(',')(rest) {
                let (rest, _) = multispace0(rest)?;
                remaining = rest;
            } else {
                remaining = rest;
            }
        } else {
            break;
        }
    }
    
    let (input, _) = char(']')(remaining)?;
    Ok((input, Token::ArrayLiteral(elements)))
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

fn integer_with_unit(input: &str) -> IResult<&str, Token> {
    let (input, number) = recognize(many1(nom::character::complete::digit1))(input)?;
    let (input, _) = char('.')(input)?;
    let (input, unit) = unit_suffix(input)?;
    Ok((input, Token::Unit(format!("{}.{}", number, unit))))
}

fn float_with_unit(input: &str) -> IResult<&str, Token> {
    let (input, number) = recognize(pair(
        many1(nom::character::complete::digit1),
        pair(char('.'), many1(nom::character::complete::digit1)),
    ))(input)?;
    let (input, _) = char('.')(input)?;
    let (input, unit) = unit_suffix(input)?;
    Ok((input, Token::Unit(format!("{}.{}", number, unit))))
}

fn unit_suffix(input: &str) -> IResult<&str, &str> {
    alt((
        tag("px"), tag("s"), tag("ms"), tag("Hz"), tag("kHz"), 
        tag("degrees"), tag("radians"), tag("percent"), tag("%")
    ))(input)
}

fn operator(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("=>"), |_| Token::Arrow),
        map(tag("=="), |_| Token::Equals),
        map(tag("!="), |_| Token::NotEqual),
        map(tag("<="), |_| Token::LessThanOrEqual),
        map(tag(">="), |_| Token::GreaterThanOrEqual),
        map(tag("&&"), |_| Token::LogicalAnd),
        map(tag("||"), |_| Token::LogicalOr),
        map(tag("..="), |_| Token::RangeInclusive),
        map(tag(".."), |_| Token::Range),
        map(tag("<>"), |_| Token::BiDirectionalPipe),
        map(tag("<"), |_| Token::LessThan),
        map(tag(">"), |_| Token::GreaterThan),
        map(tag("|>"), |_| Token::Pipe),
        branch_operator,
        map(tag("+"), |_| Token::Plus),
        map(tag("-"), |_| Token::Minus),
        map(tag("*"), |_| Token::Multiply),
        map(tag("/"), |_| Token::Divide),
        map(tag("="), |_| Token::Assignment),
    ))(input)
}

fn branch_operator(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("branch(")(input)?;
    let (input, n) = nom::character::complete::digit1(input)?;
    let (input, _) = char(')')(input)?;
    let branch_count = n.parse().unwrap_or(2);
    Ok((input, Token::Branch(branch_count)))
}

fn punctuation(input: &str) -> IResult<&str, Token> {
    alt((
        map(char('('), |_| Token::LeftParen),
        map(char(')'), |_| Token::RightParen),
        map(char('{'), |_| Token::LeftBrace),
        map(char('}'), |_| Token::RightBrace),
        map(char('['), |_| Token::LeftBracket),
        map(char(']'), |_| Token::RightBracket),
        map(char(','), |_| Token::Comma),
        map(char(':'), |_| Token::Colon),
        map(char('.'), |_| Token::Dot),
        map(char('_'), |_| Token::Underscore),
        map(char(';'), |_| Token::Semicolon),
        map(char('#'), |_| Token::Hash),
        map(char('$'), |_| Token::Dollar),
        map(char('|'), |_| Token::Pipe2),
    ))(input)
}