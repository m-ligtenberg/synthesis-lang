use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Import(ImportItem),
    Statement(Statement),
    Loop(LoopBlock),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportItem {
    pub module: String,
    pub items: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopBlock {
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment {
        name: String,
        value: Expression,
    },
    Expression(Expression),
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    Match {
        expression: Expression,
        arms: Vec<MatchArm>,
    },
    Every {
        duration: Expression,
        body: Vec<Statement>,
    },
    After {
        duration: Expression,
        body: Vec<Statement>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    Enum {
        name: String,
        fields: Option<Vec<Pattern>>,
    },
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    FunctionCall {
        module: Option<String>,
        name: String,
        args: Vec<Expression>,
        named_args: HashMap<String, Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    Block {
        fields: HashMap<String, Expression>,
    },
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    Pipe {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnitValue {
        value: Box<Expression>,
        unit: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Pipe,
}