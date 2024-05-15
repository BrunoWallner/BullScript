use crate::data::DataType;
use crate::error::{ParseError, ParseErrorExt};
use crate::lexer::{TokenKind, TokenStream};

use super::combinator::any;
use super::{AstNode, AstNodeData, BinaryOperator, Data as AstData, FnArgument};

pub fn parse(mut input: TokenStream) -> Result<Vec<AstNode>, ParseError> {
    let mut nodes = Vec::new();

    while !input.is_empty() {
        nodes.push(node(&mut input, 0)?);
        input.skip_if(&TokenKind::Semicolon);
    }

    Ok(nodes)
}

trait ParseFunction {
    fn name(&self) -> &'static str;
    fn func(&self) -> fn(&mut TokenStream, depth: u32) -> Result<AstNode, ParseError>;
}

fn node_filter(
    input: &mut TokenStream,
    filter: &[&'static str],
    depth: u32,
) -> Result<AstNode, ParseError> {
    // order matters
    let mut fns: Vec<&dyn ParseFunction> = vec![
        &Binary {},
        &FnDeclaration {},
        &FnCall {},
        &Block {},
        &Wrap {},
        &VarDeclaration {},
        &VarAssign {},
        &Return {},
        &Data {},
        &Identifier {},
    ];
    fns.retain(|func| !filter.contains(&func.name()));
    let functions: Vec<_> = fns.iter().map(|f| f.func()).collect();

    any(&functions, input, depth)
}

fn node(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
    node_filter(input, &[], depth)
}

struct Binary {}
impl Binary {
    fn parse(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        let mut tmp_input = input.clone();
        let left = Box::new(node_filter(&mut tmp_input, &["binary"], depth)?);
        // let index = input[0].index;
        let index = tmp_input.get_current_index().idc()?;

        let op = tmp_input.next().idc()?;
        let operator = BinaryOperator::from_tokenkind(&op.kind).ok_or(ParseError::new(
            index,
            depth + 1,
            format!("invalid operator: {:?}", op.kind),
        ))?;

        let right = Box::new(node(&mut tmp_input, depth + 2)?);

        *input = tmp_input;

        Ok(AstNode::new(
            AstNodeData::BinaryOperation {
                operator,
                left,
                right,
            },
            index,
        ))
    }
}
impl ParseFunction for Binary {
    fn name(&self) -> &'static str {
        "binary"
    }

    fn func(&self) -> fn(&mut TokenStream, u32) -> Result<AstNode, ParseError> {
        Self::parse
    }
}

struct Block {}
impl Block {
    fn parse(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        input.skip_if(&TokenKind::LeftBrace).idc()?;
        let index = input.get_current_index().idc()?;
        let inner = input
            .peek_pair_counting_stripped_inclusive(&TokenKind::LeftBrace, &TokenKind::RightBrace)
            .ok_or(ParseError::new(
                index,
                depth + 1,
                format!("missing closing '}}' delimiter"),
            ))?;
        let inner_len = inner.len();
        let mut nodes = Vec::new();
        let mut inner = TokenStream::new(inner);
        let mut depth = depth + 1;
        while inner.skip_if(&TokenKind::RightBrace).is_none() {
            let index = inner.get_current_index().idc()?;
            let n = node(&mut inner, depth)?;
            nodes.push(n.clone());
            inner.skip_if(&TokenKind::Semicolon).ok_or(ParseError::new(
                index,
                depth,
                format!("expected semicolon at the end"),
            ))?;
            depth += 1;
        }
        // only if successfull
        input.advance(inner_len);

        Ok(AstNode::new(AstNodeData::Block { block: nodes }, index))
    }
}
impl ParseFunction for Block {
    fn name(&self) -> &'static str {
        "block"
    }

    fn func(&self) -> fn(&mut TokenStream, u32) -> Result<AstNode, ParseError> {
        Self::parse
    }
}

