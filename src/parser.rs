use std::collections::HashMap;

use crate::ast::*;
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};

// Precedence Constants
const _PREC_DUMMY: i32 = 0;
const LOWEST: i32 = 1;
const EQUALS: i32 = 2; // ==
const LESSGREATER: i32 = 3; // > or <
const SUM: i32 = 4; // +
const PRODUCT: i32 = 5; // *
const PREFIX: i32 = 6; // -X or !X
const CALL: i32 = 7; // myFunction(X)
const INDEX: i32 = 8; // array[index]

lazy_static::lazy_static! {
    static ref PRECEDENCES: HashMap<TokenType, i32> = {
        let mut m = HashMap::new();
        m.insert(TokenType::Eq, EQUALS);
        m.insert(TokenType::NotEq, EQUALS);
        m.insert(TokenType::Lt, LESSGREATER);
        m.insert(TokenType::Gt, LESSGREATER);
        m.insert(TokenType::Plus, SUM);
        m.insert(TokenType::Minus, SUM);
        m.insert(TokenType::Slash, PRODUCT);
        m.insert(TokenType::Asterisk, PRODUCT);
        m.insert(TokenType::LParen, CALL);
        m.insert(TokenType::LBracket, INDEX);
        m
    };
}

type PrefixParseFn = fn(&mut Parser) -> Option<Expression>;
type InfixParseFn = fn(&mut Parser, Expression) -> Option<Expression>;

pub struct Parser {
    pub l: Lexer,
    pub errors: Vec<String>,

    pub cur_token: Token,
    pub peek_token: Token,

    prefix_parse_fns: HashMap<TokenType, PrefixParseFn>,
    infix_parse_fns: HashMap<TokenType, InfixParseFn>,
}

impl Parser {
    pub fn new(mut l: Lexer) -> Self {
        let mut p = Parser {
            l,
            errors: vec![],
            cur_token: Token {
                token_type: TokenType::Illegal,
                literal: String::new(),
            },
            peek_token: Token {
                token_type: TokenType::Illegal,
                literal: String::new(),
            },
            prefix_parse_fns: HashMap::new(),
            infix_parse_fns: HashMap::new(),
        };

        p.register_prefix(TokenType::Ident, Parser::parse_identifier);
        p.register_prefix(TokenType::Int, Parser::parse_integer_literal);
        p.register_prefix(TokenType::Bang, Parser::parse_prefix_expression);
        p.register_prefix(TokenType::Minus, Parser::parse_prefix_expression);
        p.register_prefix(TokenType::True, Parser::parse_boolean);
        p.register_prefix(TokenType::False, Parser::parse_boolean);
        p.register_prefix(TokenType::LParen, Parser::parse_grouped_expression);
        p.register_prefix(TokenType::If, Parser::parse_if_expression);
        p.register_prefix(TokenType::Function, Parser::parse_function_literal);
        p.register_prefix(TokenType::String, Parser::parse_string_literal);
        p.register_prefix(TokenType::LBracket, Parser::parse_array_literal);
        p.register_prefix(TokenType::LBrace, Parser::parse_hash_literal);

        // Register infix functions
        p.register_infix(TokenType::Plus, Parser::parse_infix_expression);
        p.register_infix(TokenType::Minus, Parser::parse_infix_expression);
        p.register_infix(TokenType::Slash, Parser::parse_infix_expression);
        p.register_infix(TokenType::Asterisk, Parser::parse_infix_expression);
        p.register_infix(TokenType::Eq, Parser::parse_infix_expression);
        p.register_infix(TokenType::NotEq, Parser::parse_infix_expression);
        p.register_infix(TokenType::Lt, Parser::parse_infix_expression);
        p.register_infix(TokenType::Gt, Parser::parse_infix_expression);
        p.register_infix(TokenType::LBracket, Parser::parse_index_expression);
        p.register_infix(TokenType::LParen, Parser::parse_call_expression);

        // Prime the pump: read two tokens
        p.next_token();
        p.next_token();

        p
    }

