pub mod tokenstream;
pub use tokenstream::TokenStream;

pub mod kind;
// pub use kind::TokenKind;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alphanumeric1, char, digit1},
    combinator::{map, recognize},
    multi::{many0, many1},
    sequence::{pair, preceded, terminated},
    IResult,
};

use crate::data::DataType;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Colon,
    SemiColon,
    Bang,
    Equal,
    Greater,
    Less,

    // One or two character tokens
    BangEqual,
    EqualEqual,
    GreaterEqual,
    LessEqual,
    Arrow,

    // Literals
    Identifier,
    StringLiteral,
    FloatLiteral,
    IntLiteral,

    // Keywords
    If,
    Else,
    True,
    False,
    Fn,
    Return,
    Let,
    While,

    // End of file
    EOF,
    FloatLIteral,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub data: Option<DataType>,
    pub index: usize,
}
impl Token {
    pub fn empty(kind: TokenKind) -> Self {
        let token = Self { kind, data: None, index: 0 };
        token
    }
    pub fn new(kind: TokenKind, data: Option<DataType>) -> Self {
        Self { kind, data, index: 0 }
    }
}

pub fn tokenize(input: &str) -> TokenStream {
    let mut tokens = Vec::new();

    let initial_length = input.len(); // Store the initial length of the input
    let mut input = input.trim();
    // let mut index = initial_length - input.len();

    while !input.is_empty() {
        let (remaining_input, mut token) = match token(input) {
            Ok((rest, token)) => (rest.trim_start(), token), // Trim only the start of the input
            Err(_) => break,
        };

        // let index = initial_length - remaining_input.len();
        let index = initial_length - input.len();
        token.index = index;

        tokens.push(token);
        // index += input.len() - remaining_input.len();

        input = remaining_input;
    }

    TokenStream::from(tokens)
}

fn token(input: &str) -> IResult<&str, Token> {
    // order matters
    alt((
        string_literal,
        float_literal,
        int_literal,
        identifier_or_keyword,
        multiple_char_token,
        single_char_token,
    ))(input)
}

fn single_char_token(input: &str) -> IResult<&str, Token> {
    alt((
        map(char('('), |_| Token::empty(TokenKind::LeftParen)),
        map(char(')'), |_| Token::empty(TokenKind::RightParen)),
        map(char('{'), |_| Token::empty(TokenKind::LeftBrace)),
        map(char('}'), |_| Token::empty(TokenKind::RightBrace)),
        map(char('['), |_| Token::empty(TokenKind::LeftBracket)),
        map(char(']'), |_| Token::empty(TokenKind::RightBracket)),
        map(char(','), |_| Token::empty(TokenKind::Comma)),
        map(char('.'), |_| Token::empty(TokenKind::Dot)),
        map(char('-'), |_| Token::empty(TokenKind::Minus)),
        map(char('+'), |_| Token::empty(TokenKind::Plus)),
        map(char(';'), |_| Token::empty(TokenKind::Semicolon)),
        map(char('/'), |_| Token::empty(TokenKind::Slash)),
        map(char('*'), |_| Token::empty(TokenKind::Star)),
        map(char(':'), |_| Token::empty(TokenKind::Colon)),
        map(char(';'), |_| Token::empty(TokenKind::SemiColon)),
        map(char('!'), |_| Token::empty(TokenKind::Bang)),
        map(char('='), |_| Token::empty(TokenKind::Equal)),
        map(char('>'), |_| Token::empty(TokenKind::Greater)),
        map(char('<'), |_| Token::empty(TokenKind::Less)),
    ))(input)
}

fn multiple_char_token(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("!="), |_| Token::empty(TokenKind::BangEqual)),
        map(tag("=="), |_| Token::empty(TokenKind::EqualEqual)),
        map(tag(">="), |_| Token::empty(TokenKind::GreaterEqual)),
        map(tag("<="), |_| Token::empty(TokenKind::LessEqual)),
        map(tag("->"), |_| Token::empty(TokenKind::Arrow)),
    ))(input)
}

fn string_literal(input: &str) -> IResult<&str, Token> {
    let string_content = recognize(many0(alt((take_while1(|c| c != '"'), tag("\\\"")))));
    let string_parser = preceded(char('"'), terminated(string_content, char('"')));
    map(string_parser, |s: &str| {
        let data = Some(DataType::String(String::from(s)));
        Token::new(TokenKind::StringLiteral, data)
    })(input)
}

fn float_literal(input: &str) -> IResult<&str, Token> {
    let (input, num_str) = recognize(pair(digit1, pair(char('.'), digit1)))(input)?;
    // let num = num_str.parse::<f64>().unwrap();
    let data = num_str
        .parse::<f64>()
        .ok()
        .and_then(|d| Some(DataType::Float(d)));
    Ok((input, Token::new(TokenKind::FloatLiteral, data)))
}

fn int_literal(input: &str) -> IResult<&str, Token> {
    let (input, num_str) = recognize(digit1)(input)?;
    // let num = num_str.parse::<f64>().unwrap();
    let data = num_str
        .parse::<i64>()
        .ok()
        .and_then(|d| Some(DataType::Int(d)));
    Ok((input, Token::new(TokenKind::FloatLiteral, data)))
}

fn identifier_or_keyword(input: &str) -> IResult<&str, Token> {
    let (remaining_input, token) = alt((
        map(tag("if"), |_| Token::empty(TokenKind::If)),
        map(tag("else"), |_| Token::empty(TokenKind::Else)),
        map(tag("true"), |_| Token::empty(TokenKind::True)),
        map(tag("false"), |_| Token::empty(TokenKind::False)),
        map(tag("fn"), |_| Token::empty(TokenKind::Fn)),
        map(tag("return"), |_| Token::empty(TokenKind::Return)),
        map(tag("let"), |_| Token::empty(TokenKind::Let)),
        map(tag("while"), |_| Token::empty(TokenKind::While)),
        // map(alphanumeric1, |i: &str| Token::Identifier(i.to_string()))
        map(
            // pair(alphanumeric1, tag("_")),
            many1(alt((alphanumeric1, tag("_"), tag("::")))),
            // |i: Vec<&str>| Token::Identifier { string: i.join("") },
            |i: Vec<&str>| {
                let data = Some(DataType::String(i.join("")));
                Token::new(TokenKind::Identifier, data)
            },
        ),
    ))(input)?;

    Ok((remaining_input, token))
}
