pub mod ast;
#[cfg(test)]
mod ast_test;
#[cfg(test)]
mod parser_test;
mod precedences;

pub extern crate lexer;

use crate::ast::{
    Array, BinaryExpression, BlockStatement, Boolean, Expression, FunctionCall,
    FunctionDeclaration, Hash, IDENTIFIER, IF, Index, Integer, Let, Literal, Node, Program,
    ReturnStatement, Statement, StringType, UnaryExpression, While,
};
use crate::precedences::{Precedence, get_token_precedence};
use lexer::Lexer;
use lexer::token::{Span, Token, TokenKind};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    ExpectedToken { expected: String, got: Token },
    ExpectedIdentifier { got: Token },
    InvalidFunctionParameter { got: Token },
    NoPrefixParseFn { token: Token },
    SerializeAst(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::ExpectedToken { expected, got } => {
                write!(f, "expected token {}, got {}", expected, got)
            }
            ParseError::ExpectedIdentifier { got } => {
                write!(f, "expected identifier, got {}", got)
            }
            ParseError::InvalidFunctionParameter { got } => {
                write!(
                    f,
                    "expected function parameter to be an identifier, got {}",
                    got
                )
            }
            ParseError::NoPrefixParseFn { token } => {
                write!(f, "no prefix function for token {}", token)
            }
            ParseError::SerializeAst(err) => write!(f, "failed to serialize AST: {}", err),
        }
    }
}

