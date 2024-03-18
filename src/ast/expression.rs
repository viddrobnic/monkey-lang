use crate::{
    evaluate::{self, Environment, Evaluate},
    object::{FunctionObject, Object},
    parse::Parse,
    parse::{Error, Parser, Precedence, Result},
    token::Token,
};

use super::{InfixOperatorKind, PrefixOperatorKind, Statement};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(Identifier),
    IntegerLiteral(IntegerLiteral),
    StringLiteral(String),
    PrefixOperator(PrefixOperator),
    InfixOperator(InfixOperator),
    BooleanLiteral(BooleanLiteral),
    If(IfExpression),
    FunctionLiteral(FunctionLiteral),
    FunctionCall(FunctionCall),
}

impl Parse for Expression {
    fn parse(
        parser: &mut crate::parse::Parser,
        precedence: crate::parse::Precedence,
        _: Option<Expression>,
    ) -> Result<Self> {
        let mut left = Self::parse_prefix(parser, precedence)?;

        while *parser.get_peek_token() != Token::Semicolon && precedence < parser.peek_precedence()
        {
            if !parser.get_peek_token().is_infix() {
                return Ok(left);
            }

            parser.step();
            left = Self::parse_infix(parser, precedence, left)?;
        }

        Ok(left)
    }
}

impl Evaluate for Expression {
    fn evaluate(&self, environment: &mut Environment) -> evaluate::Result<Object> {
        match self {
            Expression::Identifier(ident) => ident.evaluate(environment),
            Expression::IntegerLiteral(literal) => Ok(Object::Integer(literal.value)),
            Expression::StringLiteral(literal) => Ok(Object::String(literal.clone())),
            Expression::PrefixOperator(prefix) => prefix.evaluate(environment),
            Expression::InfixOperator(infix) => infix.evaluate(environment),
            Expression::BooleanLiteral(literal) => Ok(Object::Boolean(literal.value)),
            Expression::If(if_expr) => if_expr.evaluate(environment),
            Expression::FunctionLiteral(fun) => Ok(Object::Function(FunctionObject {
                parameters: fun.parameters.clone(),
                body: fun.body.clone(),
                environment: environment.clone(),
            })),
            Expression::FunctionCall(fun) => fun.evaluate(environment),
        }
    }
}

impl Expression {
    pub fn debug_str(&self) -> String {
        match self {
            Expression::Identifier(identifier) => identifier.name.clone(),
            Expression::IntegerLiteral(literal) => literal.value.to_string(),
            Expression::StringLiteral(literal) => literal.clone(),
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
            Expression::FunctionCall(fun) => fun.debug_str(),
        }
    }

    fn parse_prefix(parser: &mut Parser, precedence: Precedence) -> Result<Self> {
        let expr = match parser.get_current_token() {
            Token::Ident(name) => Self::Identifier(Identifier { name: name.clone() }),
            Token::Int(value) => Self::IntegerLiteral(IntegerLiteral {
                value: value.parse()?,
            }),
            Token::String(value) => Self::StringLiteral(value.clone()),
            Token::Bang | Token::Minus => {
                Self::PrefixOperator(PrefixOperator::parse(parser, precedence, None)?)
            }
            Token::True => Self::BooleanLiteral(BooleanLiteral { value: true }),
            Token::False => Self::BooleanLiteral(BooleanLiteral { value: false }),
            Token::Lparen => Self::parse_grouped(parser)?,
            Token::If => Self::If(IfExpression::parse(parser, precedence, None)?),
            Token::Function => {
                Self::FunctionLiteral(FunctionLiteral::parse(parser, precedence, None)?)
            }
            token => return Err(Error::NotAnExpression(token.clone())),
        };

        Ok(expr)
    }

    fn parse_infix(parser: &mut Parser, precedence: Precedence, left: Expression) -> Result<Self> {
        let expr = match parser.get_current_token() {
            Token::Plus
            | Token::Minus
            | Token::Slash
            | Token::Asterisk
            | Token::Eq
            | Token::NotEq
            | Token::Lt
            | Token::Gt => {
                Self::InfixOperator(InfixOperator::parse(parser, precedence, Some(left))?)
            }
            Token::Lparen => {
                Self::FunctionCall(FunctionCall::parse(parser, precedence, Some(left))?)
            }
            _ => left,
        };

        Ok(expr)
    }

