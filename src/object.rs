use crate::{ast, evaluate::Environment};

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Return(Box<Object>),
    Function(FunctionObject),
    Null,
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.to_string(),
            Object::Boolean(b) => b.to_string(),
            Object::Return(o) => o.inspect(),
            Object::Function(fun) => fun.inspect(),
            Object::Null => "null".to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, Object::Boolean(false) | Object::Null)
    }

    pub fn data_type(&self) -> &str {
        match self {
            Object::Integer(_) => "INTEGER",
            Object::Boolean(_) => "BOOLEAN",
            Object::Return(_) => "RETURN",
            Object::Function(_) => "FUNCTION",
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
