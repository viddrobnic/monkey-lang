use crate::{parser::error::Error, token::Token};

#[derive(Debug, PartialEq)]
pub enum Statement {
    Let(Let),
    Return(Return),
    Expression(Expression),
}

impl Statement {
    pub fn debug_str(&self) -> String {
        match self {
            Self::Let(let_stmt) => format!(
                "let {} = {};",
                let_stmt.name.name,
                let_stmt.value.debug_str()
            ),
            Self::Return(return_stmt) => format!("return {};", return_stmt.value.debug_str()),
            Self::Expression(expr) => expr.debug_str(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    IntegerLiteral(IntegerLiteral),
    PrefixOperator(PrefixOperator),
    InfixOperator(InfixOperator),
    BooleanLiteral(BooleanLiteral),
    If(IfExpression),
    FunctionLiteral(FunctionLiteral),
    Empty,
}

impl Expression {
    pub fn debug_str(&self) -> String {
        match self {
            Expression::Identifier(identifier) => identifier.name.clone(),
            Expression::IntegerLiteral(literal) => literal.value.to_string(),
            Expression::PrefixOperator(prefix) => format!(
                "({}{})",
                prefix.operator.debug_str(),
                prefix.right.debug_str()
            ),
            Expression::InfixOperator(infix) => format!(
                "({} {} {})",
                infix.left.debug_str(),
                infix.operator.debug_str(),
                infix.right.debug_str()
            ),
            Expression::BooleanLiteral(literal) => literal.value.to_string(),
            Expression::If(ifs) => format!(
                "if {} {} else {}",
                ifs.condition.debug_str(),
                ifs.consequence.debug_str(),
                ifs.alternative.debug_str()
            ),
            Expression::FunctionLiteral(fun) => fun.debug_str(),
            Expression::Empty => String::new(),
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn debug_str(&self) -> String {
        let mut res = String::new();
        for stmt in &self.statements {
            res += &stmt.debug_str();
        }
        res
    }
}

#[derive(Debug, PartialEq)]
pub struct Let {
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Debug, PartialEq)]
pub struct Return {
    pub value: Expression,
}

#[derive(Debug, PartialEq)]
pub struct Identifier {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct IntegerLiteral {
    pub value: i64,
}

#[derive(Debug, PartialEq)]
pub struct BooleanLiteral {
    pub value: bool,
}

#[derive(Debug, PartialEq)]
pub enum PrefixOperatorKind {
    Not,
    Negative,
}

impl PrefixOperatorKind {
    pub fn debug_str(&self) -> String {
        match self {
            PrefixOperatorKind::Not => "!",
            PrefixOperatorKind::Negative => "-",
        }
        .to_owned()
    }
}

impl TryFrom<&Token> for PrefixOperatorKind {
    type Error = Error;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::Bang => Ok(Self::Not),
            Token::Minus => Ok(Self::Negative),
            token => Err(Error::UnexpectedToken(token.clone())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PrefixOperator {
    pub operator: PrefixOperatorKind,
    pub right: Box<Expression>,
}

#[derive(Debug, PartialEq)]
pub enum InfixOperatorKind {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
}

impl InfixOperatorKind {
    pub fn debug_str(&self) -> String {
        match self {
            InfixOperatorKind::Add => "+",
            InfixOperatorKind::Subtract => "-",
            InfixOperatorKind::Multiply => "*",
            InfixOperatorKind::Divide => "/",
            InfixOperatorKind::Equal => "==",
            InfixOperatorKind::NotEqual => "!=",
            InfixOperatorKind::GreaterThan => ">",
            InfixOperatorKind::LessThan => "<",
        }
        .to_owned()
    }
}

impl TryFrom<&Token> for InfixOperatorKind {
    type Error = Error;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::Plus => Ok(Self::Add),
            Token::Minus => Ok(Self::Subtract),
            Token::Asterisk => Ok(Self::Multiply),
            Token::Slash => Ok(Self::Divide),
            Token::Eq => Ok(Self::Equal),
            Token::NotEq => Ok(Self::NotEqual),
            Token::Gt => Ok(Self::GreaterThan),
            Token::Lt => Ok(Self::LessThan),
            token => Err(Error::UnexpectedToken(token.clone())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InfixOperator {
    pub operator: InfixOperatorKind,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub consequence: BlockStatement,
    pub alternative: BlockStatement,
}

#[derive(Debug, PartialEq)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

impl BlockStatement {
    pub fn debug_str(&self) -> String {
        let mut res = String::new();

        for stmt in &self.statements {
            res += &stmt.debug_str();
        }

        res
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionLiteral {
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}

impl FunctionLiteral {
    pub fn debug_str(&self) -> String {
        format!(
            "fn ({}) {}",
            self.parameters
                .iter()
                .map(|ident| ident.name.to_owned())
                .collect::<Vec<String>>()
                .join(", "),
            self.body.debug_str(),
        )
    }
}