struct Wrap {}
impl Wrap {
    fn parse(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        let index = input.peek(0).idc()?.index;
        input.skip_if(&TokenKind::LeftParen).idc()?;

        let inner = input
            .peek_pair_counting_stripped_inclusive(&TokenKind::LeftParen, &TokenKind::RightParen)
            .idc()?;
        let inner_len = inner.len();
        let mut stream = TokenStream::new(inner);
        let node = Box::new(node(&mut stream, depth)?);

        // only if successfull
        input.advance(inner_len);

        Ok(AstNode::new(AstNodeData::Wrap { wrap: node }, index))
    }
}
impl ParseFunction for Wrap {
    fn name(&self) -> &'static str {
        "wrap"
    }

    fn func(&self) -> fn(&mut TokenStream, u32) -> Result<AstNode, ParseError> {
        Self::parse
    }
}

struct FnDeclaration {}
impl FnDeclaration {
    fn parse(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        let index = input.peek(0).idc()?.index;
        input.skip_if(&TokenKind::Fn).idc()?;

        let peek = input.peek(0).idc()?.clone();
        (peek.kind == TokenKind::Identifier)
            .then(|| input.advance(1))
            .ok_or(ParseError::new(
                index,
                depth + 1,
                format!("expected identifier, but found: {:?}", peek.kind),
            ))?;
        let Some(DataType::String(name)) = peek.data else {
            return Err(ParseError::new(index, 2, format!("Identifier has no data")));
        };
        let arguments = Self::parse_fn_arguments(input)?;
        let returns = Self::parse_fn_return(input);
        let pre_body_index = input.get_current_index().idc()?;

        let body = Box::new(node(input, depth)?);
        match body.data {
            AstNodeData::Wrap { .. } | AstNodeData::Block { .. } => (),
            _ => {
                return Err(ParseError::new(
                    pre_body_index,
                    depth + 3,
                    format!("function body must either be a block or wrap"),
                ));
            }
        };

        return Ok(AstNode::new(
            AstNodeData::FnDeclaration {
                name,
                arguments,
                returns,
                body,
            },
            pre_body_index,
        ));
    }
    fn parse_fn_return(input: &mut TokenStream) -> Option<String> {
        input.skip_if(&TokenKind::Arrow)?;

        let peek = input.peek(0)?.clone();
        (peek.kind == TokenKind::Identifier).then(|| input.advance(1))?;
        match peek.data? {
            DataType::String(s) => Some(s),
            _ => None,
        }
    }

    fn parse_fn_arguments(input: &mut TokenStream) -> Result<Vec<FnArgument>, ParseError> {
        input.skip_if(&TokenKind::LeftParen).idc()?;
        let mut arguments = Vec::new();
        while let Some(argument) = parse_fn_inner(input) {
            arguments.push(argument)
        }
        input.skip_if(&TokenKind::RightParen).idc()?;

        return Ok(arguments);

        fn parse_fn_inner(input: &mut TokenStream) -> Option<FnArgument> {
            let peek = input.peek(0)?.clone();
            (peek.kind == TokenKind::Identifier).then(|| input.advance(1))?;
            let name = match peek.data? {
                DataType::String(s) => Some(s),
                _ => None,
            }?;

            // (input.peek(0)?.kind == TokenKind::Colon).then(|| input.advance(1))?;
            input.skip_if(&TokenKind::Colon)?;

            let peek = input.peek(0)?.clone();
            (peek.kind == TokenKind::Identifier).then(|| input.advance(1))?;
            let data_type = match peek.data? {
                DataType::String(s) => Some(s),
                _ => None,
            }?;

            // no ? at end, does not need to occur, but if it occurs do not fail but skip
            (input.peek(0)?.kind == TokenKind::Comma).then(|| input.advance(1));

            Some(FnArgument { name, data_type })
        }
    }
}
impl ParseFunction for FnDeclaration {
    fn name(&self) -> &'static str {
        "fn_declaration"
    }

    fn func(&self) -> fn(&mut TokenStream, u32) -> Result<AstNode, ParseError> {
        Self::parse
    }
}

