mod context;
mod functions;
pub mod value;

use crate::parser::Ast;
pub use crate::data::IntoValue;

use context::Context;
use crate::error::ExecutionError;
use crate::data::Value;

#[derive(Clone, Debug)]
pub struct Executor {
    pub ast: Ast,
    pub context: Context,
}
impl Executor {
    pub fn build(ast: Ast) -> Result<Self, ExecutionError> {
        let context = Context::build(&ast)?;
        Ok(Self { ast, context })
    }

    pub fn execute(
        &mut self,
        entry: &str,
        arguments: Vec<&dyn IntoValue>,
    ) -> Result<Option<Value>, ExecutionError> {
        let values = arguments.into_iter().map(|v| v.into_value()).collect();
        self.context.call_function(entry, values)
    }
}
