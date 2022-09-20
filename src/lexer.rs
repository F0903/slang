use super::token::Token;
use crate::{create_string_map, error::get_err_handler, token::TokenType, value::Value};
use once_cell::sync::Lazy;
use std::collections::HashMap;

static KEYWORDS: Lazy<HashMap<String, TokenType>> = Lazy::new(|| {
    create_string_map!(
        "and"      => TokenType::And,
        "class"    => TokenType::Class,
        "else"     => TokenType::Else,
        "false"    => TokenType::False,
        "true"     => TokenType::True,
        "for"      => TokenType::For,
        "if"       => TokenType::If,
        "none"     => TokenType::None,
        "or"       => TokenType::Or,
        "is"       => TokenType::Is,
        "not"      => TokenType::Not,
        "return"   => TokenType::Return,
        "super"    => TokenType::Super,
        "this"     => TokenType::This,
        "offering" => TokenType::Offering,
        "ritual"   => TokenType::Ritual,
        "end"      => TokenType::End,
        "while"    => TokenType::While
    )
});

pub struct Lexer {
    source: String,
    start: usize,
    current: usize,
    line: usize,
    ignore_newline: bool,
    last_token: Option<Token>,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Lexer {
            source,
            start: 0,
            current: 0,
            line: 0,
            ignore_newline: false,
            last_token: None,
        }
    }

    fn at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn get_current_char_unchecked(&self) -> char {
        unsafe { self.source.chars().nth(self.current).unwrap_unchecked() }
    }

    fn peek(&self) -> char {
        if self.at_end() {
            '\0'
        } else {
            self.get_current_char_unchecked()
        }
    }

    fn peekpeek(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            unsafe { self.source.chars().nth(self.current + 1).unwrap_unchecked() }
        }
    }

    fn next_char(&mut self) -> char {
        let ch = self.get_current_char_unchecked();
        self.current += 1;
        ch
    }

    fn make_token(&mut self, token_type: TokenType) -> Token {
        let text = self.source[self.start..self.current].to_owned();
        let token = Token::new(token_type, text, Value::None, self.line);
        token
    }

    fn make_token_literal(&mut self, token_type: TokenType, literal: Value) -> Token {
        let text = self.source[self.start..self.current].to_owned();
        let token = Token::new(token_type, text, literal, self.line);
        token
    }

    fn matches_next(&mut self, ch: char) -> bool {
        if self.at_end() {
            return false;
        }
        if self.peek() != ch {
            return false;
        }

        self.current += 1;
        return true;
    }

    const fn alphanumeric_or_underscore(ch: char) -> bool {
        ch == '_' || ch.is_ascii_alphanumeric()
    }

    fn handle_string(&mut self) -> Option<Token> {
        while self.peek() != '"' && !self.at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.next_char();
        }

        if self.at_end() {
            get_err_handler().report(self.line, "Unterminated string.");
            return None;
        }

        // Closing "
        self.next_char();

        let literal = self.source[self.start + 1..self.current - 1].to_owned();
        Some(self.make_token_literal(TokenType::String, Value::String(literal)))
    }

    fn handle_number(&mut self) -> Option<Token> {
        while self.peek().is_ascii_digit() {
            self.next_char();
        }

        if self.peek() == '.' && self.peekpeek().is_ascii_digit() {
            // Consume the .
            self.next_char();
            while self.peek().is_ascii_digit() {
                self.next_char();
            }
        }

        let value = match self.source[self.start..self.current].parse::<f64>() {
            Ok(x) => x,
            Err(_) => {
                get_err_handler().report(self.line, "Could not parse number!");
                return None;
            }
        };
        Some(self.make_token_literal(TokenType::Number, Value::Number(value)))
    }

    fn handle_identifier(&mut self) -> Token {
        while Self::alphanumeric_or_underscore(self.peek()) {
            self.next_char();
        }

        let text = self.source[self.start..self.current].to_owned();
        let token_type = match KEYWORDS.get(&text) {
            Some(x) => *x,
            None => TokenType::Identifier,
        };

        self.make_token(token_type)
    }

    fn is_maybe_stmt_end(test_type: &TokenType) -> bool {
        static STMT_END_TOKENS: &'static [TokenType] = &[
            TokenType::BraceClose,
            TokenType::ParenClose,
            TokenType::SquareClose,
            TokenType::True,
            TokenType::False,
            TokenType::Number,
            TokenType::String,
            TokenType::None,
            TokenType::End,
            TokenType::Identifier,
        ];
        STMT_END_TOKENS.iter().any(|x| x == test_type)
    }

    fn lex_token(&mut self) -> Token {
        self.start = self.current;
        let next = self.next_char();
        match next {
            '?' => {
                // Skip line, is a comment.
                while self.peek() != '\n' && !self.at_end() {
                    self.next_char();
                }
                self.lex_token()
            }
            ' ' | '\r' | '\t' => self.lex_token(),
            '\n' => {
                self.line += 1;
                if !self.ignore_newline {
                    let last = match &self.last_token {
                        Some(x) => x,
                        None => return self.lex_token(),
                    };
                    if !Self::is_maybe_stmt_end(&last.token_type) {
                        return self.lex_token();
                    }
                    return self.make_token(TokenType::StatementEnd);
                }
                self.lex_token()
            }
            '(' => {
                self.ignore_newline = true;
                self.make_token(TokenType::ParenOpen)
            }
            ')' => {
                self.ignore_newline = false;
                self.make_token(TokenType::ParenClose)
            }
            '[' => {
                self.ignore_newline = true;
                self.make_token(TokenType::SquareOpen)
            }
            ']' => {
                self.ignore_newline = false;
                self.make_token(TokenType::SquareClose)
            }
            '{' => self.make_token(TokenType::BraceOpen),
            '}' => self.make_token(TokenType::BraceClose),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => {
                if self.matches_next('-') {
                    self.make_token(TokenType::MinusMinus)
                } else if self.matches_next('=') {
                    self.make_token(TokenType::MinusEqual)
                } else {
                    self.make_token(TokenType::Minus)
                }
            }
            '+' => {
                if self.matches_next('+') {
                    self.make_token(TokenType::PlusPlus)
                } else if self.matches_next('=') {
                    self.make_token(TokenType::PlusEqual)
                } else {
                    self.make_token(TokenType::Plus)
                }
            }
            '*' => self.make_token(TokenType::Multiply),
            '/' => self.make_token(TokenType::Divide),
            '=' => self.make_token(TokenType::Equal),
            '$' => {
                let token = if self.matches_next('>') {
                    TokenType::DollarGreater
                } else if self.matches_next('<') {
                    TokenType::DollarLess
                } else {
                    return self.lex_token();
                };
                self.make_token(token)
            }
            '<' => {
                let token = match self.matches_next('=') {
                    true => TokenType::LessEqual,
                    false => TokenType::Less,
                };
                self.make_token(token)
            }
            '>' => {
                let token = match self.matches_next('=') {
                    true => TokenType::GreaterEqual,
                    false => TokenType::Greater,
                };
                self.make_token(token)
            }
            '"' => match self.handle_string() {
                Some(x) => x,
                None => self.lex_token(),
            },
            _ => {
                if next.is_ascii_digit() {
                    match self.handle_number() {
                        Some(x) => return x,
                        None => return self.lex_token(),
                    }
                } else if next.is_ascii_alphanumeric() {
                    return self.handle_identifier();
                }
                self.lex_token()
            }
        }
    }

    //TODO: Convert to iterator
    pub fn lex(&mut self) -> Token {
        if self.at_end() {
            if let Some(x) = &self.last_token {
                let eof = Token::new(TokenType::EOF, "EOF".to_owned(), Value::None, self.line);
                if let TokenType::EOF = x.token_type {
                    return eof;
                }
                self.last_token = Some(eof);
                return Token::new(
                    TokenType::StatementEnd,
                    "\n".to_owned(),
                    Value::None,
                    self.line,
                );
            }
        }

        let token = self.lex_token();
        self.last_token = Some(token.clone());
        token
    }
}

impl Iterator for Lexer {
    type Item = Token;

    /// Will always return Some, so check the returned token for EOF.
    fn next(&mut self) -> Option<Self::Item> {
        let token = self.lex();
        Some(token)
    }
}
