use {
    crate::{
        chunk::Chunk,
        collections::DynArray,
        lexing::{
            scanner::Scanner,
            token::{Precedence, Token, TokenType},
        },
        opcode::OpCode,
        value::{
            Value,
            object::{ObjectContainer, ObjectManager},
        },
        vm::VmHeap,
    },
    std::{cell::RefCell, rc::Rc},
};

type ParseFn<'a> = fn(&mut Parser<'a>);

#[derive(Debug)]
struct ParseRule<'a> {
    prefix: Option<ParseFn<'a>>,
    infix: Option<ParseFn<'a>>,
    precedence: Precedence,
}

macro_rules! define_parse_rule_table {
    ($($token_val:expr => $rule_init:tt),*) => {{
        let mut v = crate::collections::DynArray::new();
        $(
            v.insert($token_val as usize, ParseRule $rule_init);
        )*
        v
    }};
}
pub struct Parser<'a> {
    scanner: Scanner,
    heap: Rc<RefCell<VmHeap>>,
    current_chunk: Rc<RefCell<Chunk>>,
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool,
    panic_mode: bool,
    parse_rule_table: DynArray<ParseRule<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: Scanner, heap: Rc<RefCell<VmHeap>>, chunk: Rc<RefCell<Chunk>>) -> Self {
        Self {
            scanner,
            heap,
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
                TokenType::Not          => {prefix: Some(Self::unary), infix: None, precedence: Precedence::None},
                TokenType::Equal        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Is           => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Equality},
                TokenType::IsNot        => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Equality},
                TokenType::Greater      => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Comparison},
                TokenType::GreaterEqual => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Comparison},
                TokenType::Less         => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Comparison},
                TokenType::LessEqual    => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Comparison},
                TokenType::Identifier   => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::String       => {prefix: Some(Self::string), infix: None, precedence: Precedence::None},
                TokenType::Number       => {prefix: Some(Self::number), infix: None, precedence: Precedence::None},
                TokenType::And          => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Class        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Else         => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::False        => {prefix: Some(Self::literal), infix: None, precedence: Precedence::None},
                TokenType::For          => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Fn           => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::If           => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::None         => {prefix: Some(Self::literal), infix: None, precedence: Precedence::None},
                TokenType::Or           => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Return       => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Super        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::This         => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::True         => {prefix: Some(Self::literal), infix: None, precedence: Precedence::None},
                TokenType::Let          => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::While        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::EOF          => {prefix: None, infix: None, precedence: Precedence::None}
            },
        }
    }

    pub fn set_current_chunk(&mut self, chunk: Rc<RefCell<Chunk>>) {
        self.current_chunk = chunk;
    }

    fn get_rule(&self, token: TokenType) -> &ParseRule<'a> {
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
        print!("[line {}] Error!", token.line);

        match token.token_type {
            TokenType::EOF => print!(" at end."),
            _ => print!(" at {}", token.name),
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
        let previous_token_type = unsafe { self.previous.as_ref().unwrap_unchecked().token_type };
        let prefix_rule = self.get_rule(previous_token_type).prefix;
        match prefix_rule {
            Some(x) => x(self),
            None => {
                self.error("Expected an expression.");
                return;
            }
        }

        while precedence
            <= self
                .get_rule(unsafe { self.current.as_ref().unwrap_unchecked().token_type })
                .precedence
        {
            self.advance();
            let previous_token_type =
                unsafe { self.previous.as_ref().unwrap_unchecked().token_type };
            let infix_rule = self.get_rule(previous_token_type).infix;
            infix_rule.unwrap()(self);
        }
    }

    fn string(&mut self) {
        let token = self.previous.as_ref().unwrap();
        let name = &token.name;
        let name = &name[1..name.len() - 1];
        self.current_chunk.borrow_mut().write_constant(
            Value::object(
                ObjectContainer::alloc_string(name, &mut self.heap.borrow_mut().objects).take(),
            ), // Can "take" pointer value because the pointer will be appended to VM list, so no leak.
            token.line,
        );
    }

    fn literal(&mut self) {
        match self.previous.as_ref().unwrap().token_type {
            TokenType::False => self.emit_op(OpCode::False),
            TokenType::True => self.emit_op(OpCode::True),
            TokenType::None => self.emit_op(OpCode::None),
            _ => unreachable!(),
        }
    }

    fn unary(&mut self) {
        let operator_type = unsafe { self.previous.as_ref().unwrap_unchecked().token_type };

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Not => self.emit_op(OpCode::Not),
            TokenType::Minus => self.emit_op(OpCode::Negate),
            _ => return,
        }
    }

    fn binary(&mut self) {
        let next_token_type = unsafe { self.current.as_ref().unwrap_unchecked().token_type };
        let operator_type = unsafe { self.previous.as_ref().unwrap_unchecked().token_type };

        let parse_rule = self.get_rule(operator_type);
        self.parse_precedence(parse_rule.precedence.add(1));

        match operator_type {
            TokenType::IsNot => self.emit_op(OpCode::IsNot),
            TokenType::Is => {
                if next_token_type == TokenType::Not {
                    self.current_chunk
                        .borrow_mut()
                        .replace_last_op(OpCode::IsNot);
                    return;
                }
                self.emit_op(OpCode::Is)
            }
            TokenType::Greater => self.emit_op(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_op(OpCode::GreaterEqual),
            TokenType::Less => self.emit_op(OpCode::Less),
            TokenType::LessEqual => self.emit_op(OpCode::LessEqual),
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
        let num: f64 = unsafe {
            self.previous
                .as_ref()
                .unwrap_unchecked()
                .name
                .parse()
                .unwrap_unchecked()
        };
        let line = self.get_current_line();
        self.current_chunk
            .borrow_mut()
            .write_constant(Value::number(num), line);
    }

    pub(crate) fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    pub(crate) fn consume(&mut self, token_type: TokenType, err_msg: &str) {
        let current_token_type = unsafe { self.current.as_ref().unwrap_unchecked().token_type };
        if current_token_type == token_type {
            self.advance();
            return;
        }
        self.error_at_current(err_msg);
    }
}
