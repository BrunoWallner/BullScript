pub mod parse;
pub use parse::parse;

use crate::{data, lexer::TokenKind};
use data::DataType;

mod combinator;

#[derive(Clone, Debug)]
pub enum Data {
    Base(DataType),
    Array(Vec<AstNode>),
}

#[derive(Clone, Debug)]
pub struct FnArgument {
    pub name: String,
    pub data_type: String,
}

#[derive(Copy, Clone, Debug)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
}
impl BinaryOperator {
    pub fn from_tokenkind(t: &TokenKind) -> Option<Self> {
        match t {
            &TokenKind::Plus => Some(Self::Add),
            &TokenKind::Minus => Some(Self::Sub),
            &TokenKind::Star => Some(Self::Mul),
            &TokenKind::Slash => Some(Self::Div),
            _ => None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct AstNode {
    pub data: AstNodeData,
    pub index: usize,
}
impl AstNode {
    pub fn new(data: AstNodeData, index: usize) -> Self {
        Self { data, index }
    }
}

#[derive(Clone, Debug)]
pub enum AstNodeData {
    Block {
        block: Vec<AstNode>,
    },
    Wrap {
        wrap: Box<AstNode>,
    },
    FnDeclaration {
        name: String,
        arguments: Vec<FnArgument>,
        returns: Option<String>,
        body: Box<AstNode>,
    },
    FnCall {
        name: String,
        arguments: Vec<AstNode>,
    },
    VarDeclaration {
        name: String,
        value: Box<AstNode>,
    },
    VarAssign {
        name: String,
        value: Box<AstNode>,
    },
    BinaryOperation {
        operator: BinaryOperator,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    IfStatement {
        condition: Box<AstNode>,
        when: Box<AstNode>,
        unless: Option<Box<AstNode>>,
    },
    Return {
        value: Option<Box<AstNode>>,
    },
    Identifier {
        value: String,
    },
    Data {
        data: Data,
    },
}

#[derive(Clone, Debug)]
pub struct Ast {
    pub nodes: Vec<AstNode>,
}
impl Ast {
    pub fn new(nodes: Vec<AstNode>) -> Self {
        Self { nodes }
    }
}
