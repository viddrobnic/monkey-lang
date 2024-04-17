mod environment;
mod error;
mod object;

#[cfg(test)]
mod test;

use std::collections::HashSet;
use std::rc::Rc;

use environment::*;

pub use error::*;
pub use object::*;

use crate::ast;

pub struct Evaluator {
    environment: Environment,

    environment_owners: HashSet<EnvironmentOwner>,
}

impl Evaluator {
    pub fn new() -> Self {
        let (env, env_owner) = Environment::new();

        let mut evaluator = Self {
            environment: env,
            environment_owners: HashSet::new(),
        };
        evaluator.environment_owners.insert(env_owner);

        evaluator
    }

    pub fn evaluate(&mut self, program: &ast::Program) -> Result<Object> {
        let mut res = Object::Null;

        // Have to work on cloned environment, because we can't
        // have two &mut references to self. This doesn't matter
        // anyway, since environment is reference counted underneath.
        let mut env = self.environment.clone();

        for stmt in &program.statements {
            res = self.evaluate_statement(stmt, &mut env)?;

            if let Object::Return(obj) = res {
                self.collect_garbage();
                return Ok((*obj).clone());
            }
        }

        self.collect_garbage();
        Ok(res)
    }

    fn collect_garbage(&mut self) {
        let mut used = HashSet::new();
        Self::collect_used_environments(&self.environment, &mut used);

        let mut to_remove = Vec::new();
        for env in self.environment_owners.iter() {
            if !used.contains(env) {
                to_remove.push(env.clone());
            }
        }

        for env in to_remove {
            self.environment_owners.remove(&env);
        }
    }

    fn collect_used_environments(env: &Environment, used: &mut HashSet<EnvironmentOwner>) {
        let inserted = used.insert(
            env.upgrade()
                .expect("Trying to access a dropped environment"),
        );
        if !inserted {
            return;
        }

        let env_rc = env
            .0
            .upgrade()
            .expect("Trying to access a dropped environment");

        for val in env_rc.borrow().store.values() {
            Self::collect_used_environments_from_obj(val, used);
        }
    }

    fn collect_used_environments_from_obj(obj: &Object, used: &mut HashSet<EnvironmentOwner>) {
        match obj {
            Object::Function(func) => Self::collect_used_environments(&func.environment, used),
            Object::Return(obj) => Self::collect_used_environments_from_obj(obj, used),
            _ => {}
        }
    }

    fn evaluate_statement(
        &mut self,
        stmt: &ast::Statement,
        environment: &mut Environment,
    ) -> Result<Object> {
        match stmt {
            ast::Statement::Let { name, value } => {
                let val = self.evaluate_expression(value, environment)?;
                environment.set(name.clone(), val);

                Ok(Object::Null)
            }
            ast::Statement::Return(expr) => {
                let val = self.evaluate_expression(expr, environment)?;
                Ok(Object::Return(Rc::new(val)))
            }
            ast::Statement::Expression(expr) => self.evaluate_expression(expr, environment),
        }
    }

    fn evaluate_expression(
        &mut self,
        expr: &ast::Expression,
        environment: &mut Environment,
    ) -> Result<Object> {
        match expr {
            ast::Expression::Identifier(ident) => {
                if let Some(val) = environment.get(ident) {
                    return Ok(val);
                }

                Err(Error::UnknownIdentifier(ident.clone()))
            }
            ast::Expression::IntegerLiteral(val) => Ok(Object::Integer(*val)),
            ast::Expression::BooleanLiteral(val) => Ok(Object::Boolean(*val)),
            ast::Expression::PrefixOperator { .. } => {
                self.evaluate_prefix_operator(expr, environment)
            }
            ast::Expression::InfixOperator { .. } => {
                self.evaluate_infix_operator(expr, environment)
            }
            ast::Expression::If { .. } => self.evaluate_if_expression(expr, environment),
            ast::Expression::FunctionLiteral { parameters, body } => {
                Ok(Object::Function(FunctionObject {
                    parameters: Rc::new(parameters.clone()),
                    body: body.clone(),
                    environment: environment.clone(),
                }))
            }
            ast::Expression::FunctionCall { .. } => self.evaluate_function_call(expr, environment),
        }
    }

