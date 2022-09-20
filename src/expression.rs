use crate::{token::Token, value::Value};

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Expression,
    pub operator: Token,
    pub right: Expression,
}

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub callee: Expression,
    pub paren: Token,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct GroupingExpression {
    pub expr: Expression,
}

#[derive(Debug, Clone)]
pub struct LiteralExpression {
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operator: Token,
    pub right: Expression,
}

#[derive(Debug, Clone)]
pub struct VariableExpression {
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct AssignExpression {
    pub name: Token,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct LogicalExpression {
    pub left: Expression,
    pub operator: Token,
    pub right: Expression,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary(Box<BinaryExpression>),
    Call(Box<CallExpression>),
    Grouping(Box<GroupingExpression>),
    Literal(Box<LiteralExpression>),
    Unary(Box<UnaryExpression>),
    Variable(Box<VariableExpression>),
    Assign(Box<AssignExpression>),
    Logical(Box<LogicalExpression>),
}