    fn parse_grouped(parser: &mut Parser) -> Result<Self> {
        parser.step();

        let expression = Expression::parse(parser, Precedence::Lowest, None)?;

        if *parser.get_peek_token() == Token::Rparen {
            parser.step();
            Ok(expression)
        } else {
            Err(Error::UnexpectedToken(parser.get_peek_token().clone()))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    pub name: String,
}

impl TryFrom<&Token> for Identifier {
    type Error = Error;

    fn try_from(token: &Token) -> Result<Self> {
        match token {
            Token::Ident(name) => Ok(Self { name: name.clone() }),
            _ => Err(Error::UnexpectedToken(token.clone())),
        }
    }
}

impl Evaluate for Identifier {
    fn evaluate(&self, environment: &mut Environment) -> evaluate::Result<Object> {
        match environment.get(&self.name) {
            Some(val) => Ok(val.clone()),
            None => Err(evaluate::Error::UnknownIdentifier(self.name.clone())),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct IntegerLiteral {
    pub value: i64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BooleanLiteral {
    pub value: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrefixOperator {
    pub operator: PrefixOperatorKind,
    pub right: Box<Expression>,
}

impl Parse for PrefixOperator {
    fn parse(parser: &mut Parser, _: Precedence, _: Option<Expression>) -> Result<Self> {
        let operator = PrefixOperatorKind::try_from(parser.get_current_token())?;
        parser.step();

        Ok(Self {
            operator,
            right: Box::new(Expression::parse(parser, Precedence::Prefix, None)?),
        })
    }
}

impl Evaluate for PrefixOperator {
    fn evaluate(&self, environment: &mut Environment) -> evaluate::Result<Object> {
        let right = self.right.evaluate(environment)?;

        match self.operator {
            PrefixOperatorKind::Not => match right {
                Object::Boolean(bool) => Ok(Object::Boolean(!bool)),
                Object::Null => Ok(Object::Boolean(true)),
                _ => Ok(Object::Boolean(false)),
            },
            PrefixOperatorKind::Negative => match right {
                Object::Integer(value) => Ok(Object::Integer(-value)),
                _ => Err(evaluate::Error::UnknownOperator(format!(
                    "-{}",
                    right.data_type()
                ))),
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct InfixOperator {
    pub operator: InfixOperatorKind,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl Parse for InfixOperator {
    fn parse(parser: &mut Parser, _: Precedence, left: Option<Expression>) -> Result<Self> {
        let Some(left) = left else {
            return Err(Error::ExpectedLeftExpression);
        };

        let operator = InfixOperatorKind::try_from(parser.get_current_token())?;
        let precedence = Precedence::from(parser.get_current_token());

        parser.step();

        let right = Expression::parse(parser, precedence, None)?;

        Ok(Self {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        })
    }
}

impl Evaluate for InfixOperator {
    fn evaluate(&self, environment: &mut Environment) -> evaluate::Result<Object> {
        let left = self.left.evaluate(environment)?;
        let right = self.right.evaluate(environment)?;

        if let (Object::Integer(left), Object::Integer(right)) = (&left, &right) {
            let res = match self.operator {
                InfixOperatorKind::Add => Object::Integer(left + right),
                InfixOperatorKind::Subtract => Object::Integer(left - right),
                InfixOperatorKind::Multiply => Object::Integer(left * right),
                InfixOperatorKind::Divide => Object::Integer(left / right),
                InfixOperatorKind::Equal => Object::Boolean(left == right),
                InfixOperatorKind::NotEqual => Object::Boolean(left != right),
                InfixOperatorKind::GreaterThan => Object::Boolean(left > right),
                InfixOperatorKind::LessThan => Object::Boolean(left < right),
            };

            return Ok(res);
        }

        if let (Object::Boolean(left_bool), Object::Boolean(right_bool)) = (&left, &right) {
            return match self.operator {
                InfixOperatorKind::Equal => Ok(Object::Boolean(left_bool == right_bool)),
                InfixOperatorKind::NotEqual => Ok(Object::Boolean(left_bool != right_bool)),
                _ => Err(evaluate::Error::UnknownOperator(format!(
                    "{} {} {}",
                    left.data_type(),
                    self.operator.debug_str(),
                    right.data_type()
                ))),
            };
        }

        if let (Object::String(left_str), Object::String(right_str)) = (&left, &right) {
            return match self.operator {
                InfixOperatorKind::Add => Ok(Object::String(format!("{}{}", left_str, right_str))),
                InfixOperatorKind::Equal => Ok(Object::Boolean(left_str == right_str)),
                InfixOperatorKind::NotEqual => Ok(Object::Boolean(left_str != right_str)),
                _ => Err(evaluate::Error::UnknownOperator(format!(
                    "{} {} {}",
                    left.data_type(),
                    self.operator.debug_str(),
                    right.data_type()
                ))),
            };
        }

        if left.data_type() != right.data_type() {
            return Err(evaluate::Error::TypeMismatch(format!(
                "{} {} {}",
                left.data_type(),
                self.operator.debug_str(),
                right.data_type()
            )));
        }

        Err(evaluate::Error::UnknownOperator(format!(
            "{} {} {}",
            left.data_type(),
            self.operator.debug_str(),
            right.data_type()
        )))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub consequence: BlockStatement,
    pub alternative: BlockStatement,
}

impl Parse for IfExpression {
    fn parse(parser: &mut Parser, precedence: Precedence, _: Option<Expression>) -> Result<Self> {
        if *parser.get_peek_token() != Token::Lparen {
            return Err(Error::UnexpectedToken(parser.get_peek_token().clone()));
        }
        parser.step();
        parser.step();

        let condition = Expression::parse(parser, Precedence::Lowest, None)?;

        if *parser.get_peek_token() != Token::Rparen {
            return Err(Error::UnexpectedToken(parser.get_peek_token().clone()));
        }
        parser.step();

        if *parser.get_peek_token() != Token::Lsquigly {
            return Err(Error::UnexpectedToken(parser.get_peek_token().clone()));
        }
        parser.step();

        let consequence = BlockStatement::parse(parser, precedence, None)?;

        if *parser.get_current_token() != Token::Rsquigly {
            return Err(Error::UnexpectedToken(parser.get_current_token().clone()));
        }

        let mut expr = Self {
            condition: Box::new(condition),
            consequence,
            alternative: BlockStatement { statements: vec![] },
        };

        if *parser.get_peek_token() == Token::Else {
            parser.step();

            if *parser.get_peek_token() != Token::Lsquigly {
                return Err(Error::UnexpectedToken(parser.get_peek_token().clone()));
            }
            parser.step();

            expr.alternative = BlockStatement::parse(parser, precedence, None)?;

            if *parser.get_current_token() != Token::Rsquigly {
                return Err(Error::UnexpectedToken(parser.get_current_token().clone()));
            }
        }

        Ok(expr)
    }
}

impl Evaluate for IfExpression {
    fn evaluate(&self, environment: &mut Environment) -> evaluate::Result<Object> {
        let condition = self.condition.evaluate(environment)?;

        if condition.is_truthy() {
            self.consequence.evaluate(environment)
        } else {
            self.alternative.evaluate(environment)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
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

impl Parse for BlockStatement {
    fn parse(parser: &mut Parser, precedence: Precedence, _: Option<Expression>) -> Result<Self> {
        parser.step();

        let mut statements = Vec::new();

        while *parser.get_current_token() != Token::Rsquigly
            && *parser.get_current_token() != Token::Eof
        {
            let stmt = Statement::parse(parser, precedence, None)?;
            statements.push(stmt);

            parser.step();
        }

        Ok(Self { statements })
    }
}

impl Evaluate for BlockStatement {
    fn evaluate(&self, environment: &mut Environment) -> evaluate::Result<Object> {
        let mut res = Object::Null;

        for stmt in &self.statements {
            res = stmt.evaluate(environment)?;

            if matches!(res, Object::Return(_)) {
                return Ok(res);
            }
        }

        Ok(res)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionLiteral {
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}

impl FunctionLiteral {
    pub fn debug_str(&self) -> String {
        format!(
            "fn ({}) {{{}}}",
            self.parameters
                .iter()
                .map(|ident| ident.name.to_owned())
                .collect::<Vec<String>>()
                .join(", "),
            self.body.debug_str(),
        )
    }

    fn parse_function_parameters(parser: &mut Parser) -> Result<Vec<Identifier>> {
        let mut identifiers = vec![];

        parser.step();
        if *parser.get_current_token() == Token::Rparen {
            return Ok(identifiers);
        }

        identifiers.push(Identifier::try_from(parser.get_current_token())?);

        while *parser.get_peek_token() == Token::Comma {
            parser.step();
            parser.step();

            identifiers.push(Identifier::try_from(parser.get_current_token())?);
        }

        if *parser.get_peek_token() != Token::Rparen {
            return Err(Error::UnexpectedToken(parser.get_peek_token().clone()));
        }
        parser.step();

        Ok(identifiers)
    }
}

impl Parse for FunctionLiteral {
    fn parse(parser: &mut Parser, precedence: Precedence, _: Option<Expression>) -> Result<Self> {
        if *parser.get_peek_token() != Token::Lparen {
            return Err(Error::UnexpectedToken(parser.get_peek_token().clone()));
        }
        parser.step();

        let parameters = Self::parse_function_parameters(parser)?;

        if *parser.get_peek_token() != Token::Lsquigly {
            return Err(Error::UnexpectedToken(parser.get_peek_token().clone()));
        }
        parser.step();

        let body = BlockStatement::parse(parser, precedence, None)?;

        Ok(Self { parameters, body })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}

impl Parse for FunctionCall {
    fn parse(parser: &mut Parser, _: Precedence, left: Option<Expression>) -> Result<Self> {
        let Some(left) = left else {
            return Err(Error::ExpectedLeftExpression);
        };

        Ok(Self {
            function: Box::new(left),
            arguments: Self::parse_call_arguments(parser)?,
        })
    }
}

impl Evaluate for FunctionCall {
    fn evaluate(&self, environment: &mut Environment) -> evaluate::Result<Object> {
        let function = self.function.evaluate(environment)?;
        let Object::Function(function) = function else {
            return Err(evaluate::Error::NotAFunction(
                function.data_type().to_string(),
            ));
        };

        let args = self
            .arguments
            .iter()
            .map(|expr| expr.evaluate(environment))
            .collect::<evaluate::Result<Vec<_>>>()?;

        let mut extended_env = function.environment.extend();

        for (index, param) in function.parameters.iter().enumerate() {
            extended_env.set(param.name.clone(), args[index].clone())
        }

        let evaluated = function.body.evaluate(&mut extended_env)?;
        match &evaluated {
            Object::Return(obj) => Ok(*obj.clone()),
            _ => Ok(evaluated.clone()),
        }
    }
}

impl FunctionCall {
    pub fn debug_str(&self) -> String {
        format!(
            "{}({})",
            self.function.debug_str(),
            self.arguments
                .iter()
                .map(|arg| arg.debug_str())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    fn parse_call_arguments(parser: &mut Parser) -> Result<Vec<Expression>> {
        let mut arguments = vec![];

        parser.step();
        if *parser.get_current_token() == Token::Rparen {
            return Ok(arguments);
        }

        arguments.push(Expression::parse(parser, Precedence::Lowest, None)?);

        while *parser.get_peek_token() == Token::Comma {
            parser.step();
            parser.step();

            arguments.push(Expression::parse(parser, Precedence::Lowest, None)?);
        }

        if *parser.get_peek_token() != Token::Rparen {
            return Err(Error::UnexpectedToken(parser.get_peek_token().clone()));
        }
        parser.step();

        Ok(arguments)
    }
}