    fn evaluate_prefix_operator(
        &mut self,
        expr: &ast::Expression,
        environment: &mut Environment,
    ) -> Result<Object> {
        let ast::Expression::PrefixOperator { operator, right } = expr else {
            panic!("Expected PrefixOperator expression, got {:?}", expr);
        };

        let right = self.evaluate_expression(right, environment)?;

        match operator {
            ast::PrefixOperatorKind::Not => match right {
                Object::Boolean(val) => Ok(Object::Boolean(!val)),
                Object::Null => Ok(Object::Boolean(true)),
                _ => Ok(Object::Boolean(false)),
            },
            ast::PrefixOperatorKind::Negative => match right {
                Object::Integer(val) => Ok(Object::Integer(-val)),
                _ => Err(Error::UnknownOperator(format!("-{}", right.data_type()))),
            },
        }
    }

    fn evaluate_infix_operator(
        &mut self,
        expr: &ast::Expression,
        environment: &mut Environment,
    ) -> Result<Object> {
        let ast::Expression::InfixOperator {
            operator,
            left,
            right,
        } = expr
        else {
            panic!("Expected InfixOperator expression, got {:?}", expr);
        };

        let left = self.evaluate_expression(left, environment)?;
        let right = self.evaluate_expression(right, environment)?;

        if let (Object::Integer(left), Object::Integer(right)) = (&left, &right) {
            let res = match operator {
                ast::InfixOperatorKind::Add => Object::Integer(left + right),
                ast::InfixOperatorKind::Subtract => Object::Integer(left - right),
                ast::InfixOperatorKind::Multiply => Object::Integer(left * right),
                ast::InfixOperatorKind::Divide => Object::Integer(left / right),
                ast::InfixOperatorKind::Equal => Object::Boolean(left == right),
                ast::InfixOperatorKind::NotEqual => Object::Boolean(left != right),
                ast::InfixOperatorKind::GreaterThan => Object::Boolean(left > right),
                ast::InfixOperatorKind::LessThan => Object::Boolean(left < right),
            };

            return Ok(res);
        }

        if let (Object::Boolean(left_bool), Object::Boolean(right_bool)) = (&left, &right) {
            let res = match operator {
                ast::InfixOperatorKind::Equal => Object::Boolean(left_bool == right_bool),
                ast::InfixOperatorKind::NotEqual => Object::Boolean(left_bool != right_bool),
                _ => {
                    return Err(Error::UnknownOperator(format!(
                        "{} {} {}",
                        left.data_type(),
                        operator.debug_str(),
                        right.data_type()
                    )))
                }
            };

            return Ok(res);
        }

        if left.data_type() != right.data_type() {
            return Err(Error::TypeMismatch(format!(
                "{} {} {}",
                left.data_type(),
                operator.debug_str(),
                right.data_type()
            )));
        }

        Err(Error::UnknownOperator(format!(
            "{} {} {}",
            left.data_type(),
            operator.debug_str(),
            right.data_type()
        )))
    }

    fn evaluate_if_expression(
        &mut self,
        expr: &ast::Expression,
        environment: &mut Environment,
    ) -> Result<Object> {
        let ast::Expression::If {
            condition,
            consequence,
            alternative,
        } = expr
        else {
            panic!("Expected If expression, got {:?}", expr);
        };

        let condition = self.evaluate_expression(condition, environment)?;

        if condition.is_truthy() {
            self.evaluate_block_statement(consequence, environment)
        } else {
            self.evaluate_block_statement(alternative, environment)
        }
    }

    fn evaluate_block_statement(
        &mut self,
        stmt: &ast::BlockStatement,
        environment: &mut Environment,
    ) -> Result<Object> {
        let mut res = Object::Null;

        for stmt in stmt.statements.iter() {
            res = self.evaluate_statement(stmt, environment)?;

            if matches!(res, Object::Return(_)) {
                return Ok(res);
            }
        }

        Ok(res)
    }

    fn evaluate_function_call(
        &mut self,
        expr: &ast::Expression,
        environment: &mut Environment,
    ) -> Result<Object> {
        let ast::Expression::FunctionCall {
            function,
            arguments,
        } = expr
        else {
            panic!("Expected FunctionCall expression, got {:?}", expr);
        };

        let args = arguments
            .iter()
            .map(|expr| self.evaluate_expression(expr, environment))
            .collect::<Result<Vec<_>>>()?;

        let function = self.evaluate_expression(function, environment)?;

        match function {
            Object::Function(function) => {
                let (mut extended_env, extended_env_owner) = function.environment.extend();
                self.environment_owners.insert(extended_env_owner);

                for (index, param) in function.parameters.iter().enumerate() {
                    extended_env.set(param.clone(), args[index].clone());
                }

                let evaluated = self.evaluate_block_statement(&function.body, &mut extended_env)?;
                match evaluated {
                    Object::Return(obj) => Ok((*obj).clone()),
                    _ => Ok(evaluated),
                }
            }
            _ => Err(Error::NotAFunction(function.data_type().to_string())),
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}