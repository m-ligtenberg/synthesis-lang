use crate::runtime::Value;

pub fn sin(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.sin()))
    } else {
        Err(anyhow::anyhow!("sin() requires a numeric argument"))
    }
}

pub fn cos(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.cos()))
    } else {
        Err(anyhow::anyhow!("cos() requires a numeric argument"))
    }
}

pub fn sqrt(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        if value < 0.0 {
            Err(anyhow::anyhow!("sqrt() requires a non-negative argument"))
        } else {
            Ok(Value::Float(value.sqrt()))
        }
    } else {
        Err(anyhow::anyhow!("sqrt() requires a numeric argument"))
    }
}

pub fn abs(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.abs()))
    } else {
        Err(anyhow::anyhow!("abs() requires a numeric argument"))
    }
}

pub fn min(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(anyhow::anyhow!("min() requires at least 2 arguments"));
    }
    
    let mut min_val = args[0].as_number()
        .ok_or_else(|| anyhow::anyhow!("min() requires numeric arguments"))?;
    
    for arg in &args[1..] {
        let val = arg.as_number()
            .ok_or_else(|| anyhow::anyhow!("min() requires numeric arguments"))?;
        if val < min_val {
            min_val = val;
        }
    }
    
    Ok(Value::Float(min_val))
}

pub fn max(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(anyhow::anyhow!("max() requires at least 2 arguments"));
    }
    
    let mut max_val = args[0].as_number()
        .ok_or_else(|| anyhow::anyhow!("max() requires numeric arguments"))?;
    
    for arg in &args[1..] {
        let val = arg.as_number()
            .ok_or_else(|| anyhow::anyhow!("max() requires numeric arguments"))?;
        if val > max_val {
            max_val = val;
        }
    }
    
    Ok(Value::Float(max_val))
}