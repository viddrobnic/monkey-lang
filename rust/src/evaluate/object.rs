use super::{builtin::BuiltinFunction, environment::Environment};
use crate::ast;

use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    String(Rc<String>),
    Boolean(bool),
    Return(Rc<Object>),
    Function(FunctionObject),
    Builtin(BuiltinFunction),
    Array(Rc<Vec<Object>>),
    Null,
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.to_string(),
            Object::String(s) => (**s).clone(),
            Object::Boolean(b) => b.to_string(),
            Object::Return(o) => o.inspect(),
            Object::Function(fun) => fun.inspect(),
            Object::Builtin(f) => format!("builtin function {:?}", f),
            Object::Array(arr) => {
                let elements = arr
                    .iter()
                    .map(|obj| obj.inspect())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", elements)
            }
            Object::Null => "null".to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, Object::Boolean(false) | Object::Null)
    }

    pub fn data_type(&self) -> &str {
        match self {
            Object::Integer(_) => "INTEGER",
            Object::String(_) => "STRING",
            Object::Boolean(_) => "BOOLEAN",
            Object::Return(_) => "RETURN",
            Object::Function(_) => "FUNCTION",
            Object::Builtin(_) => "BUILTIN",
            Object::Array(_) => "ARRAY",
            Object::Null => "NULL",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionObject {
    pub parameters: Rc<Vec<String>>,
    pub body: ast::BlockStatement,
    pub environment: Environment,
}

impl FunctionObject {
    fn inspect(&self) -> String {
        format!(
            "fn ({}) {{\n{}\n}}",
            self.parameters.join(", "),
            self.body.debug_str(),
        )
    }
}
