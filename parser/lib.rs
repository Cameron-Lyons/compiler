pub mod ast;
mod ast_test;
mod parser_test;
mod precedences;

pub extern crate lexer;

use crate::ast::{
    Array, BinaryExpression, BlockStatement, Boolean, Expression, FunctionCall,
    FunctionDeclaration, Hash, Index, Integer, Let, Literal, Node, Program, ReturnStatement,
    Statement, StringType, UnaryExpression, IDENTIFIER, IF,
};
use crate::precedences::{get_token_precedence, Precedence};
use lexer::token::{Span, Token, TokenKind};
use lexer::Lexer;

type ParseError = String;
type ParseErrors = Vec<ParseError>;

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
        let p = Parser {
            lexer,
            current_token: cur,
            peek_token: next,
            errors,
        };

        return p;
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn current_token_is(&mut self, token: &TokenKind) -> bool {
        self.current_token.kind == *token
    }

    fn peek_token_is(&mut self, token: &TokenKind) -> bool {
        self.peek_token.kind == *token
    }

    fn expect_peek(&mut self, token: &TokenKind) -> Result<(), ParseError> {
        self.next_token();
        if self.current_token.kind == *token {
            Ok(())
        } else {
            let e = format!("expected token: {} got: {}", token, self.current_token);
            Err(e)
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
            return Ok(program);
        } else {
            return Err(self.errors.clone());
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
        let mut ident_name_str = None;
        match &self.current_token.kind {
            TokenKind::IDENTIFIER { name } => {
                ident_name_str = Some(name.clone());
            }
            _ => return Err(format!("{} not an identifier", self.current_token)),
        };

        self.expect_peek(&TokenKind::ASSIGN)?;
        self.next_token();

        let mut value = self.parse_expression(Precedence::LOWEST)?.0;
        if let (Some(ident_name), Expression::FUNCTION(ref mut f)) = (ident_name_str, &mut value) {
            f.name = ident_name;
        }

        if self.peek_token_is(&TokenKind::SEMICOLON) {
            self.next_token();
        }

        let end = self.current_token.span.end;

        return Ok(Statement::Let(Let {
            identifier: name,
            expr: value,
            span: Span { start, end },
        }));
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_token.span.start;
        self.next_token();

        let value = self.parse_expression(Precedence::LOWEST)?.0;

        if self.peek_token_is(&TokenKind::SEMICOLON) {
            self.next_token();
        }
        let end = self.current_token.span.end;

        return Ok(Statement::Return(ReturnStatement {
            argument: value,
            span: Span { start, end },
        }));
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expression(Precedence::LOWEST)?.0;
        if self.peek_token_is(&TokenKind::SEMICOLON) {
            self.next_token();
        }

        Ok(Statement::Expr(expr))
    }

    fn parse_expression(
        &mut self,
        precedence: Precedence,
    ) -> Result<(Expression, Span), ParseError> {
        let mut left_start = self.current_token.span.start;
        let mut left = self.parse_prefix_expression()?;
        while self.peek_token.kind != TokenKind::SEMICOLON
            && precedence < get_token_precedence(&self.peek_token.kind)
        {
            match self.parse_infix_expression(&left, left_start) {
                Some(infix) => {
                    left = infix?;
                    if let Expression::INFIX(b) = left.clone() {
                        left_start = b.span.start;
                    }
                }
                None => {
                    return Ok((
                        left,
                        Span {
                            start: left_start,
                            end: self.current_token.span.end,
                        },
                    ))
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
            TokenKind::IDENTIFIER { name } => {
                return Ok(Expression::IDENTIFIER(IDENTIFIER {
                    name: name.clone(),
                    span: self.current_token.clone().span,
                }))
            }
            TokenKind::INT(i) => {
                return Ok(Expression::LITERAL(Literal::Integer(Integer {
                    raw: *i,
                    span: self.current_token.clone().span,
                })))
            }
            TokenKind::STRING(s) => {
                return Ok(Expression::LITERAL(Literal::String(StringType {
                    raw: s.to_string(),
                    span: self.current_token.clone().span,
                })))
            }
            b @ TokenKind::TRUE | b @ TokenKind::FALSE => {
                return Ok(Expression::LITERAL(Literal::Boolean(Boolean {
                    raw: *b == TokenKind::TRUE,
                    span: self.current_token.clone().span,
                })))
            }
            TokenKind::BANG | TokenKind::MINUS => {
                let start = self.current_token.span.start;
                let prefix_op = self.current_token.clone();
                self.next_token();
                let (expr, span) = self.parse_expression(Precedence::PREFIX)?;
                return Ok(Expression::PREFIX(UnaryExpression {
                    op: prefix_op,
                    operand: Box::new(expr),
                    span: Span {
                        start,
                        end: span.end,
                    },
                }));
            }
            TokenKind::LPAREN => {
                self.next_token();
                let expr = self.parse_expression(Precedence::LOWEST)?.0;
                self.expect_peek(&TokenKind::RPAREN)?;
                return Ok(expr);
            }
            TokenKind::IF => self.parse_if_expression(),
            TokenKind::FUNCTION => self.parse_fn_expression(),
            TokenKind::LBRACKET => {
                let (elements, span) = self.parse_expression_list(&TokenKind::RBRACKET)?;
                return Ok(Expression::LITERAL(Literal::Array(Array {
                    elements,
                    span,
                })));
            }
            TokenKind::LBRACE => self.parse_hash_expression(),
            _ => Err(format!(
                "no prefix function for token: {}",
                self.current_token
            )),
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
            | TokenKind::EQ
            | TokenKind::NotEq
            | TokenKind::LT
            | TokenKind::GT => {
                self.next_token();
                let infix_op = self.current_token.clone();
                let precedence_value = get_token_precedence(&self.current_token.kind);
                self.next_token();
                let (right, span) = self.parse_expression(precedence_value).unwrap();
                return Some(Ok(Expression::INFIX(BinaryExpression {
                    op: infix_op,
                    left: Box::new(left.clone()),
                    right: Box::new(right),
                    span: Span {
                        start: left_start,
                        end: span.end,
                    },
                })));
            }
            TokenKind::LPAREN => {
                self.next_token();
                return Some(self.parse_fn_call_expression(left.clone()));
            }
            TokenKind::LBRACKET => {
                self.next_token();
                return Some(self.parse_index_expression(left.clone()));
            }
            _ => None,
        }
    }

    fn parse_if_expression(&mut self) -> Result<Expression, ParseError> {
        let start = self.current_token.span.start;
        self.expect_peek(&TokenKind::LPAREN)?;
        self.next_token();

        let condition = self.parse_expression(Precedence::LOWEST)?.0;
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

        return Ok(Expression::IF(IF {
            condition: Box::new(condition),
            consequent,
            alternate,
            span: Span { start, end },
        }));
    }

    fn parse_block_statement(&mut self) -> Result<BlockStatement, ParseError> {
        let start = self.current_token.span.start;
        self.next_token();
        let mut block_statement = Vec::new();

        while !self.current_token_is(&TokenKind::RBRACE) && !self.current_token_is(&TokenKind::EOF)
        {
            if let Ok(statement) = self.parse_statement() {
                block_statement.push(statement)
            }

            self.next_token();
        }

        let end = self.current_token.span.end;

        Ok(BlockStatement {
            body: block_statement,
            span: Span { start, end },
        })
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
            token => {
                return Err(format!(
                    "expected function params  to be an identifier, got {}",
                    token
                ))
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
                token => {
                    return Err(format!(
                        "expected function params  to be an identifier, got {}",
                        token
                    ))
                }
            }
        }

        self.expect_peek(&TokenKind::RPAREN)?;

        return Ok(params);
    }

    fn parse_fn_call_expression(&mut self, expr: Expression) -> Result<Expression, ParseError> {
        // fake positive
        #[allow(unused_assignments)]
        let mut start = self.current_token.span.start;
        let (arguments, ..) = self.parse_expression_list(&TokenKind::RPAREN)?;
        let end = self.current_token.span.end;
        match &expr {
            Expression::IDENTIFIER(i) => start = i.span.start,
            Expression::FUNCTION(f) => start = f.span.start,
            _ => return Err(format!("expected function")),
        }
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

        expr_list.push(self.parse_expression(Precedence::LOWEST)?.0);

        while self.peek_token_is(&TokenKind::COMMA) {
            self.next_token();
            self.next_token();
            expr_list.push(self.parse_expression(Precedence::LOWEST)?.0);
        }

        self.expect_peek(end)?;
        let end = self.current_token.span.end;

        return Ok((expr_list, Span { start, end }));
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression, ParseError> {
        let start = self.current_token.span.start;
        self.next_token();
        let index = self.parse_expression(Precedence::LOWEST)?.0;

        self.expect_peek(&TokenKind::RBRACKET)?;

        let end = self.current_token.span.end;

        return Ok(Expression::Index(Index {
            object: Box::new(left),
            index: Box::new(index),
            span: Span { start, end },
        }));
    }

    fn parse_hash_expression(&mut self) -> Result<Expression, ParseError> {
        let mut map = Vec::new();
        let start = self.current_token.span.start;
        while !self.peek_token_is(&TokenKind::RBRACE) {
            self.next_token();

            let key = self.parse_expression(Precedence::LOWEST)?.0;

            self.expect_peek(&TokenKind::COLON)?;

            self.next_token();
            let value = self.parse_expression(Precedence::LOWEST)?.0;

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
    let ast = match parse(input) {
        Ok(node) => serde_json::to_string_pretty(&node).unwrap(),
        Err(e) => return Err(e),
    };

    return Ok(ast);
}
