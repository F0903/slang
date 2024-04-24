use {
    crate::token::{Token, TokenType},
    std::{error::Error, fmt::Display, ptr::null},
};

#[derive(Debug)]
pub struct ScannerError {
    message: &'static str,
}

impl ScannerError {
    pub const fn new(message: &'static str) -> Self {
        Self { message }
    }

    pub const fn get_message(&self) -> &'static str {
        self.message
    }
}

impl Error for ScannerError {}

impl Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("SCANNER ERROR: {}", self.message))
    }
}

impl From<&'static str> for ScannerError {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

type ScannerResult<'a> = std::result::Result<Token, ScannerError>;

const fn error(message: &'static str) -> ScannerResult<'_> {
    Err(ScannerError::new(message))
}

const fn is_digit(ch: u8) -> bool {
    ch >= b'0' && ch <= b'9'
}

const fn is_alpha(ch: u8) -> bool {
    (ch >= b'a' && ch <= b'z') || (ch >= b'A' && ch <= b'Z') || ch == b'_'
}

pub struct Scanner {
    start: *const u8,
    current: *const u8,
    line: u32,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            start: null(),
            current: null(),
            line: 1,
        }
    }

    /// SOURCE MUST BE NULL TERMINATED
    pub fn set_source(&mut self, source: &[u8]) {
        self.start = source.as_ptr();
        self.current = self.start;
    }

    pub const fn get_current_line(&self) -> u32 {
        self.line
    }

    const fn is_at_end(&self) -> bool {
        unsafe { self.current.read() == b'\0' }
    }

    fn make_token(&self, typ: TokenType) -> ScannerResult<'_> {
        let name = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.start,
                self.current.offset_from(self.start) as usize,
            ))
        };
        Ok(Token::new(typ, name, self.line))
    }

    pub(crate) fn advance(&mut self) -> u8 {
        unsafe {
            let ch = self.current.read();
            self.current = self.current.add(1);
            ch
        }
    }

    fn match_current(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        unsafe {
            if self.current.read() != expected {
                return false;
            }
            self.current = self.current.add(1);
            true
        }
    }

    fn peek(&self) -> u8 {
        unsafe { self.current.read() }
    }

    fn peek_next(&self) -> Option<u8> {
        if self.is_at_end() {
            return None;
        }
        unsafe { Some(self.current.add(1).read()) }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let ch = self.peek();
            match ch {
                b' ' | b'\r' | b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                    break;
                }
                b'?' => {
                    // Skip comments
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }
                }
                _ => return,
            }
        }
    }

    fn string(&mut self) -> ScannerResult<'_> {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err("Unterminated string".into());
        }

        self.advance();
        self.make_token(TokenType::String)
    }

    fn number(&mut self) -> ScannerResult<'_> {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == b'.' && is_digit(self.peek_next().unwrap_or(b'\0')) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn check_keywords(&self, start: usize, keywords: &[(&str, TokenType)]) -> TokenType {
        for keyword in keywords {
            let name = keyword.0;
            let length = name.len();
            let name = name.as_ptr();
            let token_type = keyword.1;

            unsafe {
                if self.current.offset_from(self.start) == (start + length) as isize
                    && std::slice::from_raw_parts(self.start.add(start as usize), length as usize)
                        == std::slice::from_raw_parts(name, length as usize)
                {
                    return token_type;
                }
            }
        }
        TokenType::Identifier
    }

    fn identifier_type(&self) -> TokenType {
        unsafe {
            match self.start.read() {
                b'a' => self.check_keywords(1, &[("nd", TokenType::And)]),
                b'c' => self.check_keywords(1, &[("lass", TokenType::Class)]),
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

    fn identifier(&mut self) -> ScannerResult<'_> {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }
        let typ = self.identifier_type();
        self.make_token(typ)
    }

    pub fn scan(&mut self) -> ScannerResult<'_> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::EOF);
        }

        let ch = self.advance();

        if is_alpha(ch) {
            return self.identifier();
        } else if is_digit(ch) {
            return self.number();
        }

        match ch {
            b'(' => return self.make_token(TokenType::LeftParen),
            b')' => return self.make_token(TokenType::RightParen),
            b'{' => return self.make_token(TokenType::LeftBrace),
            b'}' => return self.make_token(TokenType::RightBrace),
            b';' => return self.make_token(TokenType::Semicolon),
            b',' => return self.make_token(TokenType::Comma),
            b'.' => return self.make_token(TokenType::Dot),
            b'-' => return self.make_token(TokenType::Minus),
            b'+' => return self.make_token(TokenType::Plus),
            b'/' => return self.make_token(TokenType::Slash),
            b'*' => return self.make_token(TokenType::Star),
            b'=' => return self.make_token(TokenType::Equal),
            b'<' => {
                if self.match_current(b'=') {
                    return self.make_token(TokenType::LessEqual);
                } else {
                    return self.make_token(TokenType::Less);
                }
            }
            b'>' => {
                if self.match_current(b'=') {
                    return self.make_token(TokenType::GreaterEqual);
                } else {
                    return self.make_token(TokenType::Greater);
                }
            }
            b'"' => return self.string(),
            _ => (),
        }

        error("Unexpected character.")
    }
}