struct FnCall {}
impl FnCall {
    fn parse(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        let peek = input.peek(0).idc()?.clone();
        let index = peek.index;

        (peek.kind == TokenKind::Identifier)
            .then(|| input.advance(1))
            .ok_or(ParseError::new(index, depth, format!("invalid identifier")))?;
        let Some(DataType::String(name)) = peek.data else {
            return Err(ParseError::new(
                index,
                depth,
                format!("invalid fn call name"),
            ));
        };
        let index = input.peek(0).idc()?.index;

        // (input.peek(0)?.kind == TokenKind::LeftParen).then(|| input.advance(1))?;
        input.skip_if(&TokenKind::LeftParen).idc()?;
        let inner = input
            .peek_pair_counting_stripped_inclusive(&TokenKind::LeftParen, &TokenKind::RightParen)
            .idc()?;
        let inner_len = inner.len();
        let mut arguments = Vec::new();
        let mut inner = TokenStream::new(inner);
        loop {
            arguments.push(node(&mut inner, depth)?);
            if inner.skip_if(&TokenKind::Comma).is_none() {
                inner.skip_if(&TokenKind::RightParen).idc()?;
                break;
            }
        }
        input.advance(inner_len);
        // input.skip_if(&TokenKind::RightParen)?;

        Ok(AstNode::new(AstNodeData::FnCall { name, arguments }, index))
    }
}
impl ParseFunction for FnCall {
    fn name(&self) -> &'static str {
        "fncall"
    }

    fn func(&self) -> fn(&mut TokenStream, u32) -> Result<AstNode, ParseError> {
        Self::parse
    }
}

struct Identifier {}
impl Identifier {
    fn parse(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        let peek = input.peek(0).idc()?.clone();
        let index = peek.index;

        // no proper error return, wanted!
        (peek.kind == TokenKind::Identifier)
            .then(|| input.advance(1))
            .idc()?;
        let Some(DataType::String(value)) = peek.data.clone() else {
            return Err(ParseError::new(index, depth, format!("invalid identifier")));
        };
        return Ok(AstNode::new(AstNodeData::Identifier { value }, index));
    }
}
impl ParseFunction for Identifier {
    fn name(&self) -> &'static str {
        "identifier"
    }

    fn func(&self) -> fn(&mut TokenStream, u32) -> Result<AstNode, ParseError> {
        Self::parse
    }
}

struct Data {}
impl Data {
    fn parse(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        any(&[Self::parse_literals, Self::parse_array], input, depth)
    }

    fn parse_literals(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        let peek = input.peek(0).idc()?;
        let index = peek.index;
        match peek.kind {
            TokenKind::StringLiteral | TokenKind::FloatLiteral => {
                let data = peek.data.clone().ok_or(ParseError::new(
                    index,
                    depth,
                    format!("invalid data"),
                ))?;
                input.advance(1);
                return Ok(AstNode::new(
                    AstNodeData::Data {
                        data: AstData::Base(data),
                    },
                    index,
                ));
            }
            TokenKind::True => {
                input.advance(1);
                return Ok(AstNode::new(
                    AstNodeData::Data {
                        data: AstData::Base(DataType::Bool(true)),
                    },
                    index,
                ));
            }
            TokenKind::False => {
                input.advance(1);
                return Ok(AstNode::new(
                    AstNodeData::Data {
                        data: AstData::Base(DataType::Bool(false)),
                    },
                    index,
                ));
            }
            _ => (),
        }
        return Err(ParseError::new(index, 0, format!("invalid data")));
    }

    fn parse_array(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        let index = input.peek(0).idc()?.index;
        input.skip_if(&TokenKind::LeftBracket).idc()?;

        let inner = input
            .peek_pair_counting_stripped_inclusive(
                &TokenKind::LeftBracket,
                &TokenKind::RightBracket,
            )
            .idc()?;
        let inner_len = inner.len();
        let mut array = Vec::new();
        let mut inner = TokenStream::new(inner);
        let mut depth = depth + 1;
        while let Ok(node) = node(&mut inner, depth) {
            array.push(node);
            // when true handle "[T; N]" case
            if inner.skip_if(&TokenKind::Comma).is_none() {
                if array.len() == 1 && inner.skip_if(&TokenKind::Semicolon).is_some() {
                    let peek = inner.peek(0).idc()?.clone();
                    (peek.kind == TokenKind::FloatLiteral)
                        .then(|| inner.advance(1))
                        .ok_or(ParseError::new(index, depth, format!("invalid data")))?;
                    let Some(DataType::Float(value)) = peek.data.clone() else {
                        return Err(ParseError::new(index, depth, format!("invalid data")));
                    };
                    let count = (value as i64 - 1).max(0);
                    for _ in 0..count {
                        array.push(array[0].clone());
                    }
                }
                break;
            }
            depth += 1;
        }
        // only if successfull
        input.advance(inner_len);

        Ok(AstNode::new(
            AstNodeData::Data {
                data: AstData::Array(array),
            },
            index,
        ))
    }
}
impl ParseFunction for Data {
    fn name(&self) -> &'static str {
        "data"
    }

    fn func(&self) -> fn(&mut TokenStream, u32) -> Result<AstNode, ParseError> {
        Self::parse
    }
}

