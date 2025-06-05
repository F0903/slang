use std::mem::MaybeUninit;

use super::{FunctionType, chunk::Chunk, local::Local};
use crate::{
    collections::{DynArray, Stack},
    compiler::upvalue::Upvalue,
    dbg_println,
    debug::disassemble_chunk,
    lexing::{
        scanner::Scanner,
        token::{Precedence, Token, TokenType},
    },
    memory::{Dealloc, HeapPtr},
    value::{
        Value,
        object::{Function, Object, ObjectNode},
    },
    vm::{VmHeap, opcode::OpCode},
};

const MAX_FUNCTION_ARITY: u8 = 255; // Maximum number of arguments a function can have.

const LOCAL_SLOTS: usize = 1024;
const UPVALUES_MAX: usize = 255;

type CompilerResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type ParseFn<'src> = fn(&mut Compiler<'src>, bool);

#[derive(Debug, Clone)]
struct ParseRule<'src> {
    prefix: Option<ParseFn<'src>>,
    infix: Option<ParseFn<'src>>,
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

#[derive(Debug, Clone)]
struct EnclosingLoop {
    start_jump_index: u32,
    exit_jump_index: u32,
}

#[derive(Debug, Clone)]
struct JumpIndecies {
    instruction_index: u32,
}

impl JumpIndecies {
    const OPCODE_SIZE: u32 = 1;
    const ARGUMENT_SIZE: u32 = 2;
    const JUMP_SIZE: u32 = Self::OPCODE_SIZE + Self::ARGUMENT_SIZE;

    pub const fn get_argument_index(&self) -> u32 {
        self.instruction_index + Self::OPCODE_SIZE
    }
}

#[derive(Debug)]
pub struct Compiler<'src> {
    current_source: &'src [u8],
    scanner: HeapPtr<Scanner<'src>>,
    heap: HeapPtr<VmHeap>,
    current_function: Function,
    current_function_type: FunctionType,
    locals: Stack<Local, LOCAL_SLOTS>,
    upvalues: Stack<Upvalue, UPVALUES_MAX>,
    scope_depth: i32,
    enclosing_loop: Option<EnclosingLoop>,
    // SAFETY: It is guaranteed that the enclosing compiler outlives the holder of the reference.
    enclosing_compiler: Option<*mut Compiler<'src>>,
    had_error: bool,
    panic_mode: bool,
    parse_rule_table: DynArray<ParseRule<'src>>,
}

