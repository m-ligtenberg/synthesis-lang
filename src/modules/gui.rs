use crate::runtime::Value;
use std::collections::HashMap;

pub fn window(args: &[Value]) -> crate::Result<Value> {
    println!("GUI.window called with {} args", args.len());
    Ok(Value::Null)
}

pub fn slider(args: &[Value]) -> crate::Result<Value> {
    println!("GUI.slider called with {} args", args.len());
    
    let default_value = args.get(3)
        .and_then(|v| v.as_number())
        .unwrap_or(1.0);
    
    Ok(Value::Float(default_value))
}

pub fn button(args: &[Value]) -> crate::Result<Value> {
    println!("GUI.button called with {} args", args.len());
    Ok(Value::Boolean(false))
}

pub fn control_group(args: &[Value]) -> crate::Result<Value> {
    println!("GUI.control_group called with {} args", args.len());
    
    let mut controls = HashMap::new();
    controls.insert("sensitivity".to_string(), Value::Float(1.0));
    controls.insert("effect_type".to_string(), Value::String("plasma".to_string()));
    
    Ok(Value::Object(controls))
}

pub fn dropdown(args: &[Value]) -> crate::Result<Value> {
    println!("GUI.dropdown called with {} args", args.len());
    
    let default_value = args.get(2)
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "plasma".to_string());
    
    Ok(Value::String(default_value))
}