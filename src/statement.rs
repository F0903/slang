use crate::{expression::Expression, token::Token};

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub expr: Expression,
}

#[derive(Debug, Clone)]
pub struct PrintStatement {
    pub expr: Expression,
}

#[derive(Debug, Clone)]
pub struct VarStatement {
    pub name: Token,
    pub initializer: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct FunctionStatement {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: BlockStatement,
    pub else_branch: Option<BlockStatement>,
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: BlockStatement,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub keyword: Token,
    pub expr: Option<Expression>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(ExpressionStatement),
    Print(PrintStatement),
    Var(VarStatement),
    Function(FunctionStatement),
    Block(BlockStatement),
    If(IfStatement),
    While(WhileStatement),
    Return(ReturnStatement),
}
