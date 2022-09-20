use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::{Env, Environment},
    error::{get_err_handler, Result},
    expression::{
        AssignExpression, BinaryExpression, CallExpression, Expression, LogicalExpression,
        UnaryExpression, VariableExpression,
    },
    statement::{
        BlockStatement, ExpressionStatement, FunctionStatement, IfStatement, PrintStatement,
        ReturnStatement, Statement, VarStatement, WhileStatement,
    },
    token::{Token, TokenType},
    value::{Function, NativeFunction, Value},
};

pub struct Interpreter {
    globals: Env,
    env: Env,
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

    pub fn get_global_env(&self) -> Env {
        self.globals.clone()
    }

    pub fn get_current_env(&self) -> Env {
        self.env.clone()
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
            Value::Callable(_) => false,
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
        self.env.borrow().get(&expr.name)
    }

    fn eval_assign(&mut self, expr: &AssignExpression) -> Result<Value> {
        let value = self.evaluate(&expr.value)?;
        self.env.borrow_mut().assign(&expr.name, value.clone())?;
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
        let callable = match callee {
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
        callable.call(self, args)
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

    fn execute_print_statement(&mut self, statement: &PrintStatement) -> Result<()> {
        let val = self.evaluate(&statement.expr)?;
        println!("{}", val);
        Ok(())
    }

    fn execute_expression_statement(&mut self, statement: &ExpressionStatement) -> Result<()> {
        self.evaluate(&statement.expr)?;
        Ok(())
    }

    fn execute_var_statement(&mut self, statement: &VarStatement) -> Result<()> {
        let mut value = Value::None;
        if let Some(init) = &statement.initializer {
            value = self.evaluate(&init)?;
        }
        self.env
            .borrow_mut()
            .define(statement.name.lexeme.clone(), value);
        Ok(())
    }

    fn execute_function_statement(&mut self, statement: &FunctionStatement) -> Result<()> {
        let function = Function::new(statement.clone());
        self.env.borrow_mut().define(
            statement.name.lexeme.clone(),
            Value::Callable(Box::new(function)),
        );
        Ok(())
    }

    pub fn execute_block(&mut self, statements: &[Statement], env: Env) -> Result<()> {
        let previous = self.env.clone();
        self.env = env;
        for statement in statements {
            self.execute(statement).ok();
        }
        self.env = previous;
        Ok(())
    }

    fn execute_block_statement(&mut self, statement: &BlockStatement) -> Result<()> {
        self.execute_block(
            &statement.statements,
            Rc::new(RefCell::new(Environment::new(Some(self.env.clone())))),
        )
    }

    fn execute_if_statement(&mut self, statement: &IfStatement) -> Result<()> {
        if Self::is_truthy(&self.evaluate(&statement.condition)?) {
            self.execute_block_statement(&statement.then_branch)?;
        } else if let Some(x) = &statement.else_branch {
            self.execute_block_statement(&x)?;
        }
        Ok(())
    }

    fn execute_while_statement(&mut self, statement: &WhileStatement) -> Result<()> {
        while Self::is_truthy(&self.evaluate(&statement.condition)?) {
            self.execute_block_statement(&statement.body)?;
        }
        Ok(())
    }

    fn execute_return_statement(&mut self, statement: &ReturnStatement) -> Result<()> {
        //TODO
        todo!()
    }

    fn execute(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Print(x) => self.execute_print_statement(x),
            Statement::Expression(x) => self.execute_expression_statement(x),
            Statement::Var(x) => self.execute_var_statement(x),
            Statement::Function(x) => self.execute_function_statement(x),
            Statement::Block(x) => self.execute_block_statement(x),
            Statement::If(x) => self.execute_if_statement(x),
            Statement::While(x) => self.execute_while_statement(x),
            Statement::Return(x) => self.execute_return_statement(x),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) {
        for statement in statements {
            if let Err(x) = self.execute(&statement) {
                get_err_handler().runtime_error(x);
            }
        }
    }
}
