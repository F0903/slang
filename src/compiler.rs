use {
    crate::{
        chunk::Chunk,
        collections::DynArray,
        dbg_println,
        debug::disassemble_chunk,
        lexing::{
            scanner::Scanner,
            token::{Precedence, Token, TokenType},
        },
        memory::HeapPtr,
        opcode::OpCode,
        value::{
            self, Value,
            object::{ObjectContainer, ObjectManager},
        },
        vm::VmHeap,
    },
    std::{cell::RefCell, rc::Rc},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type ParseFn<'a> = fn(&mut Compiler<'a>);

#[derive(Debug)]
struct ParseRule<'a> {
    prefix: Option<ParseFn<'a>>,
    infix: Option<ParseFn<'a>>,
    precedence: Precedence,
}

macro_rules! define_parse_rule_table {
    ($($token_val:expr => $rule_init:tt),*) => {{
        let mut v = crate::collections::DynArray::default();
        $(
            v.insert($token_val as usize, ParseRule $rule_init);
        )*
        v
    }};
}
pub struct Compiler<'a> {
    scanner: Scanner,
    heap: HeapPtr<VmHeap>,
    current_chunk: Rc<RefCell<Chunk>>,
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool,
    panic_mode: bool,
    parse_rule_table: DynArray<ParseRule<'a>>,
}

impl<'a> Compiler<'a> {
    pub fn new(scanner: Scanner, heap: HeapPtr<VmHeap>, chunk: Rc<RefCell<Chunk>>) -> Self {
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
                TokenType::Identifier   => {prefix: Some(Self::variable), infix: None, precedence: Precedence::None},
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

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    pub fn is_panic_mode(&self) -> bool {
        self.panic_mode
    }

    pub fn set_panic_mode(&mut self, value: bool) {
        self.panic_mode = value;
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
            Value::object(ObjectContainer::alloc_string(name, &mut self.heap).read()),
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

    fn advance(&mut self) {
        dbg_println!(
            "Advancing... last: {:?}, current: {:?}",
            &self.previous,
            &self.current
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

    fn get_previous_token(&self) -> Option<&Token> {
        self.previous.as_ref()
    }

    fn get_current_token(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn matches_previous_token(&self, token_type: TokenType) -> bool {
        self.get_previous_token()
            .map_or(false, |token| token.token_type == token_type)
    }

    fn matches_current_token(&self, token_type: TokenType) -> bool {
        self.get_current_token()
            .map_or(false, |token| token.token_type == token_type)
    }

    fn match_and_advance(&mut self, token_type: TokenType) -> bool {
        let value = self.matches_current_token(token_type);
        if value {
            self.advance();
        }
        value
    }

    fn consume(&mut self, token_type: TokenType, err_msg: &str) {
        if !self.match_and_advance(token_type) {
            self.error_at_current(err_msg);
        }
    }

    fn synchronize(&mut self) {
        self.set_panic_mode(false);

        while !self.matches_current_token(TokenType::EOF) {
            if self.matches_previous_token(TokenType::Semicolon) {
                return;
            }
            match self.get_current_token() {
                Some(t) => match t.token_type {
                    TokenType::Class
                    | TokenType::Fn
                    | TokenType::Let
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Return => return,
                    _ => {}
                },
                None => {}
            }

            self.advance();
        }
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expected ';' after expression.");
        self.current_chunk
            .borrow_mut()
            .write_opcode(OpCode::Pop, self.get_current_line());
    }

    fn statement(&mut self) {
        self.expression_statement();
    }

    /// Returns the index of the constant.
    fn parse_identifier_constant(&mut self, name_token: &Token) -> u32 {
        self.current_chunk.borrow_mut().write_constant(
            Value::object(ObjectContainer::alloc_string(&name_token.name, &mut self.heap).read()),
            name_token.line,
        )
    }

    fn parse_named_variable(&mut self, name_token: &Token) -> u32 {
        let index = self.parse_identifier_constant(name_token);
        self.current_chunk.borrow_mut().write_opcode_with_long_arg(
            OpCode::GetGlobal,
            index,
            name_token.line,
        );
        index
    }

    fn variable(&mut self) {
        let name_token = self.get_previous_token().unwrap();
        self.parse_named_variable(&name_token.clone());
    }

    fn parse_variable(&mut self, error_message: &str) -> u32 {
        self.consume(TokenType::Identifier, error_message);
        let name_token = self.get_previous_token().unwrap();
        self.parse_identifier_constant(&name_token.clone())
    }

    fn define_variable(&mut self, global_index: u32) {
        self.current_chunk.borrow_mut().write_opcode_with_long_arg(
            OpCode::DefineGlobal,
            global_index,
            self.get_current_line(),
        );
    }

    fn variable_declaration(&mut self) {
        let global = self.parse_variable("Expected variable name.");

        if self.match_and_advance(TokenType::Equal) {
            self.expression();
        } else {
            self.current_chunk
                .borrow_mut()
                .write_opcode(OpCode::None, self.get_current_line());
        }

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    fn declaration(&mut self) {
        if self.match_and_advance(TokenType::Let) {
            self.variable_declaration();
        } else {
            self.statement();
        }

        if self.is_panic_mode() {
            self.synchronize();
        }
    }

    pub fn compile(&mut self, source: &[u8]) -> Result<Rc<RefCell<Chunk>>> {
        self.set_scanner_source(source);
        self.set_current_chunk(self.current_chunk.clone());

        self.advance();
        while !self.match_and_advance(TokenType::EOF) {
            self.declaration();
        }
        self.consume(TokenType::EOF, "Expected end of file.");

        self.current_chunk
            .borrow_mut()
            .write_opcode(OpCode::Return, self.get_current_line());

        #[cfg(debug_assertions)]
        if !self.had_error() {
            disassemble_chunk(&mut self.current_chunk.borrow_mut(), "code");
        }

        self.current_chunk.borrow_mut().encode();
        if self.had_error() {
            Err("Parser encountered errors!".into())
        } else {
            Ok(self.current_chunk.clone())
        }
    }
}