pub type ParseErrors = Vec<ParseError>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
    errors: ParseErrors,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Parser<'a> {
        let cur = lexer.next_token();
        let next = lexer.next_token();
        let errors = Vec::new();

        Parser {
            lexer,
            current_token: cur,
            peek_token: next,
            errors,
        }
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn current_token_is(&self, token: &TokenKind) -> bool {
        self.current_token.kind == *token
    }

    fn peek_token_is(&self, token: &TokenKind) -> bool {
        self.peek_token.kind == *token
    }

    fn expect_peek(&mut self, token: &TokenKind) -> Result<(), ParseError> {
        if self.peek_token.kind == *token {
            self.next_token();
            Ok(())
        } else {
            Err(ParseError::ExpectedToken {
                expected: token.to_string(),
                got: self.peek_token.clone(),
            })
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseErrors> {
        let mut program = Program::new();
        while !self.current_token_is(&TokenKind::EOF) {
            match self.parse_statement() {
                Ok(stmt) => program.body.push(stmt),
                Err(e) => self.errors.push(e),
            }
            self.next_token();
        }
        program.span.end = self.current_token.span.end;

        if self.errors.is_empty() {
            Ok(program)
        } else {
            Err(self.errors.clone())
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current_token.kind {
            TokenKind::LET => self.parse_let_statement(),
            TokenKind::RETURN => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_token.span.start;
        self.next_token();

        let name = self.current_token.clone();
        let ident_name_str = match &self.current_token.kind {
            TokenKind::IDENTIFIER { name } => Some(name.clone()),
            _ => {
                return Err(ParseError::ExpectedIdentifier {
                    got: self.current_token.clone(),
                });
            }
        };

        self.expect_peek(&TokenKind::ASSIGN)?;
        self.next_token();

        let mut value = self.parse_expression(Precedence::Lowest)?.0;
        if let (Some(ident_name), Expression::FUNCTION(f)) = (ident_name_str, &mut value) {
            f.name = ident_name;
        }

        if self.peek_token_is(&TokenKind::SEMICOLON) {
            self.next_token();
        }

        let end = self.current_token.span.end;

        Ok(Statement::Let(Let {
            identifier: name,
            expr: value,
            span: Span { start, end },
        }))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_token.span.start;
        self.next_token();

        let value = self.parse_expression(Precedence::Lowest)?.0;

        if self.peek_token_is(&TokenKind::SEMICOLON) {
            self.next_token();
        }
        let end = self.current_token.span.end;

        Ok(Statement::Return(ReturnStatement {
            argument: value,
            span: Span { start, end },
        }))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expression(Precedence::Lowest)?.0;
        if self.peek_token_is(&TokenKind::SEMICOLON) {
            self.next_token();
        }

        Ok(Statement::Expr(expr))
    }

    fn parse_expression(
        &mut self,
        precedence: Precedence,
    ) -> Result<(Expression, Span), ParseError> {
        let mut left = self.parse_prefix_expression()?;
        let mut left_start = expression_start(&left);
        while self.peek_token.kind != TokenKind::SEMICOLON
            && precedence < get_token_precedence(&self.peek_token.kind)
        {
            match self.parse_infix_expression(&left, left_start) {
                Some(infix) => {
                    left = infix?;
                    left_start = expression_start(&left);
                }
                None => {
                    return Ok((
                        left,
                        Span {
                            start: left_start,
                            end: self.current_token.span.end,
                        },
                    ));
                }
            }
        }

        let end = self.current_token.span.end;

        Ok((
            left,
            Span {
                start: left_start,
                end,
            },
        ))
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression, ParseError> {
        // this is prefix fn map :)
        match &self.current_token.kind {
            TokenKind::IDENTIFIER { name } => Ok(Expression::IDENTIFIER(IDENTIFIER {
                name: name.clone(),
                span: self.current_token.clone().span,
            })),
            TokenKind::INT(i) => Ok(Expression::LITERAL(Literal::Integer(Integer {
                raw: *i,
                span: self.current_token.clone().span,
            }))),
            TokenKind::STRING(s) => Ok(Expression::LITERAL(Literal::String(StringType {
                raw: s.to_string(),
                span: self.current_token.clone().span,
            }))),
            b @ TokenKind::TRUE | b @ TokenKind::FALSE => {
                Ok(Expression::LITERAL(Literal::Boolean(Boolean {
                    raw: *b == TokenKind::TRUE,
                    span: self.current_token.clone().span,
                })))
            }
            TokenKind::BANG | TokenKind::MINUS => {
                let start = self.current_token.span.start;
                let prefix_op = self.current_token.clone();
                self.next_token();
                let (expr, span) = self.parse_expression(Precedence::Prefix)?;
                Ok(Expression::PREFIX(UnaryExpression {
                    op: prefix_op,
                    operand: Box::new(expr),
                    span: Span {
                        start,
                        end: span.end,
                    },
                }))
            }
            TokenKind::LPAREN => {
                self.next_token();
                let expr = self.parse_expression(Precedence::Lowest)?.0;
                self.expect_peek(&TokenKind::RPAREN)?;
                Ok(expr)
            }
            TokenKind::IF => self.parse_if_expression(),
            TokenKind::WHILE => self.parse_while_expression(),
            TokenKind::FUNCTION => self.parse_fn_expression(),
            TokenKind::LBRACKET => {
                let (elements, span) = self.parse_expression_list(&TokenKind::RBRACKET)?;
                Ok(Expression::LITERAL(Literal::Array(Array {
                    elements,
                    span,
                })))
            }
            TokenKind::LBRACE => self.parse_hash_expression(),
            _ => Err(ParseError::NoPrefixParseFn {
                token: self.current_token.clone(),
            }),
        }
    }

    fn parse_infix_expression(
        &mut self,
        left: &Expression,
        left_start: usize,
    ) -> Option<Result<Expression, ParseError>> {
        match self.peek_token.kind {
            TokenKind::PLUS
            | TokenKind::MINUS
            | TokenKind::ASTERISK
            | TokenKind::SLASH
            | TokenKind::PERCENT
            | TokenKind::EQ
            | TokenKind::NotEq
            | TokenKind::LT
            | TokenKind::GT
            | TokenKind::LTE
            | TokenKind::GTE => {
                self.next_token();
                let infix_op = self.current_token.clone();
                let precedence_value = get_token_precedence(&self.current_token.kind);
                self.next_token();
                let (right, span) = match self.parse_expression(precedence_value) {
                    Ok(parsed) => parsed,
                    Err(err) => return Some(Err(err)),
                };
                Some(Ok(Expression::INFIX(BinaryExpression {
                    op: infix_op,
                    left: Box::new(left.clone()),
                    right: Box::new(right),
                    span: Span {
                        start: left_start,
                        end: span.end,
                    },
                })))
            }
            TokenKind::LPAREN => {
                self.next_token();
                Some(self.parse_fn_call_expression(left.clone()))
            }
            TokenKind::LBRACKET => {
                self.next_token();
                Some(self.parse_index_expression(left.clone()))
            }
            _ => None,
        }
    }

    fn parse_if_expression(&mut self) -> Result<Expression, ParseError> {
        let start = self.current_token.span.start;
        self.expect_peek(&TokenKind::LPAREN)?;
        self.next_token();

        let condition = self.parse_expression(Precedence::Lowest)?.0;
        self.expect_peek(&TokenKind::RPAREN)?;
        self.expect_peek(&TokenKind::LBRACE)?;

        let consequent = self.parse_block_statement()?;

        let alternate = if self.peek_token_is(&TokenKind::ELSE) {
            self.next_token();
            self.expect_peek(&TokenKind::LBRACE)?;
            Some(self.parse_block_statement()?)
        } else {
            None
        };

        let end = self.current_token.span.end;

        Ok(Expression::IF(IF {
            condition: Box::new(condition),
            consequent,
            alternate,
            span: Span { start, end },
        }))
    }

    fn parse_block_statement(&mut self) -> Result<BlockStatement, ParseError> {
        let start = self.current_token.span.start;
        self.next_token();
        let mut block_statement = Vec::new();

        while !self.current_token_is(&TokenKind::RBRACE) && !self.current_token_is(&TokenKind::EOF)
        {
            match self.parse_statement() {
                Ok(statement) => block_statement.push(statement),
                Err(err) => self.errors.push(err),
            }

            self.next_token();
        }

        let end = self.current_token.span.end;

        Ok(BlockStatement {
            body: block_statement,
            span: Span { start, end },
        })
    }

    fn parse_while_expression(&mut self) -> Result<Expression, ParseError> {
        let start = self.current_token.span.start;
        self.expect_peek(&TokenKind::LPAREN)?;
        self.next_token();

        let condition = self.parse_expression(Precedence::Lowest)?.0;
        self.expect_peek(&TokenKind::RPAREN)?;
        self.expect_peek(&TokenKind::LBRACE)?;

        let body = self.parse_block_statement()?;
        let end = self.current_token.span.end;

        Ok(Expression::While(While {
            condition: Box::new(condition),
            body,
            span: Span { start, end },
        }))
    }

    fn parse_fn_expression(&mut self) -> Result<Expression, ParseError> {
        let start = self.current_token.span.start;
        self.expect_peek(&TokenKind::LPAREN)?;

        let params = self.parse_fn_parameters()?;

        self.expect_peek(&TokenKind::LBRACE)?;

        let function_body = self.parse_block_statement()?;

        let end = self.current_token.span.end;

        Ok(Expression::FUNCTION(FunctionDeclaration {
            params,
            body: function_body,
            span: Span { start, end },
            name: "".to_string(),
        }))
    }

    fn parse_fn_parameters(&mut self) -> Result<Vec<IDENTIFIER>, ParseError> {
        let mut params = Vec::new();
        if self.peek_token_is(&TokenKind::RPAREN) {
            self.next_token();
            return Ok(params);
        }

        self.next_token();

        match &self.current_token.kind {
            TokenKind::IDENTIFIER { name } => params.push(IDENTIFIER {
                name: name.clone(),
                span: self.current_token.span.clone(),
            }),
            _ => {
                return Err(ParseError::InvalidFunctionParameter {
                    got: self.current_token.clone(),
                });
            }
        }

        while self.peek_token_is(&TokenKind::COMMA) {
            self.next_token();
            self.next_token();
            match &self.current_token.kind {
                TokenKind::IDENTIFIER { name } => params.push(IDENTIFIER {
                    name: name.clone(),
                    span: self.current_token.span.clone(),
                }),
                _ => {
                    return Err(ParseError::InvalidFunctionParameter {
                        got: self.current_token.clone(),
                    });
                }
            }
        }

        self.expect_peek(&TokenKind::RPAREN)?;

        Ok(params)
    }

    fn parse_fn_call_expression(&mut self, expr: Expression) -> Result<Expression, ParseError> {
        let start = expression_start(&expr);
        let (arguments, ..) = self.parse_expression_list(&TokenKind::RPAREN)?;
        let end = self.current_token.span.end;
        let callee = Box::new(expr);

        Ok(Expression::FunctionCall(FunctionCall {
            callee,
            arguments,
            span: Span { start, end },
        }))
    }

    fn parse_expression_list(
        &mut self,
        end: &TokenKind,
    ) -> Result<(Vec<Expression>, Span), ParseError> {
        let start = self.current_token.span.start;
        let mut expr_list = Vec::new();
        if self.peek_token_is(end) {
            self.next_token();
            let end = self.current_token.span.end;
            return Ok((expr_list, Span { start, end }));
        }

        self.next_token();

        expr_list.push(self.parse_expression(Precedence::Lowest)?.0);

        while self.peek_token_is(&TokenKind::COMMA) {
            self.next_token();
            self.next_token();
            expr_list.push(self.parse_expression(Precedence::Lowest)?.0);
        }

        self.expect_peek(end)?;
        let end = self.current_token.span.end;

        Ok((expr_list, Span { start, end }))
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression, ParseError> {
        let start = self.current_token.span.start;
        self.next_token();
        let index = self.parse_expression(Precedence::Lowest)?.0;

        self.expect_peek(&TokenKind::RBRACKET)?;

        let end = self.current_token.span.end;

        Ok(Expression::Index(Index {
            object: Box::new(left),
            index: Box::new(index),
            span: Span { start, end },
        }))
    }

    fn parse_hash_expression(&mut self) -> Result<Expression, ParseError> {
        let mut map = Vec::new();
        let start = self.current_token.span.start;
        while !self.peek_token_is(&TokenKind::RBRACE) {
            self.next_token();

            let key = self.parse_expression(Precedence::Lowest)?.0;

            self.expect_peek(&TokenKind::COLON)?;

            self.next_token();
            let value = self.parse_expression(Precedence::Lowest)?.0;

            map.push((key, value));

            if !self.peek_token_is(&TokenKind::RBRACE) {
                self.expect_peek(&TokenKind::COMMA)?;
            }
        }

        self.expect_peek(&TokenKind::RBRACE)?;
        let end = self.current_token.span.end;

        Ok(Expression::LITERAL(Literal::Hash(Hash {
            elements: map,
            span: Span { start, end },
        })))
    }
}

pub fn parse(input: &str) -> Result<Node, ParseErrors> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program()?;

    Ok(Node::Program(program))
}

pub fn parse_ast_json_string(input: &str) -> Result<String, ParseErrors> {
    let node = parse(input)?;
    serde_json::to_string_pretty(&node)
        .map_err(|err| vec![ParseError::SerializeAst(err.to_string())])
}

fn expression_start(expr: &Expression) -> usize {
    match expr {
        Expression::IDENTIFIER(identifier) => identifier.span.start,
        Expression::LITERAL(literal) => literal_start(literal),
        Expression::PREFIX(prefix) => prefix.span.start,
        Expression::INFIX(infix) => infix.span.start,
        Expression::IF(if_expr) => if_expr.span.start,
        Expression::While(while_expr) => while_expr.span.start,
        Expression::FUNCTION(function) => function.span.start,
        Expression::FunctionCall(call) => call.span.start,
        Expression::Index(index) => index.span.start,
    }
}

fn literal_start(literal: &Literal) -> usize {
    match literal {
        Literal::Integer(integer) => integer.span.start,
        Literal::Boolean(boolean) => boolean.span.start,
        Literal::String(string) => string.span.start,
        Literal::Array(array) => array.span.start,
        Literal::Hash(hash) => hash.span.start,
    }
}
