use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct FunctionBody {
    pub vars: Vec<Variable>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<Argument>,
    pub body: FunctionBody,
    pub ret_val: Value,
}