impl<'src> Compiler<'src> {
    pub fn new(
        scanner: HeapPtr<Scanner<'src>>,
        heap: HeapPtr<VmHeap>,
        function_type: FunctionType,
    ) -> Self {
        let mut locals = Stack::new();
        locals.push(Local::dummy()); // Reserve first slot as index 0 is used for the "main" function.

        Self {
            current_source: &[],
            scanner,
            heap,
            current_function: Function {
                arity: 0,
                chunk: HeapPtr::alloc(Chunk::new()),
                name: None,
                upvalue_count: 0,
            },
            current_function_type: function_type,
            locals,
            upvalues: Stack::new(),
            scope_depth: 0,
            enclosing_loop: None,
            enclosing_compiler: None,
            had_error: false,
            panic_mode: false,
            parse_rule_table: define_parse_rule_table! {
                // REMEMBER TO ADD EVERY NEW TOKEN HERE
                TokenType::LeftParen    => {prefix: Some(Self::grouping), infix: Some(Self::call), precedence: Precedence::Call},
                TokenType::RightParen   => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::LeftBrace    => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::RightBrace   => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Comma        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Dot          => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Minus        => {prefix: Some(Self::unary), infix: Some(Self::binary), precedence: Precedence::Term},
                TokenType::MinusEqual   => {prefix: None, infix: None, precedence: Precedence::Term},
                TokenType::Plus         => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Term},
                TokenType::PlusEqual    => {prefix: None, infix: None, precedence: Precedence::Term},
                TokenType::Semicolon    => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Slash        => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Factor},
                TokenType::Star         => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Factor},
                TokenType::Not          => {prefix: Some(Self::unary), infix: None, precedence: Precedence::None},
                TokenType::Equal        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Is           => {prefix: None, infix: Some(Self::binary), precedence: Precedence::Equality},
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
                TokenType::Continue     => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::Break        => {prefix: None, infix: None, precedence: Precedence::None},
                TokenType::EOF          => {prefix: None, infix: None, precedence: Precedence::None}
            },
        }
    }

    fn fork(&mut self, function_type: FunctionType) -> Self {
        Self {
            current_source: self.current_source,
            scanner: self.scanner.clone(),
            heap: self.heap.clone(),
            current_function: Function {
                arity: 0,
                chunk: HeapPtr::alloc(Chunk::new()),
                name: None,
                upvalue_count: 0,
            },
            current_function_type: function_type,
            locals: Stack::new(),
            upvalues: Stack::new(),
            scope_depth: 0,
            enclosing_loop: None,
            enclosing_compiler: Some(self),
            had_error: false,
            panic_mode: false,
            parse_rule_table: self.parse_rule_table.clone(),
        }
    }

    fn get_current_chunk(&self) -> &Chunk {
        &self.current_function.chunk
    }

    fn get_current_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.current_function.chunk
    }

    fn get_rule(&self, token: TokenType) -> &ParseRule<'src> {
        debug_assert!(
            self.parse_rule_table.get_count() > token as usize,
            "Token index out of bounds in parse rule table.\nMight be missing parse rule for newly added tokens.",
        );
        self.parse_rule_table.get(token as usize)
    }

    pub const fn get_current_source(&self) -> &'src [u8] {
        self.current_source
    }

    fn get_current_token(&self) -> Option<&Token> {
        self.scanner.get_current_token()
    }

    fn get_previous_token(&self) -> Option<&Token> {
        self.scanner.get_previous_token()
    }

    pub fn get_current_line(&self) -> u32 {
        self.get_previous_token()
            .map_or_else(|| self.scanner.get_current_line(), |token| token.line)
    }

    fn get_instruction_count(&self) -> usize {
        self.get_current_chunk().get_bytes_count()
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
        if let Some(prev) = self.get_previous_token() {
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
    fn emit_op(&mut self, op: OpCode, line: u32) {
        self.get_current_chunk_mut().write_opcode(op, line);
    }

    /// Convenience function to write an opcode with a u8 arg to the current chunk.
    fn emit_op_with_byte(&mut self, op: OpCode, arg: u8, line: u32) {
        self.get_current_chunk_mut()
            .write_opcode_with_byte_arg(op, arg, line);
    }

    /// Convenience function to write an opcode with a u16 arg to the current chunk.
    fn emit_op_with_double(&mut self, op: OpCode, arg: u16, line: u32) {
        self.get_current_chunk_mut()
            .write_opcode_with_double_arg(op, arg, line);
    }

    /// Convenience function to write an opcode with a u32 arg to the current chunk.
    fn emit_op_with_quad(&mut self, op: OpCode, arg: u32, line: u32) {
        self.get_current_chunk_mut()
            .write_opcode_with_quad(op, arg, line);
    }

    /// Convenience function to replace the last opcode in the current chunk.
    fn replace_last_op(&mut self, op: OpCode) {
        self.get_current_chunk_mut().replace_last_op(op);
    }

    /// Convenience function to write a jump opcode.
    fn emit_jump(&mut self, op: OpCode, arg: u32) {
        debug_assert!(
            op == OpCode::Jump
                || op == OpCode::JumpIfFalse
                || op == OpCode::JumpIfTrue
                || op == OpCode::Backjump,
            "non-jump instruction passed to emit_jump"
        );
        if arg > u16::MAX as u32 {
            self.error("Jump distance is too far.");
        }

        self.emit_op_with_double(op, arg as u16, self.get_current_line());
    }

    /// Convenience function to write a jump opcode for backpatching.
    fn emit_jump_backpatch(&mut self, op: OpCode) -> JumpIndecies {
        self.emit_jump(op, u16::MAX as u32);
        let index = self.get_instruction_count() as u32 - JumpIndecies::JUMP_SIZE;
        JumpIndecies {
            instruction_index: index,
        }
    }

    /// Function to patch a jump instruction at the specified offset to point to the current instruction.
    fn patch_jump(&mut self, offset: u32) {
        let (code, jump) = {
            (
                self.get_current_chunk().get_code_ptr(),
                self.get_instruction_count() as u32 - offset - (JumpIndecies::ARGUMENT_SIZE),
            )
        };

        if jump > u16::MAX as u32 {
            self.error("Jump distance is too far.");
        }
        let jump = jump as u16;

        unsafe { code.add(offset as usize).cast::<u16>().write(jump) };
    }

    /// Convenience function to write a jumpback opcode that jumps back to the specified index.
    fn emit_backjump(&mut self, to: u32) {
        let backward = self.get_instruction_count() as u32 + JumpIndecies::JUMP_SIZE - to;
        self.emit_jump(OpCode::Backjump, backward);
    }

    /// Returns constant index
    fn emit_constant_with_op(&mut self, value: Value, line: u32) -> u32 {
        self.get_current_chunk_mut()
            .add_constant_with_op(value, line)
    }

    /// Returns constant index
    fn emit_constant(&mut self, value: Value) -> u32 {
        self.get_current_chunk_mut().add_constant(value)
    }

    fn emit_empty_return(&mut self) {
        self.emit_op(OpCode::None, self.get_current_line());
        self.emit_op(OpCode::Return, self.get_current_line());
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let previous_token_type =
            unsafe { self.get_previous_token().unwrap_unchecked().token_type };
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
                .get_rule(unsafe { self.get_current_token().unwrap_unchecked().token_type })
                .precedence
        {
            self.advance();
            let previous_token_type =
                unsafe { self.get_previous_token().unwrap_unchecked().token_type };
            let infix_rule = self.get_rule(previous_token_type).infix;
            infix_rule.unwrap()(self, can_assign);
        }

        if can_assign && self.match_and_advance(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn string(&mut self, _can_assign: bool) {
        let (name, line) = {
            let token = self.get_previous_token().expect("Expected a string token.");
            let source = self.get_current_source();
            let name = token.lexeme.get_str(source);
            let name = &name[1..name.len() - 1]; // Don't include the leading and trailing "
            (name, token.line)
        };

        let value = Value::Object(ObjectNode::alloc(
            Object::String(self.heap.strings.make_string(name)),
            &mut self.heap,
        ));
        self.emit_constant_with_op(value, line);
    }

    fn literal(&mut self, _can_assign: bool) {
        let (operator_type, token_line) = unsafe {
            let token = self.get_previous_token().unwrap_unchecked();
            (token.token_type, token.line)
        };

        let op = match operator_type {
            TokenType::False => OpCode::False,
            TokenType::True => OpCode::True,
            TokenType::None => OpCode::None,
            _ => unreachable!(),
        };
        self.emit_op(op, token_line);
    }

    fn unary(&mut self, _can_assign: bool) {
        let (operator_type, token_line) = unsafe {
            let token = self.get_previous_token().unwrap_unchecked();
            (token.token_type, token.line)
        };

        self.parse_precedence(Precedence::Unary);

        let op = match operator_type {
            TokenType::Not => OpCode::Not,
            TokenType::Minus => OpCode::Negate,
            _ => return,
        };

        self.emit_op(op, token_line);
    }

    fn binary(&mut self, _can_assign: bool) {
        let next_token_type = unsafe { self.get_current_token().unwrap_unchecked().token_type };
        let (operator_type, token_line) = unsafe {
            let token = self.get_previous_token().unwrap_unchecked();
            (token.token_type, token.line)
        };

        let parse_rule = self.get_rule(operator_type);
        self.parse_precedence(parse_rule.precedence.add(1));

        let op = match operator_type {
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
        self.emit_op(op, token_line);
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(
            TokenType::RightParen,
            "Expected ')' after grouping expression.",
        );
    }

    fn number(&mut self, _can_assign: bool) {
        let (num, token_line) = unsafe {
            let token = self.get_previous_token().unwrap_unchecked();
            let number = token
                .lexeme
                .get_str(self.get_current_source())
                .parse()
                .unwrap_unchecked();
            (number, token.line)
        };
        self.emit_constant_with_op(Value::Number(num), token_line);
    }

    fn advance(&mut self) {
        if let Err(err) = self.scanner.scan() {
            self.error(err.get_message());
            return;
        }

        dbg_println!(
            "Advanced to next token.\n\tlast: {:?}, current: {:?}",
            self.get_previous_token(),
            self.get_current_token()
        );
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
        self.emit_op(OpCode::Pop, self.get_current_line());
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
        if self.scope_depth <= 0 {
            self.error("Cannot end scope, no scope is open.");
            return;
        }

        self.scope_depth -= 1;

        let line = self.get_current_line();
        // Pop all locals in scope
        let mut locals_to_pop = 0;
        let mut pop_n = 0;
        //SAFETY: We do not modify the array, and locals are valid for the lifetime of Compiler.
        for local in unsafe { self.locals.unsafe_bottom_iter() } {
            if local.is_captured() {
                self.emit_op(OpCode::CloseUpvalue, line);
            } else {
                locals_to_pop += 1;
            }
            pop_n += 1;
        }
        for _ in 0..locals_to_pop {
            self.locals.pop();
        }

        if pop_n > 0 {
            self.emit_op_with_double(OpCode::PopN, pop_n, line);
        }
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump_backpatch(OpCode::JumpIfFalse);

        self.emit_op(OpCode::Pop, self.get_current_line());
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump.get_argument_index());
    }

    fn or(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump_backpatch(OpCode::JumpIfTrue);
        self.emit_op(OpCode::Pop, self.get_current_line());

        self.parse_precedence(Precedence::Or);

        self.patch_jump(end_jump.get_argument_index());
    }

    fn if_statement(&mut self) {
        self.expression();
        if !self.is_current_token(TokenType::LeftBrace) {
            self.error("Missing '{' after if statement.");
        }

        let then_jump = self.emit_jump_backpatch(OpCode::JumpIfFalse);
        self.emit_op(OpCode::Pop, self.get_current_line());

        self.statement();

        let else_jump = self.emit_jump_backpatch(OpCode::Jump);

        self.patch_jump(then_jump.get_argument_index());
        self.emit_op(OpCode::Pop, self.get_current_line());

        if self.match_and_advance(TokenType::Else) {
            if !self.is_current_token(TokenType::LeftBrace) {
                self.error("Missing '{' after else statement.");
            }
            self.statement();
        }
        self.patch_jump(else_jump.get_argument_index());
    }

    fn while_statement(&mut self) {
        let loop_start = self.get_instruction_count() as u32;

        self.expression();
        if !self.match_and_advance(TokenType::LeftBrace) {
            self.error("Missing '{' after while statement.");
        }

        let exit_jump = self.emit_jump_backpatch(OpCode::JumpIfFalse);
        self.emit_op(OpCode::Pop, self.get_current_line());

        self.enclosing_loop = Some(EnclosingLoop {
            start_jump_index: loop_start,
            exit_jump_index: exit_jump.instruction_index,
        });

        self.begin_scope();
        self.block();
        self.emit_backjump(loop_start);

        self.patch_jump(exit_jump.get_argument_index());
        self.emit_op(OpCode::Pop, self.get_current_line());
        self.end_scope();

        self.enclosing_loop = None;
    }

    fn for_statement(&mut self) {
        self.begin_scope();

        // Compile variable declaration
        if self.match_and_advance(TokenType::Let) {
            self.variable_declaration();
            self.consume(
                TokenType::Comma,
                "Expected ',' after for variable declaration.",
            );
        } else {
            self.error("Expected variable declaration in for loop.");
        }

        let mut loop_start = self.get_instruction_count();

        // Compile conditional
        self.expression();
        self.consume(TokenType::Comma, "Expected ','.");

        let exit_jump = self.emit_jump_backpatch(OpCode::JumpIfFalse);
        self.emit_op(OpCode::Pop, self.get_current_line());

        // Compile increment expression
        if !self.match_and_advance(TokenType::LeftBrace) {
            let body_jump = self.emit_jump_backpatch(OpCode::Jump);
            let increment_start = self.get_instruction_count();
            self.expression();
            self.emit_op(OpCode::Pop, self.get_current_line());

            self.emit_backjump(loop_start as u32);
            loop_start = increment_start;
            self.patch_jump(body_jump.get_argument_index());
        } else {
            self.error("Expected increment expression after conditional in for loop.");
        }

        if !self.match_and_advance(TokenType::LeftBrace) {
            self.error("Expected '{' after for clauses.");
        }

        self.enclosing_loop = Some(EnclosingLoop {
            start_jump_index: loop_start as u32,
            exit_jump_index: exit_jump.instruction_index as u32,
        });

        self.begin_scope();
        self.block();
        self.end_scope();

        self.emit_backjump(loop_start as u32);
        self.patch_jump(exit_jump.get_argument_index());
        self.emit_op(OpCode::Pop, self.get_current_line());
        self.end_scope();

        self.enclosing_loop = None;
    }

    fn continue_statement(&mut self) {
        match self.enclosing_loop.clone() {
            None => {
                self.error("Cannot use 'continue' outside of a loop.");
            }
            Some(enclosing_loop) => {
                // Make sure we discard current locals or the stack will slowly overflow.
                self.end_scope();
                self.begin_scope();
                self.emit_backjump(enclosing_loop.start_jump_index);
            }
        }
    }

    fn break_statement(&mut self) {
        match self.enclosing_loop.clone() {
            None => {
                self.error("Cannot use 'break' outside of a loop.");
            }
            Some(enclosing_loop) => {
                // Push false to the stack so the loop condition evaluates to false and exits.
                self.emit_constant_with_op(Value::Bool(false), self.get_current_line());
                self.emit_backjump(enclosing_loop.exit_jump_index);
            }
        }
    }

    fn return_statement(&mut self) {
        if self.current_function_type == FunctionType::Script {
            self.error("Cannot use 'return' in top-level code.");
        }

        if self.is_current_token(TokenType::RightBrace) {
            self.emit_empty_return();
        } else {
            self.expression();
            if !self.is_current_token(TokenType::RightBrace) {
                self.error("Expected '}' after return expression.\n The return statement must be the last statement in a block.");
            }
            self.emit_op(OpCode::Return, self.get_current_line());
        }
    }

    fn statement(&mut self) {
        dbg_println!("\nPARSING STATEMENT");

        if self.match_and_advance(TokenType::If) {
            self.if_statement();
        } else if self.match_and_advance(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else if self.match_and_advance(TokenType::For) {
            self.for_statement();
        } else if self.match_and_advance(TokenType::While) {
            self.while_statement();
        } else if self.match_and_advance(TokenType::Continue) {
            self.continue_statement();
        } else if self.match_and_advance(TokenType::Return) {
            self.return_statement();
        } else if self.match_and_advance(TokenType::Break) {
            self.break_statement();
        } else {
            self.expression_statement();
        }
    }

    /// Returns the index of the constant.
    fn identifier_constant(&mut self, name_token: &Token) -> u32 {
        let lexeme = name_token.lexeme.get_str(self.get_current_source());
        let value = Value::Object(ObjectNode::alloc(
            Object::String(self.heap.strings.make_string(lexeme)),
            &mut self.heap,
        ));
        self.emit_constant(value)
    }

    /// Returns None if no local variable with the name is found.
    fn resolve_local(&mut self, name_token: &Token) -> Option<u16> {
        for (i, local) in self.locals.bottom_iter().enumerate() {
            let local_name = local.get_name();

            if let None = local_name {
                // None is used for reserved slots, so just skip.
                continue;
            } else if let Some(local_name) = local_name {
                if name_token.lexeme.get_str(self.get_current_source())
                    == local_name.lexeme.get_str(self.get_current_source())
                {
                    if !local.is_initialized() {
                        self.error("Can't read uninitialized variable.");
                    }
                    return Some(i as u16);
                }
            }
        }

        None
    }

    fn add_upvalue(&mut self, index: u16, is_local: bool) -> u16 {
        let last_upvalue_count = self.current_function.upvalue_count;

        // Check if we already have an upvalue for the variable
        for (i, upvalue) in self.upvalues.bottom_iter().enumerate() {
            // i will never go above u16::MAX (as currently defined)
            let i = i as u16;
            if upvalue.index == i as u16 && upvalue.is_local == is_local {
                return i;
            }
        }

        if last_upvalue_count as usize >= UPVALUES_MAX {
            self.error("Too many closure variable in function!");
            return 0;
        }

        self.upvalues
            .set_at(last_upvalue_count as usize, Upvalue { is_local, index });
        self.current_function.upvalue_count += 1;
        last_upvalue_count
    }

    fn resolve_upvalue(&mut self, name_token: &Token) -> Option<u16> {
        match self.enclosing_compiler {
            Some(enclosing) => {
                let enclosing = unsafe { enclosing.as_mut_unchecked() };

                if let Some(enclosing_local) = enclosing.resolve_local(name_token) {
                    enclosing
                        .locals
                        .get_mut_at(enclosing_local as usize)
                        .capture();
                    let upvalue = self.add_upvalue(enclosing_local, true);
                    return Some(upvalue);
                } else if let Some(upvalue) = enclosing.resolve_upvalue(name_token) {
                    let upvalue = self.add_upvalue(upvalue as u16, false);
                    return Some(upvalue);
                } else {
                    return None;
                };
            }
            _ => None,
        }
    }

    fn named_variable(&mut self, name_token: &Token, can_assign: bool) -> u32 {
        enum VariableType {
            Local,
            Global,
            Upvalue,
        }

        let (slot, get_op, set_op, var_type) = if let Some(slot) = self.resolve_local(&name_token) {
            (
                slot as u32,
                OpCode::GetLocal,
                OpCode::SetLocal,
                VariableType::Local,
            )
        } else if let Some(upvalue) = self.resolve_upvalue(&name_token) {
            (
                upvalue as u32,
                OpCode::GetUpvalue,
                OpCode::SetUpvalue,
                VariableType::Upvalue,
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

        let op = if can_assign && self.match_and_advance(TokenType::Equal) {
            self.expression();
            set_op
        } else if can_assign && self.match_and_advance(TokenType::PlusEqual) {
            self.named_variable(name_token, can_assign);
            self.expression();
            self.emit_op(OpCode::Add, name_token.line);
            set_op
        } else if can_assign && self.match_and_advance(TokenType::MinusEqual) {
            self.named_variable(name_token, can_assign);
            self.expression();
            self.emit_op(OpCode::Subtract, name_token.line);
            set_op
        } else {
            get_op
        };

        match var_type {
            VariableType::Upvalue => {
                self.emit_op_with_double(op, slot as u16, name_token.line);
            }
            VariableType::Local => {
                self.emit_op_with_double(op, slot as u16, name_token.line);
            }
            VariableType::Global => {
                self.emit_op_with_quad(op, slot, name_token.line);
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

    /// Returns true if the we actually initialized a loca, and false if we are in global scope.
    fn initialize_local(&mut self) -> bool {
        // We are not in a local scope, so just return.
        if self.scope_depth < 1 {
            return false;
        }

        dbg_println!("DEFINING LAST DECLARED LOCAL VARIABLE");
        self.locals.top_mut_offset(0).initialize(self.scope_depth);
        true
    }

    fn define_variable(&mut self, global_index: u32) {
        if !self.initialize_local() {
            self.emit_op_with_quad(OpCode::DefineGlobal, global_index, self.get_current_line());
        }
    }

    fn variable_declaration(&mut self) {
        dbg_println!("\nPARSING VARIABLE DECL");

        let slot = self.parse_variable("Expected variable name.");

        if self.match_and_advance(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_op(OpCode::None, self.get_current_line());
        }

        self.define_variable(slot);
    }

    fn argument_list(&mut self) -> u8 {
        let mut arg_count = 0;
        if !self.is_current_token(TokenType::RightParen) {
            loop {
                self.expression();
                if arg_count >= MAX_FUNCTION_ARITY {
                    self.error(&format!(
                        "Function cannot have more than {} arguments.",
                        MAX_FUNCTION_ARITY
                    ));
                }
                arg_count += 1;
                if self.is_current_token(TokenType::RightParen) {
                    break;
                } else if self.match_and_advance(TokenType::Comma) {
                    continue;
                } else {
                    self.error("Expected ',' or ')' after function argument.");
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expected ')' after argument list.");
        arg_count
    }

    fn call(&mut self, _can_assign: bool) {
        let arg_count = self.argument_list();
        self.emit_op_with_byte(OpCode::Call, arg_count, self.get_current_line());
    }

    fn function(&mut self, function_type: FunctionType) {
        let mut compiler = self.fork(function_type);

        if function_type != FunctionType::Script {
            // Since we (the current compiler 'self') hit the fn token, the previous token is the function name.
            // which we set in the new Compiler that will compile the function.
            let previous_token = self
                .get_previous_token()
                .cloned()
                .expect("Expected function name token.");
            let func_name = self
                .heap
                .strings
                .make_string(previous_token.lexeme.get_str(self.current_source));
            compiler.current_function.name = Some(func_name);
        }

        compiler.begin_scope();
        compiler.consume(TokenType::LeftParen, "Expected '(' after function name.");
        if !compiler.is_current_token(TokenType::RightParen) {
            // Loop through parameters seperated by commas
            loop {
                if compiler.current_function.arity >= MAX_FUNCTION_ARITY {
                    compiler.error(&format!(
                        "Function cannot have more than {} parameters.",
                        MAX_FUNCTION_ARITY
                    ));
                }
                let constant = compiler.parse_variable("Expected parameter name.");
                compiler.define_variable(constant);
                compiler.current_function.arity += 1;
                if !compiler.match_and_advance(TokenType::Comma) {
                    break;
                }
            }
        }
        compiler.consume(
            TokenType::RightParen,
            "Expected ')' after function parameters.",
        );
        compiler.consume(TokenType::LeftBrace, "Expected '{' before function body.");
        compiler.block();

        let function = compiler.pack_function();

        #[cfg(debug_assertions)]
        {
            println!("COMPILED FUNCTION: {:?}", function);
            let chunk_name = match &function.name {
                Some(name) => name.as_str(),
                None => "<script>",
            }
            .to_owned();
            disassemble_chunk(&function.chunk, &chunk_name);
        }
        if compiler.had_error() {
            self.error(&format!(
                "Function {} had errors during compilation!",
                function
                    .name
                    .as_ref()
                    .map(|x| x.as_str().to_owned())
                    .unwrap_or("<unknown function>".to_owned())
            ));
        }

        let function_value = Value::Object(ObjectNode::alloc(
            Object::Function(function),
            &mut self.heap,
        ));
        let line = self.get_current_line();
        let constant_index = self.emit_constant(function_value);
        if constant_index > u16::MAX as u32 {
            self.error(&format!(
                "Too many constants! Cannot define more than {} constants",
                u16::MAX
            ));
        }
        self.emit_op_with_double(OpCode::Closure, constant_index as u16, line);

        let line = self.get_current_line();
        // SAFETY: We can use the unsafe_bottom_iter since it is guaranteed that the upvalues array will outlive this function.
        let upvalues_iter = unsafe { compiler.upvalues.unsafe_bottom_iter() };
        let chunk = self.get_current_chunk_mut();
        for upvalue in upvalues_iter {
            chunk.write_byte(upvalue.is_local as u8, line);
            chunk.write_double(upvalue.index, line);
        }
    }

    fn fn_declaration(&mut self) {
        dbg_println!("\nPARSING FUNCTION DECL");

        let global = self.parse_variable("Expected function name.");
        self.initialize_local();
        self.function(FunctionType::Function);
        self.define_variable(global);
    }

    fn declaration(&mut self) {
        if self.match_and_advance(TokenType::Let) {
            self.variable_declaration();
        } else if self.match_and_advance(TokenType::Fn) {
            self.fn_declaration();
        } else {
            self.statement();
        }

        if self.is_panic_mode() {
            self.synchronize();
        }
    }

    fn pack_function(&mut self) -> Function {
        // We always emit an empty return implicitly.
        self.emit_empty_return();
        self.current_function.clone()
    }

    pub fn compile(&mut self, source: &'src [u8]) -> CompilerResult<Function> {
        self.current_source = source;
        self.scanner.set_source(source);

        self.advance();
        while !self.match_and_advance(TokenType::EOF) {
            self.declaration();
        }

        if self.had_error() {
            Err("Parser encountered errors!".into())
        } else {
            Ok(self.pack_function())
        }
    }
}

impl Dealloc for Compiler<'_> {
    fn dealloc(&mut self) {
        dbg_println!("DEBUG COMPILER DEALLOC: {:?}", self);
        if !self.scanner.is_null() {
            self.scanner.dealloc();
            self.scanner = HeapPtr::null()
        }

        // We do not dealloc the heap, as it is managed by the VM.
    }
}
