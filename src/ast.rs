pub enum Node {
    Program(Program),
    ExpressionStatement(ExpressionStatement),
    InfixExpression(InfixExpression),
    IntegerLiteral(IntegerLiteral),
}

pub struct Program {
    pub statements: Vec<Node>,
}

pub struct ExpressionStatement {
    pub expression: Box<Node>,
}

pub struct InfixExpression {
    pub left: Box<Node>,
    pub right: Box<Node>,
}

pub struct IntegerLiteral {
    pub value: i64,
}
