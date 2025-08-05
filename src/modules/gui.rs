use crate::runtime::Value;
use std::collections::HashMap;

pub fn window(args: &[Value]) -> crate::Result<Value> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("window requires a title argument".into());
    }
    
    let title = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(anyhow::anyhow!("window title must be a string".into()),
    };
    
    let mut params = HashMap::new();
    for arg in &args[1..] {
        if let Value::Object(fields) = arg {
            for (key, value) in fields {
                params.insert(key.clone(), value.clone().into());
            }
        }
    }
    
    let theme = params.get("theme")
        .map(|v| match v {
            Value::String(s) => s.clone(),
            _ => "light".to_string(),
        })
        .unwrap_or_else(|| "light".to_string().into());
    
    println!("GUI.window: title='{}', theme='{}'", title, theme);
    Ok(Value::Null)
}

pub fn button(args: &[Value]) -> crate::Result<Value> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("button requires a label argument".into());
    }
    
    let label = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(anyhow::anyhow!("button label must be a string".into()),
    };
    
    let mut params = HashMap::new();
    for arg in &args[1..] {
        if let Value::Object(fields) = arg {
            for (key, value) in fields {
                params.insert(key.clone(), value.clone().into());
            }
        }
    }
    
    let style = params.get("style")
        .map(|v| match v {
            Value::String(s) => s.clone(),
            _ => "default".to_string(),
        })
        .unwrap_or_else(|| "default".to_string().into());
    
    // Mock button click (randomly return true/false for demo)
    let clicked = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() % 100 < 5; // 5% chance of being "clicked"
    
    println!("GUI.button: label='{}', style='{}', clicked={}", label, style, clicked);
    Ok(Value::Boolean(clicked))
}

pub fn slider(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 3 {
        return Err(anyhow::anyhow!("slider requires label, min, max arguments".into());
    }
    
    let label = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(anyhow::anyhow!("slider label must be a string".into()),
    };
    
    let min_val = args[1].as_number()
        .ok_or_else(|| anyhow::anyhow!("slider min must be a number".into())?;
    let max_val = args[2].as_number()
        .ok_or_else(|| anyhow::anyhow!("slider max must be a number".into())?;
    
    if min_val >= max_val {
        return Err(anyhow::anyhow!("slider min must be less than max".into());
    }
    
    let default_val = args.get(3)
        .and_then(|v| v.as_number())
        .unwrap_or((min_val + max_val) / 2.0);
    
    // Mock slider value (oscillate between min and max for demo)
    let time_factor = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    
    let normalized = (time_factor.sin() + 1.0) / 2.0; // 0.0 to 1.0
    let current_value = min_val + normalized * (max_val - min_val);
    
    println!("GUI.slider: label='{}', range=[{:.2}, {:.2}], value={:.2}", 
             label, min_val, max_val, current_value);
    Ok(Value::Float(current_value))
}

pub fn checkbox(args: &[Value]) -> crate::Result<Value> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("checkbox requires a label argument".into());
    }
    
    let label = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(anyhow::anyhow!("checkbox label must be a string".into()),
    };
    
    let default_checked = args.get(1)
        .map(|v| v.is_truthy())
        .unwrap_or(false);
    
    // Mock checkbox state (toggle periodically for demo)
    let time_factor = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() % 10; // Toggle every 10 seconds
    
    let checked = time_factor < 5;
    
    println!("GUI.checkbox: label='{}', checked={}", label, checked);
    Ok(Value::Boolean(checked))
}

pub fn dropdown(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(anyhow::anyhow!("dropdown requires label and options arguments".into());
    }
    
    let label = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(anyhow::anyhow!("dropdown label must be a string".into()),
    };
    
    let options = match &args[1] {
        Value::Array(arr) => {
            let mut opts = Vec::new();
            for opt in arr {
                match opt {
                    Value::String(s) => opts.push(s.clone()),
                    _ => return Err(anyhow::anyhow!("dropdown options must be strings".into()),
                }
            }
            opts
        }
        _ => return Err(anyhow::anyhow!("dropdown options must be an array".into()),
    };
    
    if options.is_empty() {
        return Err(anyhow::anyhow!("dropdown must have at least one option".into());
    }
    
    let default_option = args.get(2)
        .map(|v| match v {
            Value::String(s) => s.clone(),
            _ => options[0].clone(),
        })
        .unwrap_or_else(|| options[0].clone().into());
    
    // Mock selection (cycle through options for demo)
    let time_factor = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize % options.len();
    
    let selected = &options[time_factor];
    
    println!("GUI.dropdown: label='{}', selected='{}' from {:?}", 
             label, selected, options);
    Ok(Value::String(selected.clone()))
}

pub fn control_group(args: &[Value]) -> crate::Result<Value> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("control_group requires a title argument".into());
    }
    
    let title = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(anyhow::anyhow!("control_group title must be a string".into()),
    };
    
    println!("GUI.control_group: title='{}'", title);
    
    // Return an object representing the control group
    let mut controls = HashMap::new();
    controls.insert("title".to_string(), Value::String(title).into());
    controls.insert("sensitivity".to_string(), Value::Float(1.0).into());
    controls.insert("effect_type".to_string(), Value::String("plasma".to_string()).into());
    
    Ok(Value::Object(controls))
}