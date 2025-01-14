use std::fmt::Write;

use crate::token::Token;

const SEMICOLON: &str = ";";
const COMMA: &str = ",";
const COLON: &str = ":";
const LEFT_PAREN: &str = "(";
const RIGHT_PAREN: &str = ")";
const LEFT_BRACKET: &str = "[";
const RIGHT_BRACKET: &str = "]";
const LEFT_BRACE: &str = "{";
const RIGHT_BRACE: &str = "}";

pub trait Node {
    fn token_literal(&self) -> String;
    fn to_string(&self) -> String;
}

pub trait Statement: Node {
    fn statement_node(&self);
}

pub trait Expression: Node {
    fn expression_node(&self);
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Box<dyn Statement>>,
}

impl Node for Program {
    fn token_literal(&self) -> String {
        if !self.statements.is_empty() {
            self.statements[0].token_literal()
        } else {
            "".to_string()
        }
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        for stmt in &self.statements {
            out.push_str(&stmt.to_string());
        }
        out
    }
}

impl Program {
    pub fn new() -> Self {
        Program { statements: vec![] }
    }
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub token: Token, // the '{' token
    pub statements: Vec<Box<dyn Statement>>,
}

impl Statement for BlockStatement {
    fn statement_node(&self) {}
}

impl Node for BlockStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        for stmt in &self.statements {
            out.push_str(&stmt.to_string());
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub token: Token,          // the token.Let token
    pub name: Box<Identifier>, // e.g. `let x = ...`
    pub value: Option<Box<dyn Expression>>,
}

impl Statement for LetStatement {
    fn statement_node(&self) {}
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        // e.g. "let x = 5;"
        write!(out, "{} ", self.token_literal()).unwrap();
        out.push_str(&self.name.to_string());
        out.push_str(" = ");

        if let Some(val) = &self.value {
            out.push_str(&val.to_string());
        }

        out.push_str(SEMICOLON);
        out
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub token: Token, // the 'return' token
    pub return_value: Option<Box<dyn Expression>>,
}

impl Statement for ReturnStatement {
    fn statement_node(&self) {}
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        // e.g. "return 5;"
        write!(out, "{} ", self.token_literal()).unwrap();

        if let Some(val) = &self.return_value {
            out.push_str(&val.to_string());
        }

        out.push_str(SEMICOLON);
        out
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub token: Token, // the first token of the expression
    pub expression: Option<Box<dyn Expression>>,
}

impl Statement for ExpressionStatement {
    fn statement_node(&self) {}
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        match &self.expression {
            Some(expr) => expr.to_string(),
            None => "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub token: Token, // the token.Ident token
    pub value: String,
}

impl Expression for Identifier {
    fn expression_node(&self) {}
}

impl Node for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        self.value.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Boolean {
    pub token: Token,
    pub value: bool,
}

impl Expression for Boolean {
    fn expression_node(&self) {}
}

impl Node for Boolean {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        self.token.literal.clone()
    }
}

#[derive(Debug, Clone)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}

impl Expression for IntegerLiteral {
    fn expression_node(&self) {}
}

impl Node for IntegerLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        self.token.literal.clone()
    }
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub token: Token,
    pub value: String,
}

impl Expression for StringLiteral {
    fn expression_node(&self) {}
}

impl Node for StringLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        self.token.literal.clone()
    }
}

#[derive(Debug, Clone)]
pub struct PrefixExpression {
    pub token: Token, // The prefix token, e.g. !
    pub operator: String,
    pub right: Option<Box<dyn Expression>>,
}

impl Expression for PrefixExpression {
    fn expression_node(&self) {}
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        // e.g. "(!5)"
        out.push('(');
        out.push_str(&self.operator);
        if let Some(r) = &self.right {
            out.push_str(&r.to_string());
        }
        out.push(')');
        out
    }
}

#[derive(Debug, Clone)]
pub struct InfixExpression {
    pub token: Token, // The operator token, e.g. +
    pub left: Option<Box<dyn Expression>>,
    pub operator: String,
    pub right: Option<Box<dyn Expression>>,
}

impl Expression for InfixExpression {
    fn expression_node(&self) {}
}

