pub mod builtin;

use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{ast, code::Instruction, environment::Environment};
use builtin::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    String(Rc<String>),
    Boolean(bool),
    Return(Rc<Object>),
    Function(FunctionObject),
    Builtin(BuiltinFunction),
    Array(Rc<Vec<Object>>),
    HashMap(Rc<HashMap<HashKey, Object>>),
    Null,
    CompiledFunction(CompiledFunction),
    Closure(Closure),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataType {
    Integer,
    String,
    Boolean,
    Return,
    Function,
    Builtin,
    Array,
    HashMap,
    Null,
    CompiledFunction,
    Closure,
}

impl From<&Object> for DataType {
    fn from(value: &Object) -> Self {
        match value {
            Object::Integer(_) => Self::Integer,
            Object::String(_) => Self::String,
            Object::Boolean(_) => Self::Boolean,
            Object::Return(_) => Self::Return,
            Object::Function(_) => Self::Function,
            Object::Builtin(_) => Self::Builtin,
            Object::Array(_) => Self::Array,
            Object::HashMap(_) => Self::HashMap,
            Object::Null => Self::Null,
            Object::CompiledFunction(_) => Self::CompiledFunction,
            Object::Closure { .. } => Self::Closure,
        }
    }
}

impl From<Object> for DataType {
    fn from(value: Object) -> Self {
        (&value).into()
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            DataType::Integer => "INTEGER",
            DataType::String => "STRING",
            DataType::Boolean => "BOOLEAN",
            DataType::Return => "RETURN",
            DataType::Function => "FUNCTION",
            DataType::Builtin => "BUILTIN",
            DataType::Array => "ARRAY",
            DataType::HashMap => "HASH_MAP",
            DataType::Null => "NULL",
            DataType::CompiledFunction => "COMPILED_FUNCTION",
            DataType::Closure => "CLOSURE",
        };

        f.write_str(string)
    }
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
            Object::HashMap(map) => {
                let elements = map
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key.inspect(), value.inspect()))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{{{}}}", elements)
            }
            Object::Null => "null".to_string(),
            Object::CompiledFunction(fun) => {
                format!("compiled function: {:?}", fun.instructions.as_ptr())
            }
            Object::Closure(closure) => {
                format!("closure: {:?}", closure.function.instructions.as_ptr())
            }
        }
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, Object::Boolean(false) | Object::Null)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CompiledFunction {
    pub instructions: Rc<Vec<Instruction>>,
    pub num_locals: usize,
    pub num_arguments: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Closure {
    pub function: CompiledFunction,
    pub free: Rc<Vec<Object>>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashKey {
    String(Rc<String>),
    Integer(i64),
    Boolean(bool),
}

impl HashKey {
    fn inspect(&self) -> String {
        match self {
            Self::String(s) => (**s).clone(),
            Self::Integer(i) => i.to_string(),
            Self::Boolean(b) => b.to_string(),
        }
    }
}

impl TryFrom<Object> for HashKey {
    type Error = DataType;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        match value {
            Object::String(str) => Ok(Self::String(str)),
            Object::Integer(i) => Ok(Self::Integer(i)),
            Object::Boolean(b) => Ok(Self::Boolean(b)),
            _ => Err(value.into()),
        }
    }
}
