use std::ptr::NonNull;

use super::{
    scanner_error::ScannerError,
    span::Span,
    token::{Token, TokenType},
};

type ScannerResult<'t> = std::result::Result<&'t Token, ScannerError>;

fn error<'t>(message: impl ToString) -> ScannerResult<'t> {
    Err(ScannerError::new(message))
}

#[inline]
const fn is_digit(ch: u8) -> bool {
    ch >= b'0' && ch <= b'9'
}

#[inline]
const fn is_alpha(ch: u8) -> bool {
    (ch >= b'a' && ch <= b'z') || (ch >= b'A' && ch <= b'Z') || ch == b'_'
}

#[derive(Debug)]
pub struct Scanner<'src> {
    start: NonNull<u8>,
    current_start: NonNull<u8>,
    current_end: NonNull<u8>,
    end: NonNull<u8>,
    line: u32,
    current_token: Option<Token>,
    previous_token: Option<Token>,
    _src_lifetime: std::marker::PhantomData<&'src [u8]>,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src [u8]) -> Self {
        let src_ptr = source.as_ptr() as *mut u8;

        // SAFETY: For some reason NonNull only takes a *mut, but the Scanner will never mutatate the source buffer.
        let start = unsafe { NonNull::new_unchecked(src_ptr) };
        let current_start = start;
        let current_end = current_start;
        // SAFETY: This is guaranteed to be in-bounds, since we are getting the length from source, where we are also pointing to.
        let end = unsafe { NonNull::new_unchecked(src_ptr.add(source.len())) };

        Self {
            start,
            current_start,
            current_end,
            end,
            line: 1,
            current_token: None,
            previous_token: None,
            _src_lifetime: std::marker::PhantomData,
        }
    }

    #[inline]
    pub const fn get_current_line(&self) -> u32 {
        self.line
    }

    #[inline]
    pub fn get_current_token(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    #[inline]
    pub fn get_previous_token(&self) -> Option<&Token> {
        self.previous_token.as_ref()
    }

    #[inline]
    fn is_at_end(&self) -> bool {
        self.current_end >= self.end
    }

    fn make_token(&mut self, typ: TokenType) -> &Token {
        // SAFETY: make_token is only called from places where we are guaranteed to be in-bounds.
        let (start, _len, end) = unsafe {
            let start = self.current_start.offset_from(self.start) as usize;
            let len = self.current_end.offset_from(self.current_start) as usize;
            (start, len, start + len)
        };

        // If we are bulding with debug mode, include the hash to double check the source.
        #[cfg(debug_assertions)]
        let lexeme_span = {
            use crate::hashing::{GlobalHashMethod, HashMethod};

            let lexeme_str = unsafe {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    self.current_start.as_ptr(),
                    _len,
                ))
            };
            let hash = GlobalHashMethod::hash(lexeme_str.as_bytes());
            Span::new(start, end, hash)
        };
        #[cfg(not(debug_assertions))]
        let lexeme_span = Span::new(start, end);

        let token = Token::new(typ, lexeme_span, self.line);
        self.previous_token = self.current_token.take();
        self.current_token = Some(token);
        self.current_token.as_ref().unwrap()
    }

    // Gets the current character and advances to the next
    #[inline]
    fn get_and_advance(&mut self) -> u8 {
        // SAFETY: this method is only called from places where we are guaranteed to be in-bounds.
        unsafe {
            let ch = self.current_end.read();
            self.current_end = self.current_end.add(1);
            ch
        }
    }

    #[inline]
    fn match_current(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }

        // SAFETY: self.current_end is always guaranteed to be valid here due to the check above.
        unsafe {
            if self.current_end.read() != expected {
                return false;
            }
            self.current_end = self.current_end.add(1);
            true
        }
    }

    #[inline]
    fn peek(&self) -> u8 {
        // SAFETY: this function is only called from places where self.curent_end is guaranteed to be valid.
        unsafe { self.current_end.read() }
    }

    #[inline]
    fn peek_next(&self) -> Option<u8> {
        if self.is_at_end() {
            return None;
        }

        // SAFETY: self.current_end is always guaranteed to be valid here due to the check above.
        unsafe { Some(self.current_end.add(1).read()) }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let ch = self.peek();
            match ch {
                b' ' | b'\r' | b'\t' => {
                    self.get_and_advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.get_and_advance();
                }
                b'?' => {
                    // Skip comments
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.get_and_advance();
                    }
                }
                _ => return,
            }
        }
    }

    fn string<'t>(&'t mut self) -> ScannerResult<'t> {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.get_and_advance();
        }

        if self.is_at_end() {
            return Err(format!("Unterminated string at {}", self.line).into());
        }

        self.get_and_advance();
        let token = self.make_token(TokenType::String);
        Ok(token)
    }

    fn number<'t>(&'t mut self) -> ScannerResult<'t> {
        while is_digit(self.peek()) {
            self.get_and_advance();
        }

        if self.peek() == b'.' && is_digit(self.peek_next().unwrap_or(b'\0')) {
            self.get_and_advance();

            while is_digit(self.peek()) {
                self.get_and_advance();
            }
        }

        let token = self.make_token(TokenType::Number);
        Ok(token)
    }

    fn check_keywords(&self, start: usize, keywords: &[(&str, TokenType)]) -> TokenType {
        for keyword in keywords {
            let name = keyword.0;
            let length = name.len();
            let name = name.as_ptr();
            let token_type = keyword.1;

            // SAFETY: we are always in-bounds here
            unsafe {
                if self.current_end.offset_from(self.current_start) == (start + length) as isize
                    && std::slice::from_raw_parts(
                        self.current_start.add(start as usize).as_ptr(),
                        length as usize,
                    ) == std::slice::from_raw_parts(name, length as usize)
                {
                    return token_type;
                }
            }
        }
        TokenType::Identifier
    }

    fn identifier_type(&self) -> TokenType {
        // SAFETY: self.current_start is always guaranteed to be in-bounds when this is called.
        unsafe {
            match self.current_start.read() {
                b'a' => self.check_keywords(1, &[("nd", TokenType::And)]),
                b'b' => self.check_keywords(1, &[("reak", TokenType::Break)]),
                b'c' => self.check_keywords(
                    1,
                    &[("lass", TokenType::Class), ("ontinue", TokenType::Continue)],
                ),
                b'e' => self.check_keywords(1, &[("else", TokenType::Else)]),
                b'f' => self.check_keywords(
                    1,
                    &[
                        ("or", TokenType::For),
                        ("alse", TokenType::False),
                        ("n", TokenType::Fn),
                    ],
                ),
                b'i' => self.check_keywords(1, &[("f", TokenType::If), ("s", TokenType::Is)]),
                b'n' => self.check_keywords(1, &[("ot", TokenType::Not), ("one", TokenType::None)]),
                b'o' => self.check_keywords(1, &[("r", TokenType::Or)]),
                b'r' => self.check_keywords(1, &[("eturn", TokenType::Return)]),
                b's' => self.check_keywords(1, &[("uper", TokenType::Super)]),
                b't' => {
                    self.check_keywords(1, &[("rue", TokenType::True), ("his", TokenType::This)])
                }
                b'l' => self.check_keywords(1, &[("et", TokenType::Let)]),
                b'w' => self.check_keywords(1, &[("hile", TokenType::While)]),
                _ => TokenType::Identifier,
            }
        }
    }

    fn identifier<'t>(&'t mut self) -> ScannerResult<'t> {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.get_and_advance();
        }
        let typ = self.identifier_type();
        let token = self.make_token(typ);
        Ok(token)
    }

    pub fn scan<'t>(&'t mut self) -> ScannerResult<'t> {
        self.skip_whitespace();
        self.current_start = self.current_end;

        if self.is_at_end() {
            let token = self.make_token(TokenType::EOF);
            return Ok(token);
        }

        let ch = self.get_and_advance();

        if is_alpha(ch) {
            return self.identifier();
        } else if is_digit(ch) {
            return self.number();
        }

        let token = match ch {
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b';' => self.make_token(TokenType::Semicolon),
            b',' => self.make_token(TokenType::Comma),
            b'.' => self.make_token(TokenType::Dot),
            b'-' => {
                if self.match_current(b'=') {
                    self.make_token(TokenType::MinusEqual)
                } else {
                    self.make_token(TokenType::Minus)
                }
            }
            b'+' => {
                if self.match_current(b'=') {
                    self.make_token(TokenType::PlusEqual)
                } else {
                    self.make_token(TokenType::Plus)
                }
            }
            b'/' => self.make_token(TokenType::Slash),
            b'*' => self.make_token(TokenType::Star),
            b'=' => self.make_token(TokenType::Equal),
            b'<' => {
                if self.match_current(b'=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                }
            }
            b'>' => {
                if self.match_current(b'=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                }
            }
            b'"' => self.string()?,
            _ => return error(format!("Unexpected character '{}'", ch)),
        };

        Ok(token)
    }
}
