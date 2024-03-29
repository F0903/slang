use core::panic;
use std::iter::Peekable;

use crate::{
    error::{get_err_handler, Result, RuntimeError},
    expression::{
        AssignExpression, BinaryExpression, CallExpression, Expression, GetExpression,
        GroupingExpression, LiteralExpression, LogicalExpression, SetExpression, UnaryExpression,
        VariableExpression,
    },
    statement::{
        BlockStatement, ClassStatement, ExpressionStatement, FunctionStatement, IfStatement,
        ReturnStatement, Statement, VarStatement, WhileStatement,
    },
    token::{Token, TokenType},
    value::{FunctionKind, Value},
};

const MAX_FUNC_ARG_COUNT: usize = 255;

pub struct Parser<I: Iterator<Item = Token>> {
    tokens: Peekable<I>,
    last_token: Option<Token>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            last_token: None,
        }
    }

    fn error<T>(token: &Token, msg: &str) -> Result<T> {
        get_err_handler().report(token.line, msg);
        Err(RuntimeError::new(token.clone(), msg))
    }

    fn check(&mut self, typ: TokenType) -> bool {
        if self.at_end() {
            false
        } else {
            let next = self.peek();
            next.token_type == typ
        }
    }

    fn peek(&mut self) -> &Token {
        self.tokens.peek().unwrap()
    }

    fn at_end(&mut self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn previous(&self) -> Token {
        self.last_token.clone().unwrap()
    }

    fn advance(&mut self) -> Token {
        let ret = self.peek().clone();
        let next = self.tokens.next().unwrap();
        self.last_token = Some(next);
        return ret;
    }

    fn match_next_token(&mut self, types: &[TokenType]) -> bool {
        for typ in types {
            if self.check(*typ) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn synchronize(&mut self) {
        while !self.at_end() {
            let previous = self.advance();
            // Advance until a statement end is found.
            if previous.token_type == TokenType::StatementEnd {
                return;
            } else if self.peek().token_type == TokenType::StatementEnd {
                self.advance();
                return;
            }
        }
    }

    fn consume_if(&mut self, token_type: TokenType, err_msg: &str) -> Result<Token> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Self::error(self.peek(), err_msg)
    }

    fn handle_primary(&mut self) -> Result<Expression> {
        if self.match_next_token(&[TokenType::Identifier]) {
            Ok(Expression::Variable(Box::new(VariableExpression {
                name: self.previous().clone(),
                scope_depth: None,
            })))
        } else if self.match_next_token(&[TokenType::False]) {
            Ok(Expression::Literal(Box::new(LiteralExpression {
                value: Value::Boolean(false),
            })))
        } else if self.match_next_token(&[TokenType::True]) {
            Ok(Expression::Literal(Box::new(LiteralExpression {
                value: Value::Boolean(true),
            })))
        } else if self.match_next_token(&[TokenType::None]) {
            Ok(Expression::Literal(Box::new(LiteralExpression {
                value: Value::None,
            })))
        } else if self.match_next_token(&[TokenType::Number, TokenType::String]) {
            Ok(Expression::Literal(Box::new(LiteralExpression {
                value: self.previous().literal,
            })))
        } else if self.match_next_token(&[TokenType::ParenOpen]) {
            let expr = self.handle_expression()?;
            self.consume_if(TokenType::ParenClose, "Expected ')' after expression.")?;
            Ok(Expression::Grouping(Box::new(GroupingExpression { expr })))
        } else {
            Self::error(self.peek(), "Expected an expression.")
        }
    }

    fn handle_postfix(&mut self) -> Result<Expression> {
        let primary = self.handle_primary()?;
        if let Expression::Variable(x) = primary {
            if self.match_next_token(&[TokenType::MinusMinus, TokenType::PlusPlus]) {
                let prev_token = self.previous();
                let op_type = match prev_token.token_type {
                    TokenType::MinusMinus => TokenType::Minus,
                    TokenType::PlusPlus => TokenType::Plus,
                    _ => return Self::error(&prev_token, "Unknown token in postfix operator."),
                };
                let operator = Token {
                    token_type: op_type,
                    ..prev_token
                };
                let start = Expression::Assign(Box::new(AssignExpression {
                    name: x.name.clone(),
                    value: Expression::Binary(Box::new(BinaryExpression {
                        left: Expression::Variable(x),
                        operator,
                        right: Expression::Literal(Box::new(LiteralExpression {
                            value: Value::Number(1.0),
                        })),
                    })),
                    scope_depth: None,
                }));
                return Ok(start);
            }
            return Ok(Expression::Variable(x));
        }
        Ok(primary)
    }

    fn finish_call(&mut self, callee: Expression) -> Result<Expression> {
        let mut args = vec![];
        if !self.check(TokenType::ParenClose) {
            loop {
                if args.len() > MAX_FUNC_ARG_COUNT {
                    return Self::error(self.peek(), "Can't have more that 255 arguments.");
                }
                args.push(self.handle_expression()?);
                if !self.match_next_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume_if(TokenType::ParenClose, "Expected ')' after call arguments.")?;
        Ok(Expression::Call(Box::new(CallExpression {
            callee,
            args,
            paren,
            scope_depth: None,
        })))
    }

    fn handle_call(&mut self) -> Result<Expression> {
        let mut expr = self.handle_postfix()?;
        loop {
            if self.match_next_token(&[TokenType::ParenOpen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_next_token(&[TokenType::Dot]) {
                let name =
                    self.consume_if(TokenType::Identifier, "Expected property name after '.'")?;
                expr = Expression::Get(Box::new(GetExpression { object: expr, name }));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn handle_unary(&mut self) -> Result<Expression> {
        if self.match_next_token(&[TokenType::Not, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.handle_unary()?;
            return Ok(Expression::Unary(Box::new(UnaryExpression {
                operator,
                right,
            })));
        }
        self.handle_call()
    }

    fn handle_factor(&mut self) -> Result<Expression> {
        let mut expr = self.handle_unary()?;
        while self.match_next_token(&[TokenType::Divide, TokenType::Multiply]) {
            let operator = self.previous();
            let right = self.handle_unary()?;
            expr = Expression::Binary(Box::new(BinaryExpression {
                left: expr,
                operator,
                right,
            }));
        }
        Ok(expr)
    }

    fn handle_term(&mut self) -> Result<Expression> {
        let mut expr = self.handle_factor()?;
        while self.match_next_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.handle_factor()?;
            expr = Expression::Binary(Box::new(BinaryExpression {
                left: expr,
                operator,
                right,
            }));
        }
        Ok(expr)
    }

    fn handle_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.handle_term()?;
        while self.match_next_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.handle_term()?;
            expr = Expression::Binary(Box::new(BinaryExpression {
                left: expr,
                operator: operator.clone(),
                right,
            }));
        }
        Ok(expr)
    }

    fn handle_equality(&mut self) -> Result<Expression> {
        let mut expr = self.handle_comparison()?;
        while self.match_next_token(&[TokenType::Is, TokenType::Not]) {
            let operator = self.previous();
            let right = self.handle_comparison()?;
            expr = Expression::Binary(Box::new(BinaryExpression {
                left: expr,
                operator: operator.clone(),
                right,
            }));
        }
        Ok(expr)
    }

    fn handle_and(&mut self) -> Result<Expression> {
        let mut expr = self.handle_equality()?;
        while self.match_next_token(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.handle_equality()?;
            expr = Expression::Logical(Box::new(LogicalExpression {
                left: expr,
                operator,
                right,
            }))
        }
        Ok(expr)
    }

    fn handle_or(&mut self) -> Result<Expression> {
        let mut expr = self.handle_and()?;
        while self.match_next_token(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.handle_and()?;
            expr = Expression::Logical(Box::new(LogicalExpression {
                left: expr,
                operator,
                right,
            }));
        }
        Ok(expr)
    }

    fn handle_assignment(&mut self) -> Result<Expression> {
        let expr = self.handle_or()?;
        if self.match_next_token(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.handle_assignment()?;
            if let Expression::Variable(x) = expr {
                let name = x.name;
                return Ok(Expression::Assign(Box::new(AssignExpression {
                    name,
                    value,
                    scope_depth: None,
                })));
            }

            // Dont throw, just report
            Self::error::<RuntimeError>(&equals, "Invalid assignment target.").ok();
        } else if self.match_next_token(&[TokenType::PlusEqual, TokenType::MinusEqual]) {
            let prev = self.previous();
            let token_type = match prev.token_type {
                TokenType::PlusEqual => TokenType::Plus,
                TokenType::MinusEqual => TokenType::Minus,
                _ => return Self::error(&prev, "Unknown operator type in +=/-= operator."),
            };
            let operator = Token { token_type, ..prev };
            let value = self.handle_assignment()?;
            if let Expression::Variable(x) = expr {
                let name = x.name;
                return Ok(Expression::Assign(Box::new(AssignExpression {
                    name: name.clone(),
                    value: Expression::Binary(Box::new(BinaryExpression {
                        left: Expression::Variable(Box::new(VariableExpression {
                            name,
                            scope_depth: None,
                        })),
                        operator,
                        right: value,
                    })),
                    scope_depth: None,
                })));
            }
        } else if let Expression::Get(x) = expr {
            let value = self.handle_assignment()?;
            return Ok(Expression::Set(Box::new(SetExpression {
                name: x.name,
                object: x.object,
                value,
            })));
        }
        Ok(expr)
    }

    fn handle_expression(&mut self) -> Result<Expression> {
        self.handle_assignment()
    }

    fn handle_expression_statement(&mut self) -> Result<Statement> {
        let expr = self.handle_expression()?;
        self.consume_if(
            TokenType::StatementEnd,
            "Expected statement end after expression.",
        )?;
        Ok(Statement::Expression(ExpressionStatement { expr }))
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>> {
        let mut statements = vec![];
        while !self.check(TokenType::BraceClose) && !self.at_end() {
            statements.push(self.handle_declaration()?);
        }
        self.consume_if(TokenType::BraceClose, "Expected '}' after block.")?;
        self.consume_if(
            TokenType::StatementEnd,
            "Expected statement end after block.",
        )?;
        Ok(statements)
    }

    fn handle_block_statement(&mut self) -> Result<Statement> {
        Ok(Statement::Block(BlockStatement {
            statements: self.parse_block()?,
        }))
    }

    fn handle_if_statement(&mut self) -> Result<Statement> {
        let condition = self.handle_expression()?;
        let then_branch = match self.handle_statement()? {
            Statement::Block(x) => x,
            _ => return Self::error(&self.previous(), "Expected a block statement after if."),
        };
        let mut else_branch = None;
        if self.match_next_token(&[TokenType::Else]) {
            else_branch = match self.handle_statement()? {
                Statement::Block(x) => Some(x),
                _ => {
                    return Self::error(&self.previous(), "Expected a block statement after else.")
                }
            };
        }
        Ok(Statement::If(IfStatement {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn handle_while_statement(&mut self) -> Result<Statement> {
        let condition = self.handle_expression()?;
        let body = match self.handle_statement()? {
            Statement::Block(x) => x,
            _ => return Self::error(&self.previous(), "Expected block statement after 'while'."),
        };
        Ok(Statement::While(WhileStatement { condition, body }))
    }

    fn handle_return_statement(&mut self) -> Result<Statement> {
        let keyword = self.previous();
        let expr = if !self.check(TokenType::BraceClose) {
            Some(self.handle_expression()?)
        } else {
            None
        };
        self.consume_if(
            TokenType::StatementEnd,
            "Expected newline after return value.",
        )?;
        Ok(Statement::Return(ReturnStatement { expr, keyword }))
    }

    fn handle_statement(&mut self) -> Result<Statement> {
        if self.match_next_token(&[TokenType::BraceOpen]) {
            self.handle_block_statement()
        } else if self.match_next_token(&[TokenType::If]) {
            self.handle_if_statement()
        } else if self.match_next_token(&[TokenType::While]) {
            self.handle_while_statement()
        } else if self.match_next_token(&[TokenType::Ret]) {
            self.handle_return_statement()
        } else {
            self.handle_expression_statement()
        }
    }

    fn handle_var_declaration(&mut self) -> Result<Statement> {
        let name = self.consume_if(TokenType::Identifier, "Expected variable name.")?;
        let mut initializer = None;
        if self.match_next_token(&[TokenType::Equal]) {
            initializer = Some(self.handle_expression()?);
        }
        self.consume_if(
            TokenType::StatementEnd,
            "Expected statement end after variable declaration.",
        )?;
        Ok(Statement::Var(VarStatement { name, initializer }))
    }

    fn handle_function_declaration(&mut self, kind: FunctionKind) -> Result<Statement> {
        let name = self.consume_if(TokenType::Identifier, &format!("Expected {} name", kind))?;
        self.consume_if(
            TokenType::ParenOpen,
            &format!("Expected '(' after {} name.", kind),
        )?;
        let mut params = vec![];
        if !self.check(TokenType::ParenClose) {
            loop {
                if params.len() >= MAX_FUNC_ARG_COUNT {
                    return Self::error(
                        self.peek(),
                        &format!("Can't have more than {} parameters.", MAX_FUNC_ARG_COUNT),
                    );
                }
                params.push(self.consume_if(TokenType::Identifier, "Expected paramter name.")?);
                if !self.match_next_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume_if(TokenType::ParenClose, "Expected ')' after parameters.")?;
        self.consume_if(
            TokenType::BraceOpen,
            &format!("Expected '{{' before {} body.", kind),
        )?;
        let body = self.parse_block()?;
        Ok(Statement::Function(FunctionStatement {
            body,
            name,
            params,
        }))
    }

    fn handle_class_declaration(&mut self) -> Result<Statement> {
        let name = self.consume_if(TokenType::Identifier, &format!("Expected class name."))?;
        self.consume_if(
            TokenType::BraceOpen,
            &format!("Expected '{{' after class name."),
        )?;

        let mut methods = vec![];
        while !self.check(TokenType::BraceClose) && !self.at_end() {
            if self.match_next_token(&[TokenType::Fn]) {
                let func = match self.handle_function_declaration(FunctionKind::Method)? {
                    Statement::Function(x) => x,
                    _ => return Self::error(&name, "Expected function declaration in class."),
                };
                methods.push(func);
            }
        }

        self.consume_if(TokenType::BraceClose, "Expected '}}' after class body.")?;
        self.consume_if(
            TokenType::StatementEnd,
            "Expected newline after class ending brace.",
        )?;
        Ok(Statement::Class(ClassStatement { methods, name }))
    }

    fn handle_declaration(&mut self) -> Result<Statement> {
        let had_err;
        if self.match_next_token(&[TokenType::Let]) {
            match self.handle_var_declaration() {
                Ok(x) => return Ok(x),
                Err(_) => had_err = true,
            }
        } else if self.match_next_token(&[TokenType::Fn]) {
            match self.handle_function_declaration(FunctionKind::Function) {
                Ok(x) => return Ok(x),
                Err(_) => had_err = true,
            }
        } else if self.match_next_token(&[TokenType::Class]) {
            match self.handle_class_declaration() {
                Ok(x) => return Ok(x),
                Err(_) => had_err = true,
            }
        } else {
            match self.handle_statement() {
                Ok(x) => return Ok(x),
                Err(_) => had_err = true,
            }
        }

        if had_err {
            self.synchronize();
            // Dont throw error, so just return a None expr statement
            return Ok(Statement::Expression(ExpressionStatement {
                expr: Expression::Literal(Box::new(LiteralExpression { value: Value::None })),
            }));
        }

        Self::error(self.peek(), "Unexpected token in declaration.")
    }

    pub fn parse(&mut self) -> Statement {
        match self.handle_declaration() {
            Ok(x) => x,
            Err(err) => panic!("Unhandled error in parser: {}", err),
        }
    }
}

impl<I: Iterator<Item = Token>> Iterator for Parser<I> {
    type Item = Statement;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_end() {
            None
        } else {
            Some(self.parse())
        }
    }
}
