mod operator;
use std::rc::Rc;

pub use operator::*;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn debug_str(&self) -> String {
        let mut res = String::new();
        for stmt in &self.statements {
            res += &stmt.debug_str();
            res += ";";
        }

        res
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Let { name: String, value: Expression },
    Return(Expression),
    Expression(Expression),
}

impl Statement {
    pub fn debug_str(&self) -> String {
        match self {
            Self::Let { name, value } => format!("let {} = {}", name, value.debug_str()),
            Self::Return(expr) => format!("return {}", expr.debug_str()),
            Self::Expression(expr) => expr.debug_str(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockStatement {
    pub statements: Rc<Vec<Statement>>,
}

impl BlockStatement {
    pub fn debug_str(&self) -> String {
        let mut res = String::new();
        for stmt in self.statements.iter() {
            res += &stmt.debug_str();
            res += ";";
        }

        res
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Identifier(String),
    IntegerLiteral(i64),
    BooleanLiteral(bool),
    StringLiteral(String),
    ArrayLiteral(Vec<Expression>),
    PrefixOperator {
        operator: PrefixOperatorKind,
        right: Box<Expression>,
    },
    InfixOperator {
        operator: InfixOperatorKind,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        consequence: BlockStatement,
        alternative: BlockStatement,
    },
    FunctionLiteral {
        parameters: Vec<String>,
        body: BlockStatement,
    },
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Index {
        left: Box<Expression>,
        index: Box<Expression>,
    },
}

impl Expression {
    pub fn debug_str(&self) -> String {
        match self {
            Self::Identifier(name) => name.clone(),
            Self::IntegerLiteral(value) => value.to_string(),
            Self::BooleanLiteral(value) => value.to_string(),
            Self::StringLiteral(value) => value.clone(),
            Self::ArrayLiteral(value) => {
                format!(
                    "[{}]",
                    value
                        .iter()
                        .map(|exp| exp.debug_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Self::PrefixOperator { operator, right } => {
                format!("({}{})", operator.debug_str(), right.debug_str())
            }
            Self::InfixOperator {
                operator,
                left,
                right,
            } => format!(
                "({} {} {})",
                left.debug_str(),
                operator.debug_str(),
                right.debug_str()
            ),
            Self::If {
                condition,
                consequence,
                alternative,
            } => format!(
                "if ({}) {{{}}} else {{{}}}",
                condition.debug_str(),
                consequence.debug_str(),
                alternative.debug_str()
            ),
            Self::FunctionLiteral { parameters, body } => {
                format!("fn({}) {{{}}}", parameters.join(", "), body.debug_str())
            }
            Self::FunctionCall {
                function,
                arguments,
            } => {
                let args = arguments
                    .iter()
                    .map(|arg| arg.debug_str())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("{}({})", function.debug_str(), args)
            }
            Self::Index { left, index } => format!("({}[{}])", left.debug_str(), index.debug_str()),
        }
    }
}