    pub fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.l.next_token();
    }

    fn cur_token_is(&self, t: TokenType) -> bool {
        self.cur_token.token_type == t
    }

    fn peek_token_is(&self, t: TokenType) -> bool {
        self.peek_token.token_type == t
    }

    fn expect_peek(&mut self, t: TokenType) -> bool {
        if self.peek_token_is(t.clone()) {
            self.next_token();
            true
        } else {
            self.peek_error(t);
            false
        }
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }

    fn peek_error(&mut self, t: TokenType) {
        let msg = format!(
            "expected next token to be {:?}, got {:?} instead",
            t, self.peek_token.token_type
        );
        self.errors.push(msg);
    }

    fn no_prefix_parse_fn_error(&mut self, t: TokenType) {
        let msg = format!("no prefix parse function for {:?} found", t);
        self.errors.push(msg);
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program { statements: vec![] };

        while !self.cur_token_is(TokenType::Eof) {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }
            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.token_type {
            TokenType::Let => self.parse_let_statement().map(Statement::LetStatement),
            TokenType::Return => self
                .parse_return_statement()
                .map(Statement::ReturnStatement),
            _ => self
                .parse_expression_statement()
                .map(Statement::ExpressionStatement),
        }
    }

    fn parse_let_statement(&mut self) -> Option<LetStatement> {
        let token = self.cur_token.clone();

        // expecting an identifier
        if !self.expect_peek(TokenType::Ident) {
            return None;
        }
        let name = Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };

        // expecting '='
        if !self.expect_peek(TokenType::Assign) {
            return None;
        }

        // move past '='
        self.next_token();

        // parse expression
        let value = self.parse_expression(LOWEST);

        // if expression is a function literal, store the name in it
        if let Some(Expression::FunctionLiteral(fl)) = &value {
            // clone the struct so we can mutate
            let mut fl_modified = fl.clone();
            fl_modified.name = Some(name.value.clone());
            // re-wrap in expression
            let new_expr = Expression::FunctionLiteral(fl_modified);
            return Some(LetStatement {
                token,
                name,
                value: Some(new_expr),
            });
        }

        // optional semicolon
        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(LetStatement { token, name, value })
    }

    fn parse_return_statement(&mut self) -> Option<ReturnStatement> {
        let token = self.cur_token.clone();

        self.next_token();

        let return_value = self.parse_expression(LOWEST);

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(ReturnStatement {
            token,
            return_value,
        })
    }

    fn parse_expression_statement(&mut self) -> Option<ExpressionStatement> {
        let token = self.cur_token.clone();

        let expression = self.parse_expression(LOWEST);

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(ExpressionStatement { token, expression })
    }

    fn parse_expression(&mut self, precedence: i32) -> Option<Expression> {
        let prefix = self
            .prefix_parse_fns
            .get(&self.cur_token.token_type)
            .copied();

        if prefix.is_none() {
            self.no_prefix_parse_fn_error(self.cur_token.token_type.clone());
            return None;
        }

        let mut left_exp = prefix.unwrap()(self)?;

        while !self.peek_token_is(TokenType::Semicolon) && precedence < self.peek_precedence() {
            let infix = self
                .infix_parse_fns
                .get(&self.peek_token.token_type)
                .copied();
            if infix.is_none() {
                return Some(left_exp);
            }
            self.next_token();
            left_exp = infix.unwrap()(self, left_exp)?;
        }

        Some(left_exp)
    }

    fn peek_precedence(&self) -> i32 {
        if let Some(p) = PRECEDENCES.get(&self.peek_token.token_type) {
            *p
        } else {
            LOWEST
        }
    }

    fn cur_precedence(&self) -> i32 {
        if let Some(p) = PRECEDENCES.get(&self.cur_token.token_type) {
            *p
        } else {
            LOWEST
        }
    }

    fn parse_identifier(&mut self) -> Option<Expression> {
        Some(Expression::Identifier(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_integer_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let value = match self.cur_token.literal.parse::<i64>() {
            Ok(v) => v,
            Err(_) => {
                let msg = format!("could not parse {:?} as integer", self.cur_token.literal);
                self.errors.push(msg);
                return None;
            }
        };
        Some(Expression::IntegerLiteral(IntegerLiteral { token, value }))
    }

    fn parse_prefix_expression(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();

        self.next_token();

        let right = self.parse_expression(PREFIX);

        Some(Expression::PrefixExpression(PrefixExpression {
            token,
            operator,
            right: right.map(Box::new),
        }))
    }

    fn parse_boolean(&mut self) -> Option<Expression> {
        Some(Expression::Boolean(Boolean {
            token: self.cur_token.clone(),
            value: self.cur_token_is(TokenType::True),
        }))
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token(); // consume '('
        let exp = self.parse_expression(LOWEST);
        if !self.expect_peek(TokenType::RParen) {
            return None;
        }
        exp
    }

    fn parse_if_expression(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::LParen) {
            return None;
        }

        self.next_token(); // consume '('
        let condition = self.parse_expression(LOWEST).map(Box::new);

        if !self.expect_peek(TokenType::RParen) {
            return None;
        }

        if !self.expect_peek(TokenType::LBrace) {
            return None;
        }

        let consequence = self.parse_block_statement();

        let mut alternative: Option<BlockStatement> = None;
        if self.peek_token_is(TokenType::Else) {
            self.next_token();
            if !self.expect_peek(TokenType::LBrace) {
                return None;
            }
            alternative = self.parse_block_statement();
        }

        Some(Expression::IfExpression(IfExpression {
            token,
            condition,
            consequence,
            alternative,
        }))
    }

    fn parse_block_statement(&mut self) -> Option<BlockStatement> {
        // current token is '{'
        let token = self.cur_token.clone();

        let mut statements = vec![];

        self.next_token(); // move into block

        while !self.cur_token_is(TokenType::RBrace) && !self.cur_token_is(TokenType::Eof) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        Some(BlockStatement { token, statements })
    }

    fn parse_function_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::LParen) {
            return None;
        }

        let parameters = self.parse_function_parameters();

        if !self.expect_peek(TokenType::LBrace) {
            return None;
        }

        let body = self.parse_block_statement();

        Some(Expression::FunctionLiteral(FunctionLiteral {
            token,
            name: None,
            parameters: parameters.unwrap_or_default(),
            body,
        }))
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<Identifier>> {
        let mut identifiers: Vec<Identifier> = vec![];

        if self.peek_token_is(TokenType::RParen) {
            self.next_token();
            return Some(identifiers);
        }

        self.next_token(); // move onto first parameter
        identifiers.push(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        });

        while self.peek_token_is(TokenType::Comma) {
            self.next_token(); // skip comma
            self.next_token();
            identifiers.push(Identifier {
                token: self.cur_token.clone(),
                value: self.cur_token.literal.clone(),
            });
        }

        if !self.expect_peek(TokenType::RParen) {
            return None;
        }

        Some(identifiers)
    }

    fn parse_string_literal(&mut self) -> Option<Expression> {
        Some(Expression::StringLiteral(StringLiteral {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_array_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let elements = self.parse_expression_list(TokenType::RBracket)?;
        Some(Expression::ArrayLiteral(ArrayLiteral { token, elements }))
    }

    fn parse_expression_list(&mut self, end: TokenType) -> Option<Vec<Expression>> {
        let mut list = vec![];

        if self.peek_token_is(end.clone()) {
            self.next_token();
            return Some(list);
        }

        self.next_token();
        if let Some(expr) = self.parse_expression(LOWEST) {
            list.push(expr);
        }

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();
            if let Some(expr) = self.parse_expression(LOWEST) {
                list.push(expr);
            }
        }

        if !self.expect_peek(end) {
            return None;
        }

        Some(list)
    }

    fn parse_hash_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let mut pairs: Vec<(Expression, Expression)> = vec![];

        while !self.peek_token_is(TokenType::RBrace) && !self.peek_token_is(TokenType::Eof) {
            self.next_token(); // move to key
            let key = match self.parse_expression(LOWEST) {
                Some(k) => k,
                None => return None,
            };

            if !self.expect_peek(TokenType::Colon) {
                return None;
            }

            self.next_token(); // move to value
            let value = match self.parse_expression(LOWEST) {
                Some(v) => v,
                None => return None,
            };
            pairs.push((key, value));

            if !self.peek_token_is(TokenType::RBrace) && !self.expect_peek(TokenType::Comma) {
                return None;
            }
        }

        if !self.expect_peek(TokenType::RBrace) {
            return None;
        }

        Some(Expression::HashLiteral(HashLiteral { token, pairs }))
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();
        let precedence = self.cur_precedence();

        self.next_token(); // move past operator
        let right = self.parse_expression(precedence);

        Some(Expression::InfixExpression(InfixExpression {
            token,
            operator,
            left: Box::new(left),
            right: right.map(Box::new),
        }))
    }

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let arguments = self.parse_expression_list(TokenType::RParen)?;
        Some(Expression::CallExpression(CallExpression {
            token,
            function: Box::new(function),
            arguments,
        }))
    }

    fn parse_index_expression(&mut self, left: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        self.next_token(); // skip '['
        let index = self.parse_expression(LOWEST);
        if !self.expect_peek(TokenType::RBracket) {
            return None;
        }

        Some(Expression::IndexExpression(IndexExpression {
            token,
            left: Box::new(left),
            index: index.map(Box::new),
        }))
    }

    fn register_prefix(&mut self, token_type: TokenType, func: PrefixParseFn) {
        self.prefix_parse_fns.insert(token_type, func);
    }

    fn register_infix(&mut self, token_type: TokenType, func: InfixParseFn) {
        self.infix_parse_fns.insert(token_type, func);
    }
}
