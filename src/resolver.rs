use std::collections::HashMap;

use crate::{
    error::get_err_handler,
    expression::{
        BinaryExpression, CallExpression, Expression, GroupingExpression, LogicalExpression,
        UnaryExpression,
    },
    interpreter::Interpreter,
    statement::{
        BlockStatement, ExpressionStatement, FunctionStatement, IfStatement, ReturnStatement,
        Statement, VarStatement, WhileStatement,
    },
    token::Token,
    value::FunctionKind,
};

pub struct Resolver<'a> {
    interpreter: &'a Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: Option<FunctionKind>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            current_function: None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.first_mut().unwrap();
        if scope.contains_key(&name.lexeme) {
            get_err_handler().error(name.clone(), "Variable already exists in this scope.");
        }
        scope.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.first_mut().unwrap();
        *scope.get_mut(&name.lexeme).unwrap() = true;
    }

    fn resolve_block_statement(&mut self, statement: &mut BlockStatement) {
        self.begin_scope();
        self.resolve(statement.statements.iter_mut());
        self.end_scope();
    }

    fn resolve_var_statement(&mut self, statement: &mut VarStatement) {
        self.declare(&statement.name);
        if let Some(x) = &mut statement.initializer {
            self.resolve_expression(x);
        }
        self.define(&statement.name);
    }

    fn resolve_function(&mut self, function: &mut FunctionStatement, kind: FunctionKind) {
        let enclosing_function = self.current_function;
        self.current_function = Some(kind);

        self.begin_scope();
        for param in &function.params {
            self.declare(param);
            self.define(param);
        }
        self.resolve(function.body.iter_mut());
        self.end_scope();

        self.current_function = enclosing_function;
    }

    fn resolve_function_statement(&mut self, statement: &mut FunctionStatement) {
        self.declare(&statement.name);
        self.define(&statement.name);
        self.resolve_function(statement, FunctionKind::Function);
    }

    fn resolve_expression_statement(&mut self, statement: &mut ExpressionStatement) {
        self.resolve_expression(&mut statement.expr);
    }

    fn resolve_if_statement(&mut self, statement: &mut IfStatement) {
        self.resolve_expression(&mut statement.condition);
        self.resolve_block_statement(&mut statement.then_branch);
        if let Some(x) = &mut statement.else_branch {
            self.resolve_block_statement(x);
        }
    }

    fn resolve_return_statement(&mut self, statement: &mut ReturnStatement) {
        if let None = self.current_function {
            get_err_handler().error(
                statement.keyword.clone(),
                "Can't return from top-level code.",
            );
        }

        if let Some(x) = &mut statement.expr {
            self.resolve_expression(x);
        }
    }

    fn resolve_while_statement(&mut self, statement: &mut WhileStatement) {
        self.resolve_expression(&mut statement.condition);
        self.resolve_block_statement(&mut statement.body);
    }

    fn resolve_local(&mut self, expression: &mut Expression, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter
                    .resolve(expression, (self.scopes.len() - 1 - i) as u32);
                break;
            }
        }
    }

    fn resolve_var_expression(&mut self, expression: &mut Expression) {
        let var_expr = if let Expression::Variable(x) = expression {
            x
        } else {
            return;
        };

        let top_scope = self.scopes.first();
        if let Some(x) = top_scope {
            let is_var_in_top = x.get(&var_expr.name.lexeme);
            if !self.scopes.is_empty()
                && (is_var_in_top.is_some() && !x.get(&var_expr.name.lexeme).unwrap())
            {
                get_err_handler().error(
                    var_expr.name.clone(),
                    "Can't read local variable in its own initializer.",
                );
            }
        }

        let name = var_expr.name.clone();
        self.resolve_local(expression, &name);
    }

    fn resolve_assign_expression(&mut self, expression: &mut Expression) {
        let assign_expr = if let Expression::Assign(x) = expression {
            x
        } else {
            return;
        };

        self.resolve_expression(&mut assign_expr.value);
        let name = assign_expr.name.clone();
        self.resolve_local(expression, &name);
    }

    fn resolve_binary_expression(&mut self, expression: &mut BinaryExpression) {
        self.resolve_expression(&mut expression.left);
        self.resolve_expression(&mut expression.right);
    }

    fn resolve_call_expression(&mut self, expression: &mut CallExpression) {
        self.resolve_expression(&mut expression.callee);
        for arg in expression.args.iter_mut() {
            self.resolve_expression(arg);
        }
    }

    fn resolve_grouping_expression(&mut self, expression: &mut GroupingExpression) {
        self.resolve_expression(&mut expression.expr);
    }

    fn resolve_unary_expression(&mut self, expression: &mut UnaryExpression) {
        self.resolve_expression(&mut expression.right);
    }

    fn resolve_logical_expression(&mut self, expression: &mut LogicalExpression) {
        self.resolve_expression(&mut expression.left);
        self.resolve_expression(&mut expression.right);
    }

    fn resolve_expression(&mut self, expression: &mut Expression) {
        match expression {
            Expression::Variable(_) => self.resolve_var_expression(expression),
            Expression::Assign(_) => self.resolve_assign_expression(expression),
            Expression::Binary(x) => self.resolve_binary_expression(&mut *x),
            Expression::Call(x) => self.resolve_call_expression(&mut *x),
            Expression::Grouping(x) => self.resolve_grouping_expression(&mut *x),
            Expression::Literal(_) => return,
            Expression::Unary(x) => self.resolve_unary_expression(&mut *x),
            Expression::Logical(x) => self.resolve_logical_expression(&mut *x),
        }
    }

    fn resolve_statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::Block(x) => self.resolve_block_statement(x),
            Statement::Var(x) => self.resolve_var_statement(x),
            Statement::Function(x) => self.resolve_function_statement(x),
            Statement::Expression(x) => self.resolve_expression_statement(x),
            Statement::If(x) => self.resolve_if_statement(x),
            Statement::Return(x) => self.resolve_return_statement(x),
            Statement::While(x) => self.resolve_while_statement(x),
        }
    }

    pub fn resolve<'b>(&mut self, statements: impl Iterator<Item = &'b mut Statement>) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }
}