struct Return {}
impl Return {
    fn parse(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        let peek = input.peek(0).idc()?;
        let index = peek.index;
        (peek.kind == TokenKind::Return)
            .then(|| input.advance(1))
            .ok_or(ParseError::new(index, depth, format!("invalid data in r")))?;
        let value = if let Ok(value) = node(input, depth) {
            Some(Box::new(value))
        } else {
            None
        };
        let index = input.get_current_index().unwrap_or(0);
        // input.skip_if(&TokenKind::Semicolon).ok_or(ParseError::new(
        //     index,
        //     depth + 1,
        //     format!("invalid statement"),
        // ))?;
        if input.peek(0).idc()?.kind != TokenKind::Semicolon {
            return Err(ParseError::new(
                index,
                depth + 1,
                format!("invalid statement"),
            ));
        }
        return Ok(AstNode::new(AstNodeData::Return { value }, index));
    }
}
impl ParseFunction for Return {
    fn name(&self) -> &'static str {
        "return"
    }

    fn func(&self) -> fn(&mut TokenStream, u32) -> Result<AstNode, ParseError> {
        Self::parse
    }
}

struct VarDeclaration {}
impl VarDeclaration {
    fn parse(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        let index = input.peek(0).idc()?.index;
        input.skip_if(&TokenKind::Let).idc()?;
        let peek = input.peek(0).idc()?.clone();
        (peek.kind == TokenKind::Identifier)
            .then(|| input.advance(1))
            .ok_or(ParseError::new(index, 1, format!("invalid data")))?;
        // if peek.kind == TokenKind::Identifier {
        let Some(DataType::String(name)) = peek.data else {
            return Err(ParseError::new(index, depth + 1, format!("invalid data")));
        };

        // (input.peek(0)?.kind == TokenKind::Equal).then(|| input.advance(1))?;
        input.skip_if(&TokenKind::Equal).idc()?;

        let value = Box::new(node(input, depth)?);
        Ok(AstNode::new(
            AstNodeData::VarDeclaration { name, value },
            index,
        ))
    }
}
impl ParseFunction for VarDeclaration {
    fn name(&self) -> &'static str {
        "var_declaration"
    }

    fn func(&self) -> fn(&mut TokenStream, u32) -> Result<AstNode, ParseError> {
        Self::parse
    }
}

struct VarAssign {}
impl VarAssign {
    fn parse(input: &mut TokenStream, depth: u32) -> Result<AstNode, ParseError> {
        let index = input.peek(0).idc()?.index;
        let peek = input.peek(0).idc()?.clone();
        (peek.kind == TokenKind::Identifier)
            .then(|| input.advance(1))
            .ok_or(ParseError::new(index, depth, format!("invalid data")))?;
        // if peek.kind == TokenKind::Identifier {
        let Some(DataType::String(name)) = peek.data else {
            return Err(ParseError::new(index, depth, format!("invalid data")));
        };

        // (input.peek(0)?.kind == TokenKind::Equal).then(|| input.advance(1))?;
        input.skip_if(&TokenKind::Equal).idc()?;

        let value = Box::new(node(input, depth)?);
        Ok(AstNode::new(AstNodeData::VarAssign { name, value }, index))
    }
}
impl ParseFunction for VarAssign {
    fn name(&self) -> &'static str {
        "var_assign"
    }

    fn func(&self) -> fn(&mut TokenStream, u32) -> Result<AstNode, ParseError> {
        Self::parse
    }
}