impl Node for InfixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        // e.g. "(5 + 10)"
        out.push('(');
        if let Some(l) = &self.left {
            out.push_str(&l.to_string());
        }
        out.push(' ');
        out.push_str(&self.operator);
        out.push(' ');
        if let Some(r) = &self.right {
            out.push_str(&r.to_string());
        }
        out.push(')');
        out
    }
}

#[derive(Debug, Clone)]
pub struct IfExpression {
    pub token: Token, // The 'if' token
    pub condition: Option<Box<dyn Expression>>,
    pub consequence: Option<BlockStatement>,
    pub alternative: Option<BlockStatement>,
}

impl Expression for IfExpression {
    fn expression_node(&self) {}
}

impl Node for IfExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();

        out.push_str("if");
        if let Some(cond) = &self.condition {
            out.push_str(&cond.to_string());
        }
        out.push(' ');

        if let Some(cons) = &self.consequence {
            out.push_str(&cons.to_string());
        }

        if let Some(alt) = &self.alternative {
            out.push_str("else ");
            out.push_str(&alt.to_string());
        }

        out
    }
}

#[derive(Debug, Clone)]
pub struct FunctionLiteral {
    pub token: Token, // The 'fn' token
    pub parameters: Vec<Box<Identifier>>,
    pub body: Option<BlockStatement>,
}

impl Expression for FunctionLiteral {
    fn expression_node(&self) {}
}

impl Node for FunctionLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();

        // e.g. "fn(x, y) { ... }"
        out.push_str(&self.token_literal());
        out.push('(');

        let params: Vec<String> = self.parameters.iter().map(|p| p.to_string()).collect();

        out.push_str(&params.join(&format!("{} ", COMMA)));
        out.push(')');

        if let Some(b) = &self.body {
            out.push_str(&b.to_string());
        }

        out
    }
}

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub token: Token,                          // The '(' token
    pub function: Option<Box<dyn Expression>>, // Identifier or FunctionLiteral
    pub arguments: Vec<Box<dyn Expression>>,
}

impl Expression for CallExpression {
    fn expression_node(&self) {}
}

impl Node for CallExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();

        if let Some(func) = &self.function {
            out.push_str(&func.to_string());
        }
        out.push_str(LEFT_PAREN);

        let args: Vec<String> = self.arguments.iter().map(|a| a.to_string()).collect();

        out.push_str(&args.join(&format!("{} ", COMMA)));
        out.push_str(RIGHT_PAREN);

        out
    }
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub token: Token, // the '[' token
    pub elements: Vec<Box<dyn Expression>>,
}

impl Expression for ArrayLiteral {
    fn expression_node(&self) {}
}

impl Node for ArrayLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str(LEFT_BRACKET);

        let elems: Vec<String> = self.elements.iter().map(|e| e.to_string()).collect();

        out.push_str(&elems.join(&format!("{} ", COMMA)));
        out.push_str(RIGHT_BRACKET);
        out
    }
}

#[derive(Debug, Clone)]
pub struct IndexExpression {
    pub token: Token, // The '[' token
    pub left: Option<Box<dyn Expression>>,
    pub index: Option<Box<dyn Expression>>,
}

impl Expression for IndexExpression {
    fn expression_node(&self) {}
}

impl Node for IndexExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();

        // e.g. "(myArray[1])"
        out.push('(');
        if let Some(l) = &self.left {
            out.push_str(&l.to_string());
        }
        out.push_str(LEFT_BRACKET);
        if let Some(i) = &self.index {
            out.push_str(&i.to_string());
        }
        out.push_str(RIGHT_BRACKET);
        out.push(')');
        out
    }
}

#[derive(Debug, Clone)]
pub struct HashLiteral {
    pub token: Token, // The '{' token
    pub pairs: Vec<(Box<dyn Expression>, Box<dyn Expression>)>,
}

impl Expression for HashLiteral {
    fn expression_node(&self) {}
}

impl Node for HashLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();

        // e.g. "{key1: val1, key2: val2}"
        out.push_str(LEFT_BRACE);

        let mut pair_strings = vec![];
        for (key, value) in &self.pairs {
            let s = format!("{}{}{}", key.to_string(), COLON, value.to_string());
            pair_strings.push(s);
        }

        out.push_str(&pair_strings.join(&format!("{} ", COMMA)));
        out.push_str(RIGHT_BRACE);
        out
    }
}
