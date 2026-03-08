use crate::token::{Span, Token, TokenKind, lookup_identifier};

#[cfg(test)]
mod lexer_test;
pub mod token;

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    read_position: usize,
    ch: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Self {
            input,
            position: 0,
            read_position: 0,
            ch: None,
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.position = self.input.len();
            self.read_position = self.input.len();
            self.ch = None;
            return;
        }

        let mut chars = self.input[self.read_position..].chars();
        if let Some(ch) = chars.next() {
            self.position = self.read_position;
            self.read_position += ch.len_utf8();
            self.ch = Some(ch);
        } else {
            self.position = self.input.len();
            self.read_position = self.input.len();
            self.ch = None;
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.read_position >= self.input.len() {
            None
        } else {
            self.input[self.read_position..].chars().next()
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_ignored();

        let start = self.position;
        match self.ch {
            Some('=') => self.read_operator_token(start, TokenKind::ASSIGN, TokenKind::EQ),
            Some(';') => self.read_single_char_token(start, TokenKind::SEMICOLON),
            Some('(') => self.read_single_char_token(start, TokenKind::LPAREN),
            Some(')') => self.read_single_char_token(start, TokenKind::RPAREN),
            Some(',') => self.read_single_char_token(start, TokenKind::COMMA),
            Some('+') => self.read_single_char_token(start, TokenKind::PLUS),
            Some('-') => self.read_single_char_token(start, TokenKind::MINUS),
            Some('!') => self.read_operator_token(start, TokenKind::BANG, TokenKind::NotEq),
            Some('*') => self.read_single_char_token(start, TokenKind::ASTERISK),
            Some('/') => self.read_single_char_token(start, TokenKind::SLASH),
            Some('<') => self.read_operator_token(start, TokenKind::LT, TokenKind::LTE),
            Some('>') => self.read_operator_token(start, TokenKind::GT, TokenKind::GTE),
            Some('%') => self.read_single_char_token(start, TokenKind::PERCENT),
            Some('{') => self.read_single_char_token(start, TokenKind::LBRACE),
            Some('}') => self.read_single_char_token(start, TokenKind::RBRACE),
            Some('[') => self.read_single_char_token(start, TokenKind::LBRACKET),
            Some(':') => self.read_single_char_token(start, TokenKind::COLON),
            Some(']') => self.read_single_char_token(start, TokenKind::RBRACKET),
            Some('"') => {
                let (end, string) = self.read_string();
                Token {
                    span: Span { start, end },
                    kind: TokenKind::STRING(string),
                }
            }
            None => Token {
                span: Span { start, end: start },
                kind: TokenKind::EOF,
            },
            Some(ch) if is_letter(ch) => {
                let (end, identifier) = self.read_identifier();
                Token {
                    span: Span { start, end },
                    kind: lookup_identifier(&identifier),
                }
            }
            Some(ch) if is_digit(ch) => {
                let (end, raw_number) = self.read_number();
                let kind = match raw_number.parse() {
                    Ok(num) => TokenKind::INT(num),
                    Err(_) => TokenKind::ILLEGAL,
                };

                Token {
                    span: Span { start, end },
                    kind,
                }
            }
            Some(_) => self.read_single_char_token(start, TokenKind::ILLEGAL),
        }
    }

    fn read_single_char_token(&mut self, start: usize, kind: TokenKind) -> Token {
        self.read_char();
        Token {
            span: Span {
                start,
                end: self.position,
            },
            kind,
        }
    }

    fn read_operator_token(&mut self, start: usize, single: TokenKind, double: TokenKind) -> Token {
        let kind = if self.peek_char() == Some('=') {
            self.read_char();
            double
        } else {
            single
        };

        self.read_char();

        Token {
            span: Span {
                start,
                end: self.position,
            },
            kind,
        }
    }

    fn skip_ignored(&mut self) {
        loop {
            self.skip_whitespace();
            if !self.skip_comment() {
                break;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.ch, Some(ch) if ch.is_ascii_whitespace()) {
            self.read_char();
        }
    }

    fn skip_comment(&mut self) -> bool {
        if self.ch == Some('/') && self.peek_char() == Some('/') {
            while !matches!(self.ch, Some('\n') | None) {
                self.read_char();
            }
            true
        } else {
            false
        }
    }

    fn read_identifier(&mut self) -> (usize, String) {
        let start = self.position;
        while matches!(self.ch, Some(ch) if is_letter(ch)) {
            self.read_char();
        }

        (self.position, self.input[start..self.position].to_string())
    }

    fn read_number(&mut self) -> (usize, String) {
        let start = self.position;
        while matches!(self.ch, Some(ch) if is_digit(ch)) {
            self.read_char();
        }

        (self.position, self.input[start..self.position].to_string())
    }

    fn read_string(&mut self) -> (usize, String) {
        let mut result = String::new();

        self.read_char();

        while let Some(ch) = self.ch {
            match ch {
                '"' => {
                    let end = self.read_position;
                    self.read_char();
                    return (end, result);
                }
                '\\' => {
                    self.read_char();
                    match self.ch {
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('r') => result.push('\r'),
                        Some('\\') => result.push('\\'),
                        Some('"') => result.push('"'),
                        Some(other) => {
                            result.push('\\');
                            result.push(other);
                        }
                        None => {
                            result.push('\\');
                            return (self.input.len(), result);
                        }
                    }
                }
                other => result.push(other),
            }

            self.read_char();
        }

        (self.input.len(), result)
    }
}

fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}
