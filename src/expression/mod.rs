mod expressions;

pub use expressions::*;

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
    Get(Box<GetExpression>),
    Set(Box<SetExpression>),
}

impl Expression {
    pub fn get_scope_depth(&self) -> Option<u32> {
        match self {
            Expression::Call(x) => x.scope_depth,
            Expression::Variable(x) => x.scope_depth,
            Expression::Assign(x) => x.scope_depth,
            _ => None,
        }
    }

    pub fn set_scope_depth(&mut self, value: u32) {
        match self {
            Expression::Call(x) => x.scope_depth = Some(value),
            Expression::Variable(x) => x.scope_depth = Some(value),
            Expression::Assign(x) => x.scope_depth = Some(value),
            _ => (),
        }
    }
}
