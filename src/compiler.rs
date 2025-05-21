use {
    crate::{
        chunk::Chunk,
        collections::{DynArray, Stack},
        dbg_println,
        debug::disassemble_chunk,
        lexing::{
            scanner::Scanner,
            token::{Precedence, Token, TokenType},
        },
        local::Local,
        memory::HeapPtr,
        opcode::OpCode,
        value::{Value, object::ObjectNode},
        vm::VmHeap,
    },
    std::{cell::RefCell, rc::Rc},
};

const LOCAL_SLOTS: usize = 1024;

type CompilerResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type ParseFn<'a, 'src> = fn(&mut Compiler<'a, 'src>, bool);

#[derive(Debug)]
struct ParseRule<'a, 'src> {
    prefix: Option<ParseFn<'a, 'src>>,
    infix: Option<ParseFn<'a, 'src>>,
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
pub struct Compiler<'a, 'src> {
    current_source: &'src [u8],
    scanner: Scanner<'src>,
    heap: HeapPtr<VmHeap>,
    current_chunk: Rc<RefCell<Chunk>>,
    current: Option<Token>,
    previous: Option<Token>,
    locals: Stack<Local, LOCAL_SLOTS>,
    scope_depth: i32,
    had_error: bool,
    panic_mode: bool,
    parse_rule_table: DynArray<ParseRule<'a, 'src>>,
}

impl<'a, 'src> Compiler<'a, 'src>
where
    'src: 'a,
{
    pub fn new(scanner: Scanner<'src>, heap: HeapPtr<VmHeap>, chunk: Rc<RefCell<Chunk>>) -> Self {
        Self {
            current_source: &[],
            scanner,
            heap,
            current_chunk: chunk,
            current: None,
            previous: None,
            locals: Stack::new(),
            scope_depth: 0,
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
                TokenType::And          => {prefix: None, infix: Some(Self::and), precedence: Precedence::None},
                TokenType::Class        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Else         => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::False        => {prefix: Some(Self::literal), infix: None, precedence: Precedence::None},
                TokenType::For          => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Fn           => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::If           => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::None         => {prefix: Some(Self::literal), infix: None, precedence: Precedence::None},
                TokenType::Or           => {prefix: None, infix: Some(Self::or), precedence: Precedence::None},
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

    fn get_rule(&self, token: TokenType) -> &ParseRule<'a, 'src> {
        self.parse_rule_table.read(token as usize)
    }

    pub const fn get_current_source(&self) -> &'src [u8] {
        self.current_source
    }

    pub const fn get_current_line(&self) -> u32 {
        self.scanner.get_current_line()
    }

    fn get_instruction_count(&self) -> usize {
        self.current_chunk.borrow().get_bytes_count()
    }

    fn error_at_line(&mut self, line: u32, msg: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        print!("[line {}] Error!", line);

        print!("{}", msg);
        print!("\n");
        self.had_error = true;
    }

    fn error_at_token(&mut self, token: &Token, msg: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        print!("[line {}] Error!", token.line);

        match token.token_type {
            TokenType::EOF => print!(" at end."),
            _ => print!(
                " at '{}'\n\t",
                token.lexeme.get_str(self.get_current_source())
            ),
        }
        print!("{}", msg);
        print!("\n");
        self.had_error = true;
    }

    fn error(&mut self, msg: &str) {
        if let Some(prev) = &self.previous {
            self.error_at_token(&prev.clone(), msg);
        } else {
            self.error_at_line(self.get_current_line(), msg);
        }
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

    /// Convenience function to write an opcode to the current chunk.
    fn emit_op(&mut self, op: OpCode) {
        self.current_chunk
            .borrow_mut()
            .write_opcode(op, self.get_current_line());
    }

    /// Convenience function to write an opcode with a u8 arg to the current chunk.
    fn emit_op_with_byte(&mut self, op: OpCode, arg: u8) {
        self.current_chunk.borrow_mut().write_opcode_with_byte_arg(
            op,
            arg,
            self.get_current_line(),
        );
    }

    /// Convenience function to write an opcode with a u16 arg to the current chunk.
    fn emit_op_with_double(&mut self, op: OpCode, arg: u16) {
        self.current_chunk
            .borrow_mut()
            .write_opcode_with_double_arg(op, arg, self.get_current_line());
    }

    /// Convenience function to write an opcode with a u32 arg to the current chunk.
    fn emit_op_with_quad(&mut self, op: OpCode, arg: u32) {
        self.current_chunk
            .borrow_mut()
            .write_opcode_with_quad(op, arg, self.get_current_line());
    }

    /// Convenience function to replace the last opcode in the current chunk.
    fn replace_last_op(&mut self, op: OpCode) {
        self.current_chunk.borrow_mut().replace_last_op(op);
    }

    /// Convenience function to write a jump opcode.
    fn emit_jump(&mut self, op: OpCode, to: u16) {
        debug_assert!(
            op == OpCode::Jump || op == OpCode::JumpIfFalse,
            "non-jump instruction passed to emit_jump"
        );
        self.emit_op_with_double(op, to);
    }

    /// Convenience function to write a jump opcode for backpatching.
    fn emit_jump_backpatch(&mut self, op: OpCode) -> u32 {
        self.emit_jump(op, u16::MAX);
        (self.get_instruction_count() - 2) as u32
    }

    /// Convenience function to write a jumpback opcode that jumps back to the specified index.
    fn emit_backjump(&mut self, loop_start: u32) {
        let offset = self.get_instruction_count() as u32 - loop_start + 2;
        if offset > u16::MAX as u32 {
            self.error("Loop body is too large to jump.");
        }
        let offset = offset as u16;

        self.emit_op_with_double(OpCode::Backjump, offset);
    }

    /// Returns constant index
    fn emit_constant_with_op(&mut self, value: Value) -> u32 {
        self.current_chunk
            .borrow_mut()
            .add_constant_with_op(value, self.get_current_line())
    }

    /// Returns constant index
    fn emit_constant(&mut self, value: Value) -> u32 {
        self.current_chunk.borrow_mut().add_constant(value)
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let previous_token_type = unsafe { self.previous.as_ref().unwrap_unchecked().token_type };
        let prefix_rule = self.get_rule(previous_token_type).prefix;
        let can_assign = precedence <= Precedence::Assignment;
        match prefix_rule {
            Some(prefix_rule) => prefix_rule(self, can_assign),
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
            infix_rule.unwrap()(self, can_assign);
        }

        if can_assign && self.match_and_advance(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn string(&mut self, _can_assign: bool) {
        let token = self.previous.as_ref().unwrap();

        let source = self.get_current_source();
        let name = token.lexeme.get_str(source);
        let name = &name[1..name.len() - 1]; // Don't include the leading and trailing "
        let value = Value::object(ObjectNode::alloc_string(name, &mut self.heap).read());
        self.emit_constant_with_op(value);
    }

    fn literal(&mut self, _can_assign: bool) {
        let op = match self.previous.as_ref().unwrap().token_type {
            TokenType::False => OpCode::False,
            TokenType::True => OpCode::True,
            TokenType::None => OpCode::None,
            _ => unreachable!(),
        };
        self.emit_op(op);
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_type = unsafe { self.previous.as_ref().unwrap_unchecked().token_type };

        self.parse_precedence(Precedence::Unary);

        let op = match operator_type {
            TokenType::Not => OpCode::Not,
            TokenType::Minus => OpCode::Negate,
            _ => return,
        };

        self.emit_op(op);
    }

    fn binary(&mut self, _can_assign: bool) {
        let next_token_type = unsafe { self.current.as_ref().unwrap_unchecked().token_type };
        let operator_type = unsafe { self.previous.as_ref().unwrap_unchecked().token_type };

        let parse_rule = self.get_rule(operator_type);
        self.parse_precedence(parse_rule.precedence.add(1));

        let op = match operator_type {
            TokenType::IsNot => OpCode::IsNot,
            TokenType::Is => {
                if next_token_type == TokenType::Not {
                    self.replace_last_op(OpCode::IsNot);
                    return;
                }
                OpCode::Is
            }
            TokenType::Greater => OpCode::Greater,
            TokenType::GreaterEqual => OpCode::GreaterEqual,
            TokenType::Less => OpCode::Less,
            TokenType::LessEqual => OpCode::LessEqual,
            TokenType::Plus => OpCode::Add,
            TokenType::Minus => OpCode::Subtract,
            TokenType::Star => OpCode::Multiply,
            TokenType::Slash => OpCode::Divide,
            _ => unreachable!(),
        };
        self.emit_op(op);
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(
            TokenType::RightParen,
            "Expected ')' after grouping expression.",
        );
    }

    fn number(&mut self, _can_assign: bool) {
        let num: f64 = unsafe {
            self.previous
                .as_ref()
                .unwrap_unchecked()
                .lexeme
                .get_str(self.get_current_source())
                .parse()
                .unwrap_unchecked()
        };
        self.emit_constant_with_op(Value::number(num));
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
                    self.error(err.get_message());
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

    fn is_previous_token(&self, token_type: TokenType) -> bool {
        self.get_previous_token()
            .map_or(false, |token| token.token_type == token_type)
    }

    fn is_current_token(&self, token_type: TokenType) -> bool {
        self.get_current_token()
            .map_or(false, |token| token.token_type == token_type)
    }

    fn match_and_advance(&mut self, token_type: TokenType) -> bool {
        let value = self.is_current_token(token_type);
        if value {
            self.advance();
        }
        value
    }

    fn consume(&mut self, token_type: TokenType, err_msg: &str) {
        if !self.match_and_advance(token_type) {
            self.error(err_msg);
        }
    }

    fn synchronize(&mut self) {
        self.set_panic_mode(false);

        while !self.is_current_token(TokenType::EOF) {
            if self.is_previous_token(TokenType::Semicolon) {
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
        // Parse expression which pushes a value onto the stack, and then pop it off again since this is a statement
        self.expression();
        self.emit_op(OpCode::Pop);
    }

    fn block(&mut self) {
        while !self.is_current_token(TokenType::RightBrace)
            && !self.is_current_token(TokenType::EOF)
        {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block.");
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        // Pop all locals in scope
        let mut locals_to_pop = 0;
        while self.locals.count() > 0 && self.locals.peek(0).depth > self.scope_depth {
            locals_to_pop += 1;
            self.locals.pop();
        }
        if locals_to_pop > 0 {
            self.emit_op_with_double(OpCode::PopN, locals_to_pop);
        }
    }

    fn patch_jump(&mut self, offset: u32) {
        // -2 to adjust for the bytecode for the jump itself
        let (code, jump) = {
            let chunk = self.current_chunk.borrow();
            (
                chunk.get_code_ptr(),
                chunk.get_bytes_count() - offset as usize - 2,
            )
        };
        if jump > u16::MAX as usize {
            self.error("Jump distance is too far.");
        }
        let jump = jump as u16;

        unsafe { code.add(offset as usize).cast::<u16>().write(jump) };
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump_backpatch(OpCode::JumpIfFalse);

        self.emit_op(OpCode::Pop);
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    fn or(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump_backpatch(OpCode::JumpIfTrue);

        self.emit_op(OpCode::Pop);
        self.parse_precedence(Precedence::Or);

        self.patch_jump(end_jump);
    }

    fn if_statement(&mut self) {
        self.expression();
        if !self.is_current_token(TokenType::LeftBrace) {
            self.error("Missing '{' after if statement.");
        }

        let then_jump = self.emit_jump_backpatch(OpCode::JumpIfFalse);
        self.emit_op(OpCode::Pop);
        self.statement();

        let else_jump = self.emit_jump_backpatch(OpCode::Jump);

        self.patch_jump(then_jump);
        self.emit_op(OpCode::Pop);

        if self.match_and_advance(TokenType::Else) {
            if !self.is_current_token(TokenType::LeftBrace) {
                self.error("Missing '{' after else statement.");
            }
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn while_statement(&mut self) {
        let loop_start = self.get_instruction_count() as u32;

        self.expression();
        if !self.is_current_token(TokenType::LeftBrace) {
            self.error("Missing '{' after while statement.");
        }

        let exit_jump = self.emit_jump_backpatch(OpCode::JumpIfFalse);
        self.emit_op(OpCode::Pop);
        self.statement();
        self.emit_backjump(loop_start);

        self.patch_jump(exit_jump);
        self.emit_op(OpCode::Pop);
    }

    fn statement(&mut self) {
        dbg_println!("\nPARSING STATEMENT");

        if self.match_and_advance(TokenType::If) {
            self.if_statement();
        } else if self.match_and_advance(TokenType::While) {
            self.while_statement();
        } else if self.match_and_advance(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    /// Returns the index of the constant.
    fn identifier_constant(&mut self, name_token: &Token) -> u32 {
        let lexeme = name_token.lexeme.get_str(self.get_current_source());
        let value = Value::object(ObjectNode::alloc_string(lexeme, &mut self.heap).read());
        self.emit_constant(value)
    }

    /// Returns None if no local variable with the name is found.
    fn resolve_local(&mut self, name_token: &Token) -> Option<u16> {
        for (i, local) in self.locals.iter().enumerate() {
            if name_token.lexeme.get_str(self.get_current_source())
                == local.name.lexeme.get_str(self.get_current_source())
            {
                if !local.is_initialized() {
                    self.error("Can't read uninitialized variable.");
                }
                return Some(i as u16);
            }
        }

        None
    }

    fn named_variable(&mut self, name_token: &Token, can_assign: bool) -> u32 {
        enum VariableType {
            Local,
            Global,
        }

        let (slot, get_op, set_op, var_type) = if let Some(slot) = self.resolve_local(&name_token) {
            (
                slot as u32,
                OpCode::GetLocal,
                OpCode::SetLocal,
                VariableType::Local,
            )
        } else {
            let slot = self.identifier_constant(name_token);
            (
                slot,
                OpCode::GetGlobal,
                OpCode::SetGlobal,
                VariableType::Global,
            )
        };

        // If the current token is an '=', then we are assigning instead of getting.
        let op = if can_assign && self.match_and_advance(TokenType::Equal) {
            self.expression();
            set_op
        } else {
            get_op
        };

        match var_type {
            VariableType::Local => {
                self.emit_op_with_double(op, slot as u16);
            }
            VariableType::Global => {
                self.emit_op_with_quad(op, slot);
            }
        }

        slot
    }

    // Gets called from parse table on a 'let' token
    fn variable(&mut self, can_assign: bool) {
        let name_token = self
            .get_previous_token()
            .cloned()
            .expect("Unexpected state: no name token for variable.");
        self.named_variable(&name_token, can_assign);
    }

    fn add_local(&mut self, name: Token) {
        if self.locals.count() >= self.locals.stack_size() {
            self.error("Cannot add local, too many locals in scope!");
            return;
        }
        self.locals.push(Local::new(name, self.scope_depth));
    }

    fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }

        let name = self
            .get_previous_token()
            .cloned()
            .expect("Unexpected state: no name token for local variable.");
        self.add_local(name);
    }

    fn parse_variable(&mut self, error_message: &str) -> u32 {
        self.consume(TokenType::Identifier, error_message);

        self.declare_variable();
        if self.scope_depth > 0 {
            return 0;
        }

        let name_token = self.get_previous_token().cloned().unwrap();
        self.identifier_constant(&name_token)
    }

    fn define_variable(&mut self, global_index: u32) {
        if self.scope_depth > 0 {
            self.locals.peek_mut(0).depth = self.scope_depth;
            return;
        }

        self.emit_op_with_quad(OpCode::DefineGlobal, global_index);
    }

    fn variable_declaration(&mut self) {
        dbg_println!("\nPARSING VARIABLE DECL");

        let global_index = self.parse_variable("Expected variable name.");

        if self.match_and_advance(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_op(OpCode::None);
        }

        self.define_variable(global_index);
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

    pub fn compile(&mut self, source: &'src [u8]) -> CompilerResult<Rc<RefCell<Chunk>>> {
        self.current_source = source;
        self.scanner.set_source(source);
        self.set_current_chunk(self.current_chunk.clone());

        self.advance();
        while !self.match_and_advance(TokenType::EOF) {
            self.declaration();
        }
        self.consume(TokenType::EOF, "Expected end of file.");

        //Temporary None and return ops
        self.emit_constant_with_op(Value::none());
        self.emit_op(OpCode::Return);

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
