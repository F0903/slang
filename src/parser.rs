use std::{cell::RefCell, rc::Rc};

use crate::{
    chunk::Chunk,
    dynarray::DynArray,
    opcode::OpCode,
    scanner::Scanner,
    token::{Precedence, Token, TokenType},
};

// Make concrete error type for parser.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type ParseFn<'a> = fn(&mut Parser<'a>);

struct ParseRule<'a> {
    prefix: Option<ParseFn<'a>>,
    infix: Option<ParseFn<'a>>,
    precedence: Precedence,
}

macro_rules! define_parse_rule_table {
    ($($token_val:expr => $rule_init:tt),*) => {{
        let mut v = crate::dynarray::DynArray::new();
        $(
            v.insert($token_val as usize, ParseRule $rule_init);
        )*
        v
    }};
}
pub struct Parser<'a> {
    scanner: Scanner,
    current_chunk: Rc<RefCell<Chunk>>,
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool,
    panic_mode: bool,
    parse_rule_table: DynArray<ParseRule<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: Scanner, chunk: Rc<RefCell<Chunk>>) -> Self {
        Self {
            scanner,
            current_chunk: chunk,
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
            parse_rule_table: define_parse_rule_table! {
                TokenType::LeftParen    => {prefix: Some(Self::grouping), infix: None, precedence: Precedence::None},
                TokenType::RightParen   => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::LeftBrace    => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::RightBrace   => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Comma        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Dot          => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Minus        => {prefix: Some(Self::unary), infix: Some(Self::binary), precedence: Precedence::Term},
                TokenType::Plus         => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Term},
                TokenType::Semicolon    => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Slash        => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Factor},
                TokenType::Star         => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Factor},
                TokenType::Bang         => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Not          => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Equal        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Is           => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Greater      => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::GreaterEqual => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Less         => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::LessEqual    => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Identifier   => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::String       => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Number       => {prefix: Some(Self::number), infix: None, precedence: Precedence::None},
                TokenType::And          => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Class        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Else         => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::False        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::For          => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Fn           => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::If           => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::None         => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Or           => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Return       => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Super        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::This         => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::True         => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Let          => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::While        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::EOF          => {prefix: None, infix: None, precedence: Precedence::None}
            },
        }
    }

    pub fn set_current_chunk(&mut self, chunk: Rc<RefCell<Chunk>>) {
        self.current_chunk = chunk;
    }

    const fn get_rule(&self, token: TokenType) -> ParseRule<'a> {
        self.parse_rule_table.read(token as usize)
    }

    pub const fn get_current_line(&self) -> u32 {
        self.scanner.get_current_line()
    }

    pub fn set_scanner_source(&mut self, source: &[u8]) {
        self.scanner.set_source(source);
    }

    fn emit_op(&mut self, op: OpCode) {
        let line = self.get_current_line();
        self.current_chunk.borrow_mut().write_opcode(op, line);
    }

    fn error_at(&mut self, token: &Token, msg: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        print!("[line {}] Error!", token.get_line());

        match token.get_type() {
            TokenType::EOF => print!(" at end."),
            _ => print!(" at {}", token.get_name()),
        }
        print!("{}", msg);
        print!("\n");
        self.had_error = true;
    }

    fn error(&mut self, msg: &str) {
        self.error_at(
            &self
                .previous
                .clone()
                .expect("Encountered error but the previous token was null!"),
            msg,
        );
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(
            &self
                .previous
                .clone()
                .expect("Encountered error but the current token was null!"),
            message,
        )
    }

    pub(crate) fn had_error(&self) -> bool {
        self.had_error
    }

    pub(crate) fn advance(&mut self) {
        println!(
            "Advancing... last: {:?}, current: {:?}",
            &self.previous, &self.current
        );
        self.previous = self.current.clone();
        self.current = loop {
            match self.scanner.scan() {
                Ok(x) => break Some(x),
                Err(err) => {
                    self.error_at_current(err.get_message());
                }
            }
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self
            .get_rule(
                self.previous
                    .as_ref()
                    .expect("Previous token as null!")
                    .get_type(),
            )
            .prefix;
        match prefix_rule {
            Some(x) => x(self),
            None => {
                self.error("Expected an expression.");
                return;
            }
        }

        while precedence
            <= self
                .get_rule(self.current.as_ref().unwrap().get_type())
                .precedence
        {
            self.advance();
            let infix_rule = self
                .get_rule(self.previous.as_ref().unwrap().get_type())
                .infix;
            infix_rule.unwrap()(self);
        }
    }

    fn unary(&mut self) {
        let operator_type = self
            .previous
            .as_ref()
            .expect("Previous token was null!")
            .get_type();

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => {
                let line = self.get_current_line();
                self.current_chunk
                    .borrow_mut()
                    .write_opcode(OpCode::Negate, line);
            }
            _ => return,
        }
    }

    fn binary(&mut self) {
        let operator_type = self
            .previous
            .as_ref()
            .expect("Previous token was null!")
            .get_type();
        let parse_rule = self.get_rule(operator_type);
        self.parse_precedence(parse_rule.precedence.add(1));
        match operator_type {
            TokenType::Plus => self.emit_op(OpCode::Add),
            TokenType::Minus => self.emit_op(OpCode::Subtract),
            TokenType::Star => self.emit_op(OpCode::Multiply),
            TokenType::Slash => self.emit_op(OpCode::Divide),
            _ => unreachable!(),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(
            TokenType::RightParen,
            "Expected ')' after grouping expression.",
        );
    }

    fn number(&mut self) {
        let value: f64 = self
            .previous
            .as_ref()
            .expect("Previous token was null!")
            .get_name()
            .parse()
            .expect("Could not parse number!");
        let line = self.get_current_line();
        self.current_chunk.borrow_mut().write_constant(value, line);
    }

    pub(crate) fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    pub(crate) fn consume(&mut self, token_type: TokenType, err_msg: &str) {
        if self
            .current
            .as_ref()
            .expect("Current token was not set!")
            .get_type()
            == token_type
        {
            self.advance();
            return;
        }
        self.error_at_current(err_msg);
    }
}
