use super::environment::Environment;
use crate::ast;

use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    String(String),
    Boolean(bool),
    Return(Rc<Object>),
    Function(FunctionObject),
    // Builtin(BuiltinFunction),
    // Array(Vec<Object>),
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
            // Object::Builtin(_) => "builtin function".to_string(),
            // Object::Array(arr) => {
            //     let elements = arr
            //         .iter()
            //         .map(|obj| obj.inspect())
            //         .collect::<Vec<String>>()
            //         .join(", ");
            //     format!("[{}]", elements)
            // }
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
            // Object::Builtin(_) => "BUILTIN",
            // Object::Array(_) => "ARRAY",
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
//
// #[derive(Debug, PartialEq, Clone)]
// pub enum BuiltinFunction {
//     Len,
//     First,
//     Last,
//     Rest,
//     Push,
//     Puts,
// }
//
// impl BuiltinFunction {
//     pub fn from_ident(ident: &str) -> Option<Self> {
//         match ident {
//             "len" => Some(Self::Len),
//             "first" => Some(Self::First),
//             "last" => Some(Self::Last),
//             "rest" => Some(Self::Rest),
//             "push" => Some(Self::Push),
//             "puts" => Some(Self::Puts),
//             _ => None,
//         }
//     }
//
//     pub fn execute(&self, args: Vec<Object>) -> evaluate::Result<Object> {
//         match self {
//             BuiltinFunction::Len => Self::execute_len(args),
//             BuiltinFunction::First => Self::execute_first(args),
//             BuiltinFunction::Last => Self::execute_last(args),
//             BuiltinFunction::Rest => Self::execute_rest(args),
//             BuiltinFunction::Push => Self::execute_push(args),
//             BuiltinFunction::Puts => Self::execute_puts(args),
//         }
//     }
//
//     fn execute_len(args: Vec<Object>) -> evaluate::Result<Object> {
//         if args.len() != 1 {
//             return Err(evaluate::Error::WrongNumberOfArguments {
//                 expected: 1,
//                 got: args.len(),
//             });
//         }
//
//         match &args[0] {
//             Object::String(s) => Ok(Object::Integer(s.len() as i64)),
//             Object::Array(arr) => Ok(Object::Integer(arr.len() as i64)),
//             _ => Err(evaluate::Error::TypeMismatch(
//                 args[0].data_type().to_string(),
//             )),
//         }
//     }
//
//     fn execute_first(args: Vec<Object>) -> evaluate::Result<Object> {
//         if args.len() != 1 {
//             return Err(evaluate::Error::WrongNumberOfArguments {
//                 expected: 1,
//                 got: args.len(),
//             });
//         }
//
//         let Object::Array(arr) = &args[0] else {
//             return Err(evaluate::Error::TypeMismatch(
//                 args[0].data_type().to_string(),
//             ));
//         };
//
//         if arr.is_empty() {
//             Ok(Object::Null)
//         } else {
//             Ok(arr[0].clone())
//         }
//     }
//
//     fn execute_last(args: Vec<Object>) -> evaluate::Result<Object> {
//         if args.len() != 1 {
//             return Err(evaluate::Error::WrongNumberOfArguments {
//                 expected: 1,
//                 got: args.len(),
//             });
//         }
//
//         let Object::Array(arr) = &args[0] else {
//             return Err(evaluate::Error::TypeMismatch(
//                 args[0].data_type().to_string(),
//             ));
//         };
//
//         if arr.is_empty() {
//             Ok(Object::Null)
//         } else {
//             Ok(arr[arr.len() - 1].clone())
//         }
//     }
//
//     fn execute_rest(args: Vec<Object>) -> evaluate::Result<Object> {
//         if args.len() != 1 {
//             return Err(evaluate::Error::WrongNumberOfArguments {
//                 expected: 1,
//                 got: args.len(),
//             });
//         }
//
//         let Object::Array(arr) = &args[0] else {
//             return Err(evaluate::Error::TypeMismatch(
//                 args[0].data_type().to_string(),
//             ));
//         };
//
//         if arr.is_empty() {
//             Ok(Object::Null)
//         } else {
//             Ok(Object::Array(arr[1..].to_vec()))
//         }
//     }
//
//     fn execute_push(args: Vec<Object>) -> evaluate::Result<Object> {
//         if args.len() != 2 {
//             return Err(evaluate::Error::WrongNumberOfArguments {
//                 expected: 2,
//                 got: args.len(),
//             });
//         }
//
//         let Object::Array(arr) = &args[0] else {
//             return Err(evaluate::Error::TypeMismatch(
//                 args[0].data_type().to_string(),
//             ));
//         };
//
//         let mut new_arr = arr.clone();
//         new_arr.push(args[1].clone());
//         Ok(Object::Array(new_arr))
//     }
//
//     fn execute_puts(args: Vec<Object>) -> evaluate::Result<Object> {
//         for arg in args {
//             println!("{}", arg.inspect());
//         }
//         Ok(Object::Null)
//     }
// }
