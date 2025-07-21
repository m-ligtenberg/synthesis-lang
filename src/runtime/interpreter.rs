use crate::parser::ast::*;
use crate::runtime::{StreamManager, Value};
use std::collections::HashMap;

pub struct Interpreter {
    pub variables: HashMap<String, Value>,
    pub stream_manager: StreamManager,
    pub modules: HashMap<String, Module>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub functions: HashMap<String, ModuleFunction>,
}

#[derive(Debug, Clone)]
pub struct ModuleFunction {
    pub name: String,
    pub callback: fn(&[Value]) -> crate::Result<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            variables: HashMap::new(),
            stream_manager: StreamManager::new(),
            modules: HashMap::new(),
        };
        
        interpreter.register_builtin_modules();
        interpreter
    }
    
    pub fn execute(&mut self, program: &Program) -> crate::Result<()> {
        for item in &program.items {
            match item {
                Item::Import(import) => self.execute_import(import)?,
                Item::Statement(stmt) => {
                    self.execute_statement(stmt)?;
                }
                Item::Loop(loop_block) => {
                    loop {
                        for stmt in &loop_block.body {
                            self.execute_statement(stmt)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    fn execute_import(&mut self, _import: &ImportItem) -> crate::Result<()> {
        Ok(())
    }
    
    fn execute_statement(&mut self, stmt: &Statement) -> crate::Result<Value> {
        match stmt {
            Statement::Assignment { name, value } => {
                let val = self.evaluate_expression(value)?;
                self.variables.insert(name.clone(), val.clone());
                Ok(val)
            }
            Statement::Expression(expr) => self.evaluate_expression(expr),
            Statement::If { condition, then_branch, else_branch } => {
                let cond_value = self.evaluate_expression(condition)?;
                
                if cond_value.is_truthy() {
                    for stmt in then_branch {
                        self.execute_statement(stmt)?;
                    }
                } else if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.execute_statement(stmt)?;
                    }
                }
                
                Ok(Value::Null)
            }
        }
    }
    
    fn evaluate_expression(&mut self, expr: &Expression) -> crate::Result<Value> {
        match expr {
            Expression::Literal(lit) => Ok(self.evaluate_literal(lit)),
            Expression::Identifier(name) => {
                self.variables.get(name)
                    .cloned()
                    .or_else(|| Some(self.stream_manager.get_stream_value(name)))
                    .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", name))
            }
            Expression::FunctionCall { module, name, args } => {
                self.evaluate_function_call(module.as_ref(), name, args)
            }
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.evaluate_binary_op(&left_val, op, &right_val)
            }
            Expression::Block { fields } => {
                let mut object = HashMap::new();
                for (key, value_expr) in fields {
                    let value = self.evaluate_expression(value_expr)?;
                    object.insert(key.clone(), value);
                }
                Ok(Value::Object(object))
            }
        }
    }
    
    fn evaluate_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Integer(n) => Value::Integer(*n),
            Literal::Float(f) => Value::Float(*f),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Boolean(b) => Value::Boolean(*b),
        }
    }
    
    fn evaluate_function_call(
        &mut self,
        module: Option<&String>,
        name: &str,
        args: &[Expression],
    ) -> crate::Result<Value> {
        let arg_values: Result<Vec<_>, _> = args.iter()
            .map(|arg| self.evaluate_expression(arg))
            .collect();
        let arg_values = arg_values?;
        
        if let Some(module_name) = module {
            if let Some(module) = self.modules.get(module_name) {
                if let Some(function) = module.functions.get(name) {
                    return (function.callback)(&arg_values);
                }
            }
            return Err(anyhow::anyhow!("Function {}.{} not found", module_name, name));
        }
        
        Err(anyhow::anyhow!("Function {} not found", name))
    }
    
    fn evaluate_binary_op(
        &self,
        left: &Value,
        op: &BinaryOperator,
        right: &Value,
    ) -> crate::Result<Value> {
        match op {
            BinaryOperator::Add => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Err(anyhow::anyhow!("Cannot add {:?} and {:?}", left.type_name(), right.type_name())),
            },
            BinaryOperator::Subtract => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err(anyhow::anyhow!("Cannot subtract {:?} and {:?}", left.type_name(), right.type_name())),
            },
            BinaryOperator::Multiply => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
                _ => Err(anyhow::anyhow!("Cannot multiply {:?} and {:?}", left.type_name(), right.type_name())),
            },
            BinaryOperator::Divide => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if *b == 0 {
                        return Err(anyhow::anyhow!("Division by zero"));
                    }
                    Ok(Value::Float(*a as f64 / *b as f64))
                },
                (Value::Float(a), Value::Float(b)) => {
                    if *b == 0.0 {
                        return Err(anyhow::anyhow!("Division by zero"));
                    }
                    Ok(Value::Float(a / b))
                },
                (Value::Integer(a), Value::Float(b)) => {
                    if *b == 0.0 {
                        return Err(anyhow::anyhow!("Division by zero"));
                    }
                    Ok(Value::Float(*a as f64 / b))
                },
                (Value::Float(a), Value::Integer(b)) => {
                    if *b == 0 {
                        return Err(anyhow::anyhow!("Division by zero"));
                    }
                    Ok(Value::Float(a / *b as f64))
                },
                _ => Err(anyhow::anyhow!("Cannot divide {:?} and {:?}", left.type_name(), right.type_name())),
            },
            BinaryOperator::Equal => Ok(Value::Boolean(self.values_equal(left, right))),
            BinaryOperator::NotEqual => Ok(Value::Boolean(!self.values_equal(left, right))),
            BinaryOperator::LessThan => {
                if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
                    Ok(Value::Boolean(a < b))
                } else {
                    Err(anyhow::anyhow!("Cannot compare {:?} and {:?}", left.type_name(), right.type_name()))
                }
            },
            BinaryOperator::LessThanOrEqual => {
                if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
                    Ok(Value::Boolean(a <= b))
                } else {
                    Err(anyhow::anyhow!("Cannot compare {:?} and {:?}", left.type_name(), right.type_name()))
                }
            },
            BinaryOperator::GreaterThan => {
                if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
                    Ok(Value::Boolean(a > b))
                } else {
                    Err(anyhow::anyhow!("Cannot compare {:?} and {:?}", left.type_name(), right.type_name()))
                }
            },
            BinaryOperator::GreaterThanOrEqual => {
                if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
                    Ok(Value::Boolean(a >= b))
                } else {
                    Err(anyhow::anyhow!("Cannot compare {:?} and {:?}", left.type_name(), right.type_name()))
                }
            },
        }
    }
    
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Integer(a), Value::Float(b)) => (*a as f64 - b).abs() < f64::EPSILON,
            (Value::Float(a), Value::Integer(b)) => (a - *b as f64).abs() < f64::EPSILON,
            _ => false,
        }
    }
    
    fn register_builtin_modules(&mut self) {
        let mut graphics_module = Module {
            name: "Graphics".to_string(),
            functions: HashMap::new(),
        };
        
        graphics_module.functions.insert(
            "clear".to_string(),
            ModuleFunction {
                name: "clear".to_string(),
                callback: |_args| Ok(Value::Null),
            }
        );
        
        self.modules.insert("Graphics".to_string(), graphics_module);
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}