use crate::{ast, evaluate, evaluate::Environment};

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    String(String),
    Boolean(bool),
    Return(Box<Object>),
    Function(FunctionObject),
    Builtin(BuiltinFunction),
    Array(Vec<Object>),
    Null,
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.to_string(),
            Object::String(s) => s.clone(),
            Object::Boolean(b) => b.to_string(),
            Object::Return(o) => o.inspect(),
            Object::Function(fun) => fun.inspect(),
            Object::Builtin(_) => "builtin function".to_string(),
            Object::Array(arr) => {
                let elements = arr
                    .iter()
                    .map(|obj| obj.inspect())
                    .collect::<Vec<String>>()
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
    pub parameters: Vec<ast::Identifier>,
    pub body: ast::BlockStatement,
    pub environment: Environment,
}

impl FunctionObject {
    fn inspect(&self) -> String {
        format!(
            "fn ({}) {{\n{}\n}}",
            self.parameters
                .iter()
                .map(|ident| ident.name.to_owned())
                .collect::<Vec<String>>()
                .join(", "),
            self.body.debug_str(),
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BuiltinFunction {
    Len,
}

impl BuiltinFunction {
    pub fn from_ident(ident: &str) -> Option<Self> {
        match ident {
            "len" => Some(Self::Len),
            _ => None,
        }
    }

    pub fn execute(&self, args: Vec<Object>) -> evaluate::Result<Object> {
        match self {
            BuiltinFunction::Len => Self::execute_len(args),
        }
    }

    fn execute_len(args: Vec<Object>) -> evaluate::Result<Object> {
        if args.len() != 1 {
            return Err(evaluate::Error::WrongNumberOfArguments {
                expected: 1,
                got: args.len(),
            });
        }

        match &args[0] {
            Object::String(s) => Ok(Object::Integer(s.len() as i64)),
            Object::Array(arr) => Ok(Object::Integer(arr.len() as i64)),
            _ => Err(evaluate::Error::TypeMismatch(
                args[0].data_type().to_string(),
            )),
        }
    }
}
