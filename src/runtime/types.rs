use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Stream(Stream),
    Function(Function),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Stream(stream) => write!(f, "Stream<{}>", stream.name),
            Value::Function(func) => write!(f, "Function<{}>", func.name),
            Value::Object(obj) => {
                write!(f, "{{")?;
                for (key, value) in obj {
                    write!(f, "{}: {}, ", key, value)?;
                }
                write!(f, "}}")
            }
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, value) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
            }
            Value::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    pub name: String,
    pub data_type: DataType,
    pub sample_rate: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Audio,
    Visual,
    Control,
    MIDI,
    Generic,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: crate::parser::ast::Statement,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Stream(_) => "stream",
            Value::Function(_) => "function",
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::Null => "null",
        }
    }
    
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Integer(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Null => false,
            _ => true,
        }
    }
    
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Integer(n) => Some(*n as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }
}