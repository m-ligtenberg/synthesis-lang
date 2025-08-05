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
                Item::Function(_func_def) => {
                    // TODO: Implement function definition handling
                    // For now, skip function definitions in the interpreter
                }
                Item::Class(_class_def) => {
                    // TODO: Implement class definition handling  
                    // For now, skip class definitions in the interpreter
                }
                Item::Struct(_struct_def) => {
                    // TODO: Implement struct definition handling
                    // For now, skip struct definitions in the interpreter
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
            Statement::Match { expression, arms } => {
                let expr_value = self.evaluate_expression(expression)?;
                
                for arm in arms {
                    if self.pattern_matches(&arm.pattern, &expr_value)? {
                        for stmt in &arm.body {
                            self.execute_statement(stmt)?;
                        }
                        break;
                    }
                }
                
                Ok(Value::Null)
            }
            Statement::Every { duration, body } => {
                // For now, just execute once - full temporal logic would need runtime support
                let _duration_val = self.evaluate_expression(duration)?;
                for stmt in body {
                    self.execute_statement(stmt)?;
                }
                Ok(Value::Null)
            }
            Statement::After { duration, body } => {
                // For now, just execute once - full temporal logic would need runtime support
                let _duration_val = self.evaluate_expression(duration)?;
                for stmt in body {
                    self.execute_statement(stmt)?;
                }
                Ok(Value::Null)
            }
            Statement::While { condition, body } => {
                while self.evaluate_expression(condition)?.is_truthy() {
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                }
                Ok(Value::Null)
            }
            Statement::For { variable: _variable, iterable: _iterable, body: _body } => {
                // TODO: Implement for-in loops
                // For now, return null
                Ok(Value::Null)
            }
            Statement::Let { name, type_annotation: _type_annotation, value } => {
                // Variable declaration with optional initialization
                let val = if let Some(expr) = value {
                    self.evaluate_expression(expr)?
                } else {
                    Value::Null
                };
                self.variables.insert(name.clone(), val.clone());
                Ok(val)
            }
            Statement::Return(expr) => {
                // TODO: Implement proper return handling with control flow
                if let Some(e) = expr {
                    self.evaluate_expression(e)
                } else {
                    Ok(Value::Null)
                }
            }
            Statement::Break => {
                // TODO: Implement proper break handling with control flow
                Ok(Value::Null)
            }
            Statement::Continue => {
                // TODO: Implement proper continue handling with control flow
                Ok(Value::Null)
            }
        }
    }
    
    fn evaluate_expression(&mut self, expr: &Expression) -> crate::Result<Value> {
        match expr {
            Expression::Literal(lit) => Ok(self.evaluate_literal(lit)),
            Expression::Identifier(name) => {
                // Check for module constants like Graphics.black
                if name.contains('.') {
                    let parts: Vec<&str> = name.split('.').collect();
                    if parts.len() == 2 {
                        let module_name = parts[0];
                        let constant_name = parts[1];
                        
                        match (module_name, constant_name) {
                            ("Graphics", "black") => return Ok(Value::Integer(0x000000)),
                            ("Graphics", "white") => return Ok(Value::Integer(0xFFFFFF)),
                            ("Graphics", "neon") => return Ok(Value::String("neon".to_string())),
                            _ => {}
                        }
                    }
                }
                
                Ok(self.variables.get(name)
                    .cloned()
                    .or_else(|| Some(self.stream_manager.get_stream_value(name)))
                    .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", name))?)
            }
            Expression::FunctionCall { module, name, args, named_args } => {
                self.evaluate_function_call(module.as_ref(), name, args, named_args)
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
            Expression::ArrayAccess { array, index } => {
                let array_val = self.evaluate_expression(array)?;
                let index_val = self.evaluate_expression(index)?;
                
                match (&array_val, &index_val) {
                    (Value::Array(arr), Value::Integer(i)) => {
                        let idx = *i as usize;
                        if idx < arr.len() {
                            Ok(arr[idx].clone())
                        } else {
                            Err(anyhow::anyhow!("Array index {} out of bounds (length {})", idx, arr.len()).into())
                        }
                    }
                    _ => Err(anyhow::anyhow!("Cannot index {:?} with {:?}", array_val.type_name(), index_val.type_name()).into())
                }
            }
            Expression::Pipe { left, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                
                // Enhanced pipe logic for stream processing
                match (&left_val, &right_val) {
                    (Value::Stream(stream), _) => {
                        // Apply processing to stream
                        println!("Piping stream '{}' through operation", stream.name);
                        Ok(right_val)
                    }
                    _ => Ok(right_val),
                }
            }
            Expression::BiDirectionalPipe { left, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                
                // Create bidirectional connection between streams
                match (&left_val, &right_val) {
                    (Value::Stream(left_stream), Value::Stream(right_stream)) => {
                        println!("Creating bidirectional connection: '{}' <> '{}'", 
                                left_stream.name, right_stream.name);
                        
                        // Connect both directions - this needs to be moved to a helper method
                        // since we can't mutably borrow stream_manager in evaluate_expression
                        println!("Would connect: {} <-> {}", left_stream.name, right_stream.name);
                        
                        Ok(left_val)
                    }
                    _ => Err(anyhow::anyhow!("Bidirectional pipe requires two streams").into())
                }
            }
            Expression::StreamBranch { stream, count } => {
                let stream_val = self.evaluate_expression(stream)?;
                
                match stream_val {
                    Value::Stream(stream) => {
                        println!("Branching stream '{}' into {} outputs", stream.name, count);
                        
                        // Create branch streams - placeholder for now
                        for i in 0..*count {
                            let branch_name = format!("{}_branch_{}", stream.name, i + 1);
                            println!("Would create branch stream: {}", branch_name);
                        }
                        
                        Ok(Value::Stream(stream))
                    }
                    _ => Err(anyhow::anyhow!("Cannot branch non-stream value").into())
                }
            }
            Expression::StreamMerge { streams, output_name } => {
                let mut stream_names = Vec::new();
                
                for stream_expr in streams {
                    let stream_val = self.evaluate_expression(stream_expr)?;
                    match stream_val {
                        Value::Stream(stream) => stream_names.push(stream.name),
                        _ => return Err(anyhow::anyhow!("Cannot merge non-stream value").into())
                    }
                }
                
                if !stream_names.is_empty() {
                    println!("Merging {} streams into '{}'", stream_names.len(), output_name);
                    // Placeholder - actual merge would happen at execution level
                    
                    Ok(Value::Stream(crate::runtime::types::Stream {
                        name: output_name.clone(),
                        data_type: crate::runtime::types::DataType::Audio,
                        sample_rate: Some(44100.0),
                    }))
                } else {
                    Err(anyhow::anyhow!("No streams to merge").into())
                }
            }
            Expression::UnitValue { value, unit } => {
                let val = self.evaluate_expression(value)?;
                match val {
                    Value::Integer(n) => {
                        if let Some(unit_val) = crate::runtime::units::UnitValue::from_string(n as f64, unit) {
                            Ok(Value::UnitValue(unit_val))
                        } else {
                            Err(anyhow::anyhow!("Unknown unit: {}", unit).into())
                        }
                    }
                    Value::Float(f) => {
                        if let Some(unit_val) = crate::runtime::units::UnitValue::from_string(f, unit) {
                            Ok(Value::UnitValue(unit_val))
                        } else {
                            Err(anyhow::anyhow!("Unknown unit: {}", unit).into())
                        }
                    }
                    _ => Err(anyhow::anyhow!("Unit values must be numeric, got {:?}", val.type_name()).into()),
                }
            }
            Expression::ArrayLiteral(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate_expression(element)?);
                }
                Ok(Value::Array(values))
            }
            Expression::Range { start, end, inclusive: _ } => {
                let start_val = self.evaluate_expression(start)?;
                let end_val = self.evaluate_expression(end)?;
                // For now, create a simple range representation
                Ok(Value::String(format!("{}..{}", start_val, end_val)))
            }
            Expression::Lambda { parameters: _, body: _ } => {
                // TODO: Implement lambda expressions
                Ok(Value::String("<lambda>".to_string()))
            }
            Expression::MethodCall { object, method, args, named_args: _ } => {
                let obj_val = self.evaluate_expression(object)?;
                // For now, handle basic method calls
                match method.as_str() {
                    "map" | "push" | "length" => {
                        // TODO: Implement method calls properly
                        Ok(obj_val)
                    }
                    _ => Ok(Value::Null)
                }
            }
            Expression::InterpolatedString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        crate::parser::ast::StringPart::Text(text) => {
                            result.push_str(text);
                        }
                        crate::parser::ast::StringPart::Interpolation(expr) => {
                            let val = self.evaluate_expression(expr)?;
                            result.push_str(&val.to_string());
                        }
                    }
                }
                Ok(Value::String(result))
            }
            Expression::ConditionalExpression { condition, true_expr, false_expr } => {
                let cond_val = self.evaluate_expression(condition)?;
                if cond_val.is_truthy() {
                    self.evaluate_expression(true_expr)
                } else {
                    self.evaluate_expression(false_expr)
                }
            }
            Expression::MatchExpression { expr: _, arms: _ } => {
                // TODO: Implement match expressions
                Ok(Value::Null)
            }
            Expression::TypeCast { expr, target_type: _ } => {
                // For now, just return the expression value
                self.evaluate_expression(expr)
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
        _named_args: &std::collections::HashMap<String, Expression>,
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
            return Err(anyhow::anyhow!("Function {}.{} not found", module_name, name).into());
        }
        
        Err(anyhow::anyhow!("Function {} not found", name).into())
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
                (Value::UnitValue(a), Value::UnitValue(b)) => {
                    if let Some(result) = a.add(b) {
                        Ok(Value::UnitValue(result))
                    } else {
                        Err(anyhow::anyhow!("Cannot add incompatible units: {} and {}", a.unit.to_string(), b.unit.to_string()).into())
                    }
                }
                _ => Err(anyhow::anyhow!("Cannot add {:?} and {:?}", left.type_name(), right.type_name()).into()),
            },
            BinaryOperator::Subtract => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
                (Value::UnitValue(a), Value::UnitValue(b)) => {
                    if let Some(result) = a.subtract(b) {
                        Ok(Value::UnitValue(result))
                    } else {
                        Err(anyhow::anyhow!("Cannot subtract incompatible units: {} and {}", a.unit.to_string(), b.unit.to_string()).into())
                    }
                }
                _ => Err(anyhow::anyhow!("Cannot subtract {:?} and {:?}", left.type_name(), right.type_name()).into()),
            },
            BinaryOperator::Multiply => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
                (Value::UnitValue(a), Value::Integer(b)) => Ok(Value::UnitValue(a.multiply(*b as f64))),
                (Value::UnitValue(a), Value::Float(b)) => Ok(Value::UnitValue(a.multiply(*b))),
                (Value::Integer(a), Value::UnitValue(b)) => Ok(Value::UnitValue(b.multiply(*a as f64))),
                (Value::Float(a), Value::UnitValue(b)) => Ok(Value::UnitValue(b.multiply(*a))),
                _ => Err(anyhow::anyhow!("Cannot multiply {:?} and {:?}", left.type_name(), right.type_name()).into()),
            },
            BinaryOperator::Divide => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if *b == 0 {
                        return Err(anyhow::anyhow!("Division by zero").into());
                    }
                    Ok(Value::Float(*a as f64 / *b as f64))
                },
                (Value::Float(a), Value::Float(b)) => {
                    if *b == 0.0 {
                        return Err(anyhow::anyhow!("Division by zero").into());
                    }
                    Ok(Value::Float(a / b))
                },
                (Value::Integer(a), Value::Float(b)) => {
                    if *b == 0.0 {
                        return Err(anyhow::anyhow!("Division by zero").into());
                    }
                    Ok(Value::Float(*a as f64 / b))
                },
                (Value::Float(a), Value::Integer(b)) => {
                    if *b == 0 {
                        return Err(anyhow::anyhow!("Division by zero").into());
                    }
                    Ok(Value::Float(a / *b as f64))
                },
                (Value::UnitValue(a), Value::Integer(b)) => {
                    if *b == 0 {
                        return Err(anyhow::anyhow!("Division by zero").into());
                    }
                    if let Some(result) = a.divide(*b as f64) {
                        Ok(Value::UnitValue(result))
                    } else {
                        Err(anyhow::anyhow!("Division by zero").into())
                    }
                },
                (Value::UnitValue(a), Value::Float(b)) => {
                    if *b == 0.0 {
                        return Err(anyhow::anyhow!("Division by zero").into());
                    }
                    if let Some(result) = a.divide(*b) {
                        Ok(Value::UnitValue(result))
                    } else {
                        Err(anyhow::anyhow!("Division by zero").into())
                    }
                },
                _ => Err(anyhow::anyhow!("Cannot divide {:?} and {:?}", left.type_name(), right.type_name()).into()),
            },
            BinaryOperator::Equal => Ok(Value::Boolean(self.values_equal(left, right))),
            BinaryOperator::NotEqual => Ok(Value::Boolean(!self.values_equal(left, right))),
            BinaryOperator::LessThan => {
                if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
                    Ok(Value::Boolean(a < b))
                } else {
                    Err(anyhow::anyhow!("Cannot compare {:?} and {:?}", left.type_name(), right.type_name()).into())
                }
            },
            BinaryOperator::LessThanOrEqual => {
                if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
                    Ok(Value::Boolean(a <= b))
                } else {
                    Err(anyhow::anyhow!("Cannot compare {:?} and {:?}", left.type_name(), right.type_name()).into())
                }
            },
            BinaryOperator::GreaterThan => {
                if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
                    Ok(Value::Boolean(a > b))
                } else {
                    Err(anyhow::anyhow!("Cannot compare {:?} and {:?}", left.type_name(), right.type_name()).into())
                }
            },
            BinaryOperator::GreaterThanOrEqual => {
                if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
                    Ok(Value::Boolean(a >= b))
                } else {
                    Err(anyhow::anyhow!("Cannot compare {:?} and {:?}", left.type_name(), right.type_name()).into())
                }
            },
            BinaryOperator::Pipe => {
                // For now, just return the right operand - full pipe logic would need more context
                Ok(right.clone())
            },
            BinaryOperator::BiDirectionalPipe => {
                // For bidirectional pipe, return the left operand
                Ok(left.clone())
            },
            BinaryOperator::LogicalAnd => {
                if left.is_truthy() {
                    Ok(right.clone())
                } else {
                    Ok(left.clone())
                }
            },
            BinaryOperator::LogicalOr => {
                if left.is_truthy() {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
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
            (Value::UnitValue(a), Value::UnitValue(b)) => {
                if a.unit.is_compatible(&b.unit) {
                    if let Some(converted) = b.convert_to(&a.unit) {
                        (a.value - converted.value).abs() < f64::EPSILON
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    fn pattern_matches(&self, pattern: &crate::parser::ast::Pattern, value: &Value) -> crate::Result<bool> {
        use crate::parser::ast::Pattern;
        
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Literal(lit) => {
                let pattern_val = self.evaluate_literal(lit);
                Ok(self.values_equal(&pattern_val, value))
            }
            Pattern::Identifier(name) => {
                // For now, treat identifiers as enum variant names
                match value {
                    Value::String(s) => Ok(s == name),
                    _ => Ok(false),
                }
            }
            Pattern::Enum { name, fields: _ } => {
                // For now, just match against string values
                match value {
                    Value::String(s) => Ok(s == name),
                    _ => Ok(false),
                }
            }
        }
    }
    
    fn register_builtin_modules(&mut self) {
        // Graphics module
        let mut graphics_module = Module {
            name: "Graphics".to_string(),
            functions: HashMap::new(),
        };
        
        graphics_module.functions.insert("clear".to_string(), ModuleFunction {
            name: "clear".to_string(),
            callback: crate::modules::graphics::clear,
        });
        
        graphics_module.functions.insert("plasma".to_string(), ModuleFunction {
            name: "plasma".to_string(),
            callback: crate::modules::graphics::plasma,
        });
        
        graphics_module.functions.insert("starfield".to_string(), ModuleFunction {
            name: "starfield".to_string(),
            callback: crate::modules::graphics::starfield,
        });
        
        graphics_module.functions.insert("flash".to_string(), ModuleFunction {
            name: "flash".to_string(),
            callback: crate::modules::graphics::flash,
        });
        
        graphics_module.functions.insert("rect".to_string(), ModuleFunction {
            name: "rect".to_string(),
            callback: crate::modules::graphics::rect,
        });
        
        graphics_module.functions.insert("circle".to_string(), ModuleFunction {
            name: "circle".to_string(),
            callback: crate::modules::graphics::circle,
        });
        
        graphics_module.functions.insert("line".to_string(), ModuleFunction {
            name: "line".to_string(),
            callback: crate::modules::graphics::line,
        });
        
        graphics_module.functions.insert("text".to_string(), ModuleFunction {
            name: "text".to_string(),
            callback: crate::modules::graphics::text,
        });
        
        // Advanced effects
        graphics_module.functions.insert("particle_system".to_string(), ModuleFunction {
            name: "particle_system".to_string(),
            callback: crate::modules::graphics::particle_system,
        });
        
        graphics_module.functions.insert("bloom_effect".to_string(), ModuleFunction {
            name: "bloom_effect".to_string(),
            callback: crate::modules::graphics::bloom_effect,
        });
        
        graphics_module.functions.insert("depth_of_field".to_string(), ModuleFunction {
            name: "depth_of_field".to_string(),
            callback: crate::modules::graphics::depth_of_field,
        });
        
        graphics_module.functions.insert("screen_shake".to_string(), ModuleFunction {
            name: "screen_shake".to_string(),
            callback: crate::modules::graphics::screen_shake,
        });
        
        graphics_module.functions.insert("wind_effect".to_string(), ModuleFunction {
            name: "wind_effect".to_string(),
            callback: crate::modules::graphics::wind_effect,
        });
        
        graphics_module.functions.insert("flash".to_string(), ModuleFunction {
            name: "flash".to_string(),
            callback: crate::modules::graphics::flash,
        });
        
        graphics_module.functions.insert("lightning_strike".to_string(), ModuleFunction {
            name: "lightning_strike".to_string(),
            callback: crate::modules::graphics::lightning_strike,
        });
        
        graphics_module.functions.insert("rainbow_arc".to_string(), ModuleFunction {
            name: "rainbow_arc".to_string(),
            callback: crate::modules::graphics::rainbow_arc,
        });
        
        graphics_module.functions.insert("rain_effect".to_string(), ModuleFunction {
            name: "rain_effect".to_string(),
            callback: crate::modules::graphics::rain_effect,
        });
        
        self.modules.insert("Graphics".to_string(), graphics_module);
        
        // Audio module
        let mut audio_module = Module {
            name: "Audio".to_string(),
            functions: HashMap::new(),
        };
        
        audio_module.functions.insert("mic_input".to_string(), ModuleFunction {
            name: "mic_input".to_string(),
            callback: crate::modules::audio::mic_input,
        });
        
        audio_module.functions.insert("analyze_fft".to_string(), ModuleFunction {
            name: "analyze_fft".to_string(),
            callback: crate::modules::audio::analyze_fft,
        });
        
        audio_module.functions.insert("beat_detect".to_string(), ModuleFunction {
            name: "beat_detect".to_string(),
            callback: crate::modules::audio::beat_detect,
        });
        
        audio_module.functions.insert("load_file".to_string(), ModuleFunction {
            name: "load_file".to_string(),
            callback: crate::modules::audio::load_file,
        });
        
        audio_module.functions.insert("play".to_string(), ModuleFunction {
            name: "play".to_string(),
            callback: crate::modules::audio::play,
        });
        
        audio_module.functions.insert("volume".to_string(), ModuleFunction {
            name: "volume".to_string(),
            callback: crate::modules::audio::volume,
        });
        
        // Audio classification functions
        audio_module.functions.insert("classify_beat".to_string(), ModuleFunction {
            name: "classify_beat".to_string(),
            callback: crate::modules::audio::classify_beat,
        });
        
        audio_module.functions.insert("classify_mood".to_string(), ModuleFunction {
            name: "classify_mood".to_string(),
            callback: crate::modules::audio::classify_mood,
        });
        
        audio_module.functions.insert("onset_detection".to_string(), ModuleFunction {
            name: "onset_detection".to_string(),
            callback: crate::modules::audio::onset_detection,
        });
        
        audio_module.functions.insert("tempo_detection".to_string(), ModuleFunction {
            name: "tempo_detection".to_string(),
            callback: crate::modules::audio::tempo_detection,
        });
        
        audio_module.functions.insert("spectral_centroid".to_string(), ModuleFunction {
            name: "spectral_centroid".to_string(),
            callback: crate::modules::audio::spectral_centroid,
        });
        
        self.modules.insert("Audio".to_string(), audio_module);
        
        // Math module
        let mut math_module = Module {
            name: "Math".to_string(),
            functions: HashMap::new(),
        };
        
        math_module.functions.insert("sin".to_string(), ModuleFunction {
            name: "sin".to_string(),
            callback: crate::modules::math::sin,
        });
        
        math_module.functions.insert("cos".to_string(), ModuleFunction {
            name: "cos".to_string(),
            callback: crate::modules::math::cos,
        });
        
        math_module.functions.insert("sqrt".to_string(), ModuleFunction {
            name: "sqrt".to_string(),
            callback: crate::modules::math::sqrt,
        });
        
        math_module.functions.insert("abs".to_string(), ModuleFunction {
            name: "abs".to_string(),
            callback: crate::modules::math::abs,
        });
        
        math_module.functions.insert("min".to_string(), ModuleFunction {
            name: "min".to_string(),
            callback: crate::modules::math::min,
        });
        
        math_module.functions.insert("max".to_string(), ModuleFunction {
            name: "max".to_string(),
            callback: crate::modules::math::max,
        });
        
        math_module.functions.insert("floor".to_string(), ModuleFunction {
            name: "floor".to_string(),
            callback: crate::modules::math::floor,
        });
        
        math_module.functions.insert("ceil".to_string(), ModuleFunction {
            name: "ceil".to_string(),
            callback: crate::modules::math::ceil,
        });
        
        math_module.functions.insert("round".to_string(), ModuleFunction {
            name: "round".to_string(),
            callback: crate::modules::math::round,
        });
        
        math_module.functions.insert("pow".to_string(), ModuleFunction {
            name: "pow".to_string(),
            callback: crate::modules::math::pow,
        });
        
        math_module.functions.insert("log".to_string(), ModuleFunction {
            name: "log".to_string(),
            callback: crate::modules::math::log,
        });
        
        math_module.functions.insert("exp".to_string(), ModuleFunction {
            name: "exp".to_string(),
            callback: crate::modules::math::exp,
        });
        
        math_module.functions.insert("tan".to_string(), ModuleFunction {
            name: "tan".to_string(),
            callback: crate::modules::math::tan,
        });
        
        math_module.functions.insert("clamp".to_string(), ModuleFunction {
            name: "clamp".to_string(),
            callback: crate::modules::math::clamp,
        });
        
        math_module.functions.insert("lerp".to_string(), ModuleFunction {
            name: "lerp".to_string(),
            callback: crate::modules::math::lerp,
        });
        
        self.modules.insert("Math".to_string(), math_module);
        
        // GUI module
        let mut gui_module = Module {
            name: "GUI".to_string(),
            functions: HashMap::new(),
        };
        
        gui_module.functions.insert("window".to_string(), ModuleFunction {
            name: "window".to_string(),
            callback: crate::modules::gui::window,
        });
        
        gui_module.functions.insert("button".to_string(), ModuleFunction {
            name: "button".to_string(),
            callback: crate::modules::gui::button,
        });
        
        gui_module.functions.insert("slider".to_string(), ModuleFunction {
            name: "slider".to_string(),
            callback: crate::modules::gui::slider,
        });
        
        gui_module.functions.insert("checkbox".to_string(), ModuleFunction {
            name: "checkbox".to_string(),
            callback: crate::modules::gui::checkbox,
        });
        
        gui_module.functions.insert("dropdown".to_string(), ModuleFunction {
            name: "dropdown".to_string(),
            callback: crate::modules::gui::dropdown,
        });
        
        gui_module.functions.insert("control_group".to_string(), ModuleFunction {
            name: "control_group".to_string(),
            callback: crate::modules::gui::control_group,
        });
        
        self.modules.insert("GUI".to_string(), gui_module);
        
        // Generate module
        let mut generate_module = Module {
            name: "Generate".to_string(),
            functions: HashMap::new(),
        };
        
        generate_module.functions.insert("l_system".to_string(), ModuleFunction {
            name: "l_system".to_string(),
            callback: crate::modules::generate::l_system,
        });
        
        generate_module.functions.insert("perlin_noise".to_string(), ModuleFunction {
            name: "perlin_noise".to_string(),
            callback: crate::modules::generate::perlin_noise,
        });
        
        generate_module.functions.insert("euclidean".to_string(), ModuleFunction {
            name: "euclidean".to_string(),
            callback: crate::modules::generate::euclidean,
        });
        
        generate_module.functions.insert("fractal_terrain".to_string(), ModuleFunction {
            name: "fractal_terrain".to_string(),
            callback: crate::modules::generate::fractal_terrain,
        });
        
        self.modules.insert("Generate".to_string(), generate_module);
        
        // Timeline module
        let mut timeline_module = Module {
            name: "Timeline".to_string(),
            functions: HashMap::new(),
        };
        
        timeline_module.functions.insert("create".to_string(), ModuleFunction {
            name: "create".to_string(),
            callback: crate::modules::time::timeline_create,
        });
        
        timeline_module.functions.insert("sequencer".to_string(), ModuleFunction {
            name: "sequencer".to_string(),
            callback: crate::modules::time::sequencer_create,
        });
        
        timeline_module.functions.insert("animation_curve".to_string(), ModuleFunction {
            name: "animation_curve".to_string(),
            callback: crate::modules::time::animation_curve_create,
        });
        
        timeline_module.functions.insert("every".to_string(), ModuleFunction {
            name: "every".to_string(),
            callback: crate::modules::time::every,
        });
        
        timeline_module.functions.insert("after".to_string(), ModuleFunction {
            name: "after".to_string(),
            callback: crate::modules::time::after,
        });
        
        timeline_module.functions.insert("sequence".to_string(), ModuleFunction {
            name: "sequence".to_string(),
            callback: crate::modules::time::sequence,
        });
        
        timeline_module.functions.insert("now".to_string(), ModuleFunction {
            name: "now".to_string(),
            callback: crate::modules::time::now,
        });
        
        timeline_module.functions.insert("delta_time".to_string(), ModuleFunction {
            name: "delta_time".to_string(),
            callback: crate::modules::time::delta_time,
        });
        
        timeline_module.functions.insert("fps".to_string(), ModuleFunction {
            name: "fps".to_string(),
            callback: crate::modules::time::fps,
        });
        
        self.modules.insert("Timeline".to_string(), timeline_module);
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}