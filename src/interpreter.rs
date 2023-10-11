use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::{EnvPtr, Environment, GetDeep},
    error::{get_err_handler, Result},
    expression::{
        AssignExpression, BinaryExpression, CallExpression, Expression, LogicalExpression,
        UnaryExpression, VariableExpression,
    },
    statement::{
        BlockStatement, ClassStatement, ExpressionStatement, FunctionStatement, IfStatement,
        ReturnStatement, Statement, VarStatement, WhileStatement,
    },
    token::{Token, TokenType},
    value::{Class, Function, FunctionResult, NativeFunction, RuntimeOrNativeError, Value},
};

pub enum MaybeReturn {
    Normal(Value),
    Return(Value),
}

impl From<()> for MaybeReturn {
    fn from(_: ()) -> Self {
        Self::Normal(Value::None)
    }
}

pub struct Interpreter {
    globals: EnvPtr,
    env: EnvPtr,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new(None)));
        Self {
            globals: globals.clone(),
            env: globals,
        }
    }

    pub fn register_native(&self, func: NativeFunction) {
        let mut env = self.env.borrow_mut();
        env.define(func.get_name().to_owned(), Value::Callable(Box::new(func)));
    }

    pub fn get_global_env(&self) -> EnvPtr {
        self.globals.clone()
    }

    pub fn get_current_env(&self) -> EnvPtr {
        self.env.clone()
    }

    pub fn resolve(&self, expression: &mut Expression, depth: u32) {
        expression.set_scope_depth(depth);
    }

    pub fn look_up_variable(&self, name: &Token, expr: &VariableExpression) -> Result<Value> {
        let distance = expr.scope_depth;
        if let Some(x) = distance {
            Ok(self.env.get_at(x, &name.lexeme))
        } else {
            self.globals.borrow().get(name)
        }
    }

    fn is_truthy(val: &Value) -> bool {
        match val {
            Value::None => false,
            Value::Boolean(x) => *x,
            _ => true,
        }
    }

    fn error<T>(token: Token, msg: impl ToString) -> Result<T> {
        Err((token, msg).into())
    }

    fn eval_unary(&mut self, expr: &UnaryExpression) -> Result<Value> {
        let right = self.evaluate(&expr.right)?;
        let val = match expr.operator.token_type {
            TokenType::Minus => {
                let val = match right {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Minus unary operator can only be used on numbers.",
                        )
                    }
                };
                Value::Number(-val)
            }
            TokenType::Not => Value::Boolean(!Self::is_truthy(&right)),
            _ => {
                return Self::error(
                    expr.operator.clone(),
                    "Minus unary operator can only be used on numbers.",
                )
            }
        };
        Ok(val)
    }

    fn is_equal(a: Value, b: Value) -> bool {
        match a {
            Value::None => match b {
                Value::None => true,
                _ => false,
            },
            Value::Boolean(x) => match b {
                Value::Boolean(y) => y == x,
                _ => false,
            },
            Value::Number(x) => match b {
                Value::Number(y) => x == y,
                _ => false,
            },
            Value::String(x) => match b {
                Value::String(y) => x == y,
                _ => false,
            },
            Value::Callable(_) => false, //TODO
            Value::Class(_) => false,    //TODO
        }
    }

    fn eval_binary(&mut self, expr: &BinaryExpression) -> Result<Value> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;
        let val = match expr.operator.token_type {
            TokenType::Minus => {
                let left_val = match left {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Minus binary operator can only be used on numbers.",
                        )
                    }
                };
                let right_val = match right {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Minus binary operator can only be used on numbers.",
                        )
                    }
                };
                Value::Number(left_val - right_val)
            }
            TokenType::Divide => {
                let left_val = match left {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Divide binary operator can only be used on numbers.",
                        )
                    }
                };
                let right_val = match right {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Divide binary operator can only be used on numbers.",
                        )
                    }
                };
                Value::Number(left_val / right_val)
            }
            TokenType::Multiply => {
                let left_val = match left {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Multiply binary operator can only be used on numbers.",
                        )
                    }
                };
                let right_val = match right {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Multiply binary operator can only be used on numbers.",
                        )
                    }
                };
                Value::Number(left_val * right_val)
            }
            TokenType::Plus => {
                if let Value::String(x) = left {
                    if let Value::String(y) = right {
                        Value::String(x + &y)
                    } else {
                        match right {
                            Value::Number(y) => Value::String(x + &y.to_string()),
                            Value::Boolean(y) => Value::String(x + &y.to_string()),
                            Value::None => Value::String(x + "none"),
                            _ => {
                                return Self::error(
                                    expr.operator.clone(),
                                    "Unknown right operand in string concat.",
                                )
                            }
                        }
                    }
                } else if let Value::Number(x) = left {
                    if let Value::Number(y) = right {
                        Value::Number(x + y)
                    } else {
                        return Self::error(
                            expr.operator.clone(),
                            "Cannot add non-number to number.",
                        );
                    }
                } else {
                    return Self::error(
                        expr.operator.clone(),
                        "Plus binary operator can only be used with strings or numbers",
                    );
                }
            }
            TokenType::Greater => {
                let left_val = match left {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Greater binary operator can only be used on numbers.",
                        )
                    }
                };
                let right_val = match right {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Greater binary operator can only be used on numbers.",
                        )
                    }
                };
                Value::Boolean(left_val > right_val)
            }
            TokenType::GreaterEqual => {
                let left_val = match left {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Greater-or-Equal binary operator can only be used on numbers.",
                        )
                    }
                };
                let right_val = match right {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Greater-or-Equal binary operator can only be used on numbers.",
                        )
                    }
                };
                Value::Boolean(left_val >= right_val)
            }
            TokenType::Less => {
                let left_val = match left {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Less binary operator can only be used on numbers.",
                        )
                    }
                };
                let right_val = match right {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Less binary operator can only be used on numbers.",
                        )
                    }
                };
                Value::Boolean(left_val < right_val)
            }
            TokenType::LessEqual => {
                let left_val = match left {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Less-or-Equal binary operator can only be used on numbers.",
                        )
                    }
                };
                let right_val = match right {
                    Value::Number(x) => x,
                    _ => {
                        return Self::error(
                            expr.operator.clone(),
                            "Less-or-Equal binary operator can only be used on numbers.",
                        )
                    }
                };
                Value::Boolean(left_val <= right_val)
            }
            TokenType::Is => Value::Boolean(Self::is_equal(left, right)),
            _ => {
                return Self::error(
                    expr.operator.clone(),
                    "Unknown operator in binary expression.",
                )
            }
        };
        Ok(val)
    }

    fn eval_variable(&self, expr: &VariableExpression) -> Result<Value> {
        self.look_up_variable(&expr.name, expr)
    }

    fn eval_assign(&mut self, expr: &AssignExpression) -> Result<Value> {
        let value = self.evaluate(&expr.value)?;
        let distance = expr.scope_depth;
        if let Some(x) = distance {
            self.env.assign_at(x, &expr.name, value.clone());
        } else {
            self.globals
                .borrow_mut()
                .assign(&expr.name, value.clone())?;
        }
        Ok(value)
    }

    fn eval_logical(&mut self, expr: &LogicalExpression) -> Result<Value> {
        let left = self.evaluate(&expr.left)?;
        if expr.operator.token_type == TokenType::Or {
            if Self::is_truthy(&left) {
                return Ok(left);
            }
        } else {
            if !Self::is_truthy(&left) {
                return Ok(left);
            }
        }
        Ok(self.evaluate(&expr.right)?)
    }

    fn eval_call(&mut self, expr: &CallExpression) -> Result<Value> {
        let callee = self.evaluate(&expr.callee)?;
        let mut args = vec![];
        for arg in &expr.args {
            args.push(self.evaluate(arg)?);
        }
        let mut callable = match callee {
            Value::Callable(x) => x,
            _ => return Self::error(expr.paren.clone(), "Expected callable object."),
        };
        let arg_num = args.len();
        let arg_needed = callable.get_arity();
        if arg_num != arg_needed {
            return Self::error(
                expr.paren.clone(),
                format!("Exptected {} arguments, but got {}", arg_needed, arg_num),
            );
        }
        match callable.call(self, args) {
            FunctionResult::Ok(x) => Ok(x),
            FunctionResult::Err(e) => match e {
                RuntimeOrNativeError::Runtime(e) => Err(e),
                RuntimeOrNativeError::Native(e) => {
                    get_err_handler().report_native(callable.get_name(), e, expr.paren.line);
                    Ok(Value::None)
                }
            },
        }
    }

    fn evaluate(&mut self, expr: &Expression) -> Result<Value> {
        match expr {
            Expression::Literal(x) => Ok(x.value.clone()),
            Expression::Grouping(x) => self.evaluate(&x.expr),
            Expression::Unary(x) => self.eval_unary(&*x),
            Expression::Binary(x) => self.eval_binary(&*x),
            Expression::Variable(x) => self.eval_variable(&*x),
            Expression::Assign(x) => self.eval_assign(&*x),
            Expression::Logical(x) => self.eval_logical(&*x),
            Expression::Call(x) => self.eval_call(&*x),
        }
    }

    fn execute_expression_statement(
        &mut self,
        statement: &ExpressionStatement,
    ) -> Result<MaybeReturn> {
        self.evaluate(&statement.expr)?;
        Ok(().into())
    }

    fn execute_var_statement(&mut self, statement: &VarStatement) -> Result<MaybeReturn> {
        let mut value = Value::None;
        if let Some(init) = &statement.initializer {
            value = self.evaluate(&init)?;
        }
        self.env
            .borrow_mut()
            .define(statement.name.lexeme.clone(), value);
        Ok(().into())
    }

    fn execute_function_statement(&mut self, statement: &FunctionStatement) -> Result<MaybeReturn> {
        let function = Function::new(statement.clone(), self.env.borrow().clone());
        self.env.borrow_mut().define(
            statement.name.lexeme.clone(),
            Value::Callable(Box::new(function)),
        );
        Ok(().into())
    }

    pub fn execute_block(&mut self, statements: &[Statement], env: EnvPtr) -> Result<MaybeReturn> {
        let previous = self.env.clone();
        self.env = env;
        for statement in statements {
            if let MaybeReturn::Return(x) = self.execute(statement)? {
                return Ok(MaybeReturn::Return(x));
            }
        }
        self.env = previous;
        Ok(().into())
    }

    fn execute_block_statement(&mut self, statement: &BlockStatement) -> Result<MaybeReturn> {
        self.execute_block(
            &statement.statements,
            Rc::new(RefCell::new(Environment::new(Some(self.env.clone())))),
        )
    }

    fn execute_if_statement(&mut self, statement: &IfStatement) -> Result<MaybeReturn> {
        if Self::is_truthy(&self.evaluate(&statement.condition)?) {
            self.execute_block_statement(&statement.then_branch)
        } else if let Some(x) = &statement.else_branch {
            self.execute_block_statement(&x)
        } else {
            Ok(().into())
        }
    }

    fn execute_while_statement(&mut self, statement: &WhileStatement) -> Result<MaybeReturn> {
        while Self::is_truthy(&self.evaluate(&statement.condition)?) {
            match self.execute_block_statement(&statement.body)? {
                MaybeReturn::Return(x) => return Ok(MaybeReturn::Return(x)),
                _ => (),
            };
        }
        Ok(().into())
    }

    fn execute_return_statement(&mut self, statement: &ReturnStatement) -> Result<MaybeReturn> {
        let value = if let Some(x) = &statement.expr {
            self.evaluate(x)?
        } else {
            Value::None
        };
        Ok(MaybeReturn::Return(value))
    }

    fn execute_class_statement(&mut self, statement: &ClassStatement) -> Result<MaybeReturn> {
        let mut env = self.env.borrow_mut();
        env.define(statement.name.lexeme.clone(), Value::None);
        let class = Class::new(statement.name.lexeme.clone());
        env.assign(&statement.name, Value::Class(class))?;
        Ok(MaybeReturn::Normal(Value::None))
    }

    fn execute(&mut self, statement: &Statement) -> Result<MaybeReturn> {
        match statement {
            Statement::Expression(x) => self.execute_expression_statement(x),
            Statement::Var(x) => self.execute_var_statement(x),
            Statement::Function(x) => self.execute_function_statement(x),
            Statement::Block(x) => self.execute_block_statement(x),
            Statement::If(x) => self.execute_if_statement(x),
            Statement::While(x) => self.execute_while_statement(x),
            Statement::Return(x) => self.execute_return_statement(x),
            Statement::Class(x) => self.execute_class_statement(x),
        }
    }

    pub fn interpret(&mut self, statements: impl Iterator<Item = Statement>) {
        for statement in statements {
            if let Err(x) = self.execute(&statement) {
                get_err_handler().runtime_error(x);
            }
        }
    }
}
