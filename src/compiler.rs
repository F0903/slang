use std::rc::Rc;

#[cfg(debug_assertions)]
use crate::debug::disassemble_chunk;
use {
    crate::{chunk::Chunk, opcode::OpCode, parser::Parser, scanner::Scanner, token::TokenType},
    std::cell::RefCell,
};

// Make concrete error type for compiler.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Compiler<'a> {
    current_chunk: Rc<RefCell<Chunk>>,
    parser: Parser<'a>,
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        let chunk = Rc::new(RefCell::new(Chunk::new()));
        Self {
            current_chunk: chunk.clone(),
            parser: Parser::new(Scanner::new(), chunk),
        }
    }

    pub fn compile(&mut self, source: &[u8]) -> Result<Rc<RefCell<Chunk>>> {
        self.parser.set_scanner_source(source);
        self.parser.set_current_chunk(self.current_chunk.clone());

        self.parser.advance();
        self.parser.expression();
        self.parser.consume(TokenType::EOF, "Expected end of file.");
        self.current_chunk
            .borrow_mut()
            .write_opcode(OpCode::Return, self.parser.get_current_line());

        #[cfg(debug_assertions)]
        if !self.parser.had_error() {
            disassemble_chunk(&mut self.current_chunk.borrow_mut(), "code");
        }

        self.current_chunk.borrow_mut().encode();
        if self.parser.had_error() {
            Err("Parser encountered errors!".into())
        } else {
            Ok(self.current_chunk.clone())
        }
    }
}
