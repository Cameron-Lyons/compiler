use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Special
    Illegal,
    Eof,

    // Identifiers + literals
    Ident,  // add, foobar, x, y, ...
    Int,    // 1343456
    String, // "string literals"

    // Operators
    Assign,   // "="
    Plus,     // "+"
    Minus,    // "-"
    Bang,     // "!"
    Asterisk, // "*"
    Slash,    // "/"

    Lt, // "<"
    Gt, // ">"

    Eq,     // "=="
    NotEq,  // "!="

    // Delimiters
    Comma,     // ","
    Semicolon, // ";"
    Colon,     // ":"

    LParen,    // "("
    RParen,    // ")"
    LBrace,    // "{"
    RBrace,    // "}"
    LBracket,  // "["
    RBracket,  // "]"

    // Keywords
    Function, // "fn"
    Let,      // "let"
    True,     // "true"
    False,    // "false"
    If,       // "if"
    Else,     // "else"
    Return,   // "return"
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

impl Token {
    pub fn new(token_type: TokenType, literal: String) -> Self {
        Token { token_type, literal }
    }
}

pub struct Keywords {
    map: HashMap<&'static str, TokenType>,
}

impl Keywords {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("fn", TokenType::Function);
        map.insert("let", TokenType::Let);
        map.insert("true", TokenType::True);
        map.insert("false", TokenType::False);
        map.insert("if", TokenType::If);
        map.insert("else", TokenType::Else);
        map.insert("return", TokenType::Return);

        Keywords { map }
    }

    pub fn lookup_ident(&self, ident: &str) -> TokenType {
        if let Some(token_type) = self.map.get(ident) {
            token_type.clone()
        } else {
            TokenType::Ident
        }
    }
}
