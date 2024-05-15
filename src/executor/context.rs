use crate::error::ExecutionError;
use super::{functions, Value};
use crate::parser::{Ast, AstNode, AstNodeData, BinaryOperator, Data, FnArgument};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Function {
    pub arguments: Vec<FnArgument>,
    pub returns: Option<String>,
    pub body: Box<AstNode>,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub variables: HashMap<String, Value>,
    pub functions: HashMap<String, Function>,
}
impl Context {
    pub fn new() -> Self {
        Self {
            variables: HashMap::default(),
            functions: HashMap::default(),
        }
    }

    pub fn build(ast: &Ast) -> Result<Self, ExecutionError> {
        let mut ctx = Context::new();
        ctx.collect_functions(&ast)?;
        ctx.evaluate(&ast)?;

        Ok(ctx)
    }

    pub fn call_function(&self, name: &str, arguments: Vec<Value>) -> Result<Option<Value>, ExecutionError> {
        match functions::call_inbuilt(name, arguments.clone()) {
            Ok(value) => return Ok(value),
            Err(_) => (),
        }
        // TODO: clone context and insert arguments into it
        let Some(function) = self.functions.get(name) else {
            return Err(ExecutionError::new(
                0,
                format!("function '{}' is not declared", name),
            ));
        };
        let body = function.body.clone();
        let arg_definitions = function.arguments.clone();
        if arg_definitions.len() != arguments.len() {
            return Err(ExecutionError::new(
                0,
                format!(
                    "invalid function arguments, expected {} value(s), found: {}",
                    arg_definitions.len(),
                    arguments.len()
                ),
            ));
        }

        let mut ctx = self.clone();
        for (i, arg) in arguments.into_iter().enumerate() {
            let name = arg_definitions[i].name.clone();
            ctx.variables.insert(name, arg);
        }
        ctx.handle_node(&*body)
    }

    fn collect_functions(&mut self, ast: &Ast) -> Result<(), ExecutionError> {
        for node in ast.nodes.iter().cloned() {
            match node.data {
                AstNodeData::FnDeclaration {
                    name,
                    arguments,
                    returns,
                    body,
                } => {
                    self.functions.insert(
                        name,
                        Function {
                            arguments,
                            returns,
                            body,
                        },
                    );
                }
                _ => continue,
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, ast: &Ast) -> Result<(), ExecutionError> {
        for node in ast.nodes.iter() {
            self.handle_node(node)?;
        }

        Ok(())
    }

    fn handle_node(&mut self, node: &AstNode) -> Result<Option<Value>, ExecutionError> {
        let mut output = None;
        match &node.data {
            AstNodeData::Block { block } => {
                for node in block.iter() {
                    let result = self.handle_node(node)?;
                    if matches!(node.data, AstNodeData::Return { .. }) {
                        return Ok(result);
                    }
                }
            }
            AstNodeData::Wrap { wrap } => output = self.handle_node(&*wrap)?,
            AstNodeData::FnCall { name, arguments } => {
                let mut ctx = self.clone();
                let mut args = Vec::new();
                for arg in arguments {
                    let Some(value) = ctx.handle_node(&arg)? else {
                        return Err(ExecutionError::new(
                            arg.index,
                            format!("invalid function argument: {}", name),
                        ));
                    };
                    args.push(value);
                }
                output = self.call_function(&name, args).map_err(|mut err| {
                    err.at = node.index;
                    err
                })?;
            }
            AstNodeData::VarDeclaration { name, value } => {
                let mut ctx = self.clone();
                let Some(value) = ctx.handle_node(&*value)? else {
                    return Err(ExecutionError::new(
                        node.index,
                        format!("invalid var declaration, value cannot be None"),
                    ));
                };
                let name = name.clone();
                self.variables.insert(name, value);
            }
            AstNodeData::VarAssign { name, value } => {
                let mut ctx = self.clone();
                let Some(value) = ctx.handle_node(&*value)? else {
                    return Err(ExecutionError::new(
                        node.index,
                        format!("invalid var declaration, value cannot be None"),
                    ));
                };
                // let name = name.clone();
                // self.variables.insert(name, value);
                let Some(var) = self.variables.get_mut(name) else {
                    return Err(ExecutionError::new(
                        node.index,
                        format!("cannot assign to: '{}', variable is not declared", name),
                    ));
                };
                *var = value;
            }
            AstNodeData::BinaryOperation {
                operator,
                left,
                right,
            } => {
                let Some(lhs) = self.handle_node(&*left)? else {
                    return Err(ExecutionError::new(
                        left.index,
                        format!("left hand side cannot be evaluated"),
                    ));
                };
                let Some(rhs) = self.handle_node(&*right)? else {
                    return Err(ExecutionError::new(
                        right.index,
                        format!("right hand side cannot be evaluated"),
                    ));
                };
                match operator {
                    BinaryOperator::Add => output = lhs.add(rhs.clone()),
                    BinaryOperator::Sub => output = lhs.sub(rhs.clone()),
                    BinaryOperator::Mul => output = lhs.mul(rhs.clone()),
                    BinaryOperator::Div => output = lhs.div(rhs.clone()),
                }
                if output.is_none() {
                    return Err(ExecutionError::new(
                        right.index,
                        format!(
                            "could not apply binary operation from: {:?} to: {:?}",
                            lhs, rhs
                        ),
                    ));
                }
                // TODO: impl math based on operator
            }
            AstNodeData::Return { value } => {
                let mut ctx = self.clone();
                if let Some(value) = value {
                    output = ctx.handle_node(&*value)?;
                }
            }
            AstNodeData::Identifier { value } => {
                // output = self.variables.get(value.as_str()).cloned()
                match self.variables.get(value.as_str()).cloned() {
                    Some(o) => output = Some(o),
                    None => {
                        return Err(ExecutionError::new(
                            node.index,
                            format!("variable: '{}' is not declared", value,),
                        ))
                    }
                }
            }
            AstNodeData::Data { data } => match data {
                Data::Base(b) => output = Some(Value::Data(b.clone())),
                Data::Array(a) => {
                    let mut array = Vec::new();
                    for node in a {
                        let mut ctx = self.clone();
                        let Some(value) = ctx.handle_node(&node)? else {
                            return Err(ExecutionError::new(node.index, format!("could not evaluate")));
                        };
                        array.push(value);
                    }
                    output = Some(Value::Array(array));
                }
            },
            _ => (),
        }

        Ok(output)
    }
}
