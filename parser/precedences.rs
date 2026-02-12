use lexer::token::TokenKind;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Precedence {
    Lowest,
    Equals,      // ==
    LessGreater, // > or <
    Sum,         // + or =
    Product,     // * or /
    Prefix,      // -X or !X
    Call,        // myFunction(x)
    Index,       // array[index]
}

pub fn get_token_precedence(token: &TokenKind) -> Precedence {
    match token {
        TokenKind::EQ => Precedence::Equals,
        TokenKind::NotEq => Precedence::Equals,
        TokenKind::LT => Precedence::LessGreater,
        TokenKind::GT => Precedence::LessGreater,
        TokenKind::LTE => Precedence::LessGreater,
        TokenKind::GTE => Precedence::LessGreater,
        TokenKind::PLUS => Precedence::Sum,
        TokenKind::MINUS => Precedence::Sum,
        TokenKind::ASTERISK => Precedence::Product,
        TokenKind::SLASH => Precedence::Product,
        TokenKind::PERCENT => Precedence::Product,
        TokenKind::LPAREN => Precedence::Call,
        TokenKind::LBRACKET => Precedence::Index,
        _ => Precedence::Lowest,
    }
}
