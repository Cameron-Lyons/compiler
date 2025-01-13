use crate::token::{lookup_ident, Token, TokenType};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: char,
}

impl Lexer {
    /// Creates a new `Lexer` from the given input string
    pub fn new(input: &str) -> Self {
        let mut l = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: '\0',
        };
        // Initialize by reading the first character
        l.read_char();
        l
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let mut tok = Token::new(TokenType::Illegal, "".to_string());

        match self.ch {
            '=' => {
                if self.peek_char() == '=' {
                    // '==' token
                    let ch = self.ch;
                    self.read_char();
                    let literal = format!("{}{}", ch, self.ch);
                    tok = Token::new(TokenType::Eq, literal);
                } else {
                    // '=' token
                    tok = new_token(TokenType::Assign, self.ch);
                }
            }
            '+' => {
                tok = new_token(TokenType::Plus, self.ch);
            }
            '-' => {
                tok = new_token(TokenType::Minus, self.ch);
            }
            '!' => {
                if self.peek_char() == '=' {
                    // '!=' token
                    let ch = self.ch;
                    self.read_char();
                    let literal = format!("{}{}", ch, self.ch);
                    tok = Token::new(TokenType::NotEq, literal);
                } else {
                    // '!' token
                    tok = new_token(TokenType::Bang, self.ch);
                }
            }
            '/' => {
                tok = new_token(TokenType::Slash, self.ch);
            }
            '*' => {
                tok = new_token(TokenType::Asterisk, self.ch);
            }
            '<' => {
                tok = new_token(TokenType::Lt, self.ch);
            }
            '>' => {
                tok = new_token(TokenType::Gt, self.ch);
            }
            ';' => {
                tok = new_token(TokenType::Semicolon, self.ch);
            }
            ',' => {
                tok = new_token(TokenType::Comma, self.ch);
            }
            '{' => {
                tok = new_token(TokenType::LBrace, self.ch);
            }
            '}' => {
                tok = new_token(TokenType::RBrace, self.ch);
            }
            '(' => {
                tok = new_token(TokenType::LParen, self.ch);
            }
            ')' => {
                tok = new_token(TokenType::RParen, self.ch);
            }
            '[' => {
                tok = new_token(TokenType::LBracket, self.ch);
            }
            ']' => {
                tok = new_token(TokenType::RBracket, self.ch);
            }
            '"' => {
                // Read string literal
                tok.token_type = TokenType::String;
                tok.literal = self.read_string();
            }
            ':' => {
                tok = new_token(TokenType::Colon, self.ch);
            }
            '\0' => {
                // End of file
                tok.token_type = TokenType::Eof;
                tok.literal = "".to_string();
            }
            _ => {
                if is_letter(self.ch) {
                    let literal = self.read_identifier();
                    let token_type = lookup_ident(&literal);
                    return Token::new(token_type, literal);
                } else if is_digit(self.ch) {
                    let number = self.read_number();
                    return Token::new(TokenType::Int, number);
                } else {
                    // Illegal character
                    tok = new_token(TokenType::Illegal, self.ch);
                }
            }
        }

        // Advance to the next character
        self.read_char();
        tok
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    fn skip_whitespace(&mut self) {
        while self.ch == ' ' || self.ch == '\t' || self.ch == '\n' || self.ch == '\r' {
            self.read_char();
        }
    }

    fn read_identifier(&mut self) -> String {
        let start_pos = self.position;
        while is_letter(self.ch) {
            self.read_char();
        }
        self.input[start_pos..self.position].iter().collect()
    }

    fn read_number(&mut self) -> String {
        let start_pos = self.position;
        while is_digit(self.ch) {
            self.read_char();
        }
        self.input[start_pos..self.position].iter().collect()
    }

    fn read_string(&mut self) -> String {
        let start_pos = self.position + 1; // skip opening quote
        loop {
            self.read_char();
            if self.ch == '"' || self.ch == '\0' {
                break;
            }
        }
        self.input[start_pos..self.position].iter().collect()
    }
}

fn is_letter(ch: char) -> bool {
    (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch == '_'
}

fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

fn new_token(token_type: TokenType, ch: char) -> Token {
    Token::new(token_type, ch.to_string())
}
