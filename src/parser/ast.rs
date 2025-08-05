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
    Function(FunctionDef),
    Class(ClassDef),
    Struct(StructDef),
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
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub default_value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub methods: Vec<FunctionDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_annotation: TypeAnnotation,
    pub default_value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    Simple(String),
    Generic {
        base: String,
        params: Vec<TypeAnnotation>,
    },
    Array(Box<TypeAnnotation>),
    Function {
        params: Vec<TypeAnnotation>,
        return_type: Box<TypeAnnotation>,
    },
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
    For {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Let {
        name: String,
        type_annotation: Option<TypeAnnotation>,
        value: Option<Expression>,
    },
    Return(Option<Expression>),
    Break,
    Continue,
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
    BiDirectionalPipe {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    StreamBranch {
        stream: Box<Expression>,
        count: u8,
    },
    StreamMerge {
        streams: Vec<Expression>,
        output_name: String,
    },
    UnitValue {
        value: Box<Expression>,
        unit: String,
    },
    ArrayLiteral(Vec<Expression>),
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },
    Lambda {
        parameters: Vec<String>,
        body: Box<Expression>,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
        named_args: HashMap<String, Expression>,
    },
    InterpolatedString(Vec<StringPart>),
    ConditionalExpression {
        condition: Box<Expression>,
        true_expr: Box<Expression>,
        false_expr: Box<Expression>,
    },
    MatchExpression {
        expr: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    TypeCast {
        expr: Box<Expression>,
        target_type: TypeAnnotation,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Text(String),
    Interpolation(Expression),
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
    LogicalAnd,
    LogicalOr,
    Pipe,
    BiDirectionalPipe,
}