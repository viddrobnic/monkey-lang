use std::rc::Rc;

use super::{DataType, Error, Result};
use crate::object::{BuiltinFunction, Object};

pub(super) fn execute(fun: &BuiltinFunction, args: Vec<Object>) -> Result<Object> {
    match fun {
        BuiltinFunction::Len => execute_len(args),
        BuiltinFunction::First => execute_first(args),
        BuiltinFunction::Last => execute_last(args),
        BuiltinFunction::Rest => execute_rest(args),
        BuiltinFunction::Push => execute_push(args),
        BuiltinFunction::Puts => execute_puts(args),
    }
}

fn execute_len(args: Vec<Object>) -> Result<Object> {
    if args.len() != 1 {
        return Err(Error::WrongNumberOfArguments {
            expected: 1,
            got: args.len(),
        });
    }

    match &args[0] {
        Object::String(s) => Ok(Object::Integer(s.len() as i64)),
        Object::Array(arr) => Ok(Object::Integer(arr.len() as i64)),
        _ => Err(Error::TypeMismatch(DataType::from(&args[0]).to_string())),
    }
}

fn execute_first(args: Vec<Object>) -> Result<Object> {
    if args.len() != 1 {
        return Err(Error::WrongNumberOfArguments {
            expected: 1,
            got: args.len(),
        });
    }

    let Object::Array(arr) = &args[0] else {
        return Err(Error::TypeMismatch(DataType::from(&args[0]).to_string()));
    };

    if arr.is_empty() {
        Ok(Object::Null)
    } else {
        Ok(arr[0].clone())
    }
}

fn execute_last(args: Vec<Object>) -> Result<Object> {
    if args.len() != 1 {
        return Err(Error::WrongNumberOfArguments {
            expected: 1,
            got: args.len(),
        });
    }

    let Object::Array(arr) = &args[0] else {
        return Err(Error::TypeMismatch(DataType::from(&args[0]).to_string()));
    };

    if arr.is_empty() {
        Ok(Object::Null)
    } else {
        Ok(arr[arr.len() - 1].clone())
    }
}

fn execute_rest(args: Vec<Object>) -> Result<Object> {
    if args.len() != 1 {
        return Err(Error::WrongNumberOfArguments {
            expected: 1,
            got: args.len(),
        });
    }

    let Object::Array(arr) = &args[0] else {
        return Err(Error::TypeMismatch(DataType::from(&args[0]).to_string()));
    };

    if arr.is_empty() {
        Ok(Object::Null)
    } else {
        Ok(Object::Array(Rc::new(arr[1..].to_vec())))
    }
}

fn execute_push(args: Vec<Object>) -> Result<Object> {
    if args.len() != 2 {
        return Err(Error::WrongNumberOfArguments {
            expected: 2,
            got: args.len(),
        });
    }

    let Object::Array(arr) = &args[0] else {
        return Err(Error::TypeMismatch(DataType::from(&args[0]).to_string()));
    };

    let mut new_arr = (**arr).clone();
    new_arr.push(args[1].clone());
    Ok(Object::Array(Rc::new(new_arr)))
}

fn execute_puts(args: Vec<Object>) -> Result<Object> {
    for arg in args {
        println!("{}", arg.inspect());
    }
    Ok(Object::Null)
}
