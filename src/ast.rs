#[derive(Debug)]
pub enum Statement {
    Let(LetStatement),
    Empty,
}

#[derive(Debug)]
pub enum Expression {
    Identifier(Identifier),
    Empty,
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct LetStatement {
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Debug)]
pub struct Identifier {
    pub name: String,
}
