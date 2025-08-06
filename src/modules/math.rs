use crate::runtime::Value;

pub fn sin(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.sin()))
    } else {
        Err(crate::errors::synthesis_error(crate::errors::ErrorKind::TypeMismatch, "sin() requires a numeric argument"))
    }
}

pub fn cos(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.cos()))
    } else {
        Err(crate::errors::synthesis_error(crate::errors::ErrorKind::TypeMismatch, "cos() requires a numeric argument"))
    }
}

pub fn sqrt(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        if value < 0.0 {
            Err(crate::errors::synthesis_error(crate::errors::ErrorKind::InvalidExpression, "sqrt() requires a non-negative argument"))
        } else {
            Ok(Value::Float(value.sqrt()))
        }
    } else {
        Err(crate::errors::synthesis_error(crate::errors::ErrorKind::TypeMismatch, "sqrt() requires a numeric argument"))
    }
}

pub fn abs(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.abs()))
    } else {
        Err(crate::errors::synthesis_error(crate::errors::ErrorKind::TypeMismatch, "abs() requires a numeric argument"))
    }
}

pub fn min(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(crate::errors::synthesis_error(crate::errors::ErrorKind::InvalidExpression, "min() requires at least 2 arguments"));
    }
    
    let mut min_val = args[0].as_number()
        .ok_or_else(|| crate::errors::synthesis_error(crate::errors::ErrorKind::TypeMismatch, "min() requires numeric arguments"))?;
    
    for arg in &args[1..] {
        let val = arg.as_number()
            .ok_or_else(|| crate::errors::synthesis_error(crate::errors::ErrorKind::TypeMismatch, "min() requires numeric arguments"))?;
        if val < min_val {
            min_val = val;
        }
    }
    
    Ok(Value::Float(min_val))
}

pub fn max(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(crate::errors::synthesis_error(crate::errors::ErrorKind::InvalidExpression, "max() requires at least 2 arguments"));
    }
    
    let mut max_val = args[0].as_number()
        .ok_or_else(|| anyhow::anyhow!("max(.into() requires numeric arguments"))?;
    
    for arg in &args[1..] {
        let val = arg.as_number()
            .ok_or_else(|| anyhow::anyhow!("max(.into() requires numeric arguments"))?;
        if val > max_val {
            max_val = val;
        }
    }
    
    Ok(Value::Float(max_val))
}

pub fn floor(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.floor()))
    } else {
        Err(anyhow::anyhow!("floor(.into() requires a numeric argument"))
    }
}

pub fn ceil(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.ceil()))
    } else {
        Err(anyhow::anyhow!("ceil(.into() requires a numeric argument"))
    }
}

pub fn round(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.round()))
    } else {
        Err(anyhow::anyhow!("round(.into() requires a numeric argument"))
    }
}

pub fn pow(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(anyhow::anyhow!("pow(.into() requires base and exponent arguments"));
    }
    
    let base = args[0].as_number()
        .ok_or_else(|| anyhow::anyhow!("pow(.into() base must be a number"))?;
    let exponent = args[1].as_number()
        .ok_or_else(|| anyhow::anyhow!("pow(.into() exponent must be a number"))?;
    
    Ok(Value::Float(base.powf(exponent)))
}

pub fn log(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        if value <= 0.0 {
            Err(anyhow::anyhow!("log(.into() requires a positive argument"))
        } else {
            Ok(Value::Float(value.ln()))
        }
    } else {
        Err(anyhow::anyhow!("log(.into() requires a numeric argument"))
    }
}

pub fn exp(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.exp()))
    } else {
        Err(anyhow::anyhow!("exp(.into() requires a numeric argument"))
    }
}

pub fn tan(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.tan()))
    } else {
        Err(anyhow::anyhow!("tan(.into() requires a numeric argument"))
    }
}

pub fn clamp(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 3 {
        return Err(anyhow::anyhow!("clamp(.into() requires value, min, max arguments"));
    }
    
    let value = args[0].as_number()
        .ok_or_else(|| anyhow::anyhow!("clamp(.into() value must be a number"))?;
    let min_val = args[1].as_number()
        .ok_or_else(|| anyhow::anyhow!("clamp(.into() min must be a number"))?;
    let max_val = args[2].as_number()
        .ok_or_else(|| anyhow::anyhow!("clamp(.into() max must be a number"))?;
    
    if min_val > max_val {
        return Err(anyhow::anyhow!("clamp(.into() min must be less than or equal to max"));
    }
    
    let clamped = value.max(min_val).min(max_val);
    Ok(Value::Float(clamped))
}

pub fn lerp(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 3 {
        return Err(anyhow::anyhow!("lerp(.into() requires start, end, t arguments"));
    }
    
    let start = args[0].as_number()
        .ok_or_else(|| anyhow::anyhow!("lerp(.into() start must be a number"))?;
    let end = args[1].as_number()
        .ok_or_else(|| anyhow::anyhow!("lerp(.into() end must be a number"))?;
    let t = args[2].as_number()
        .ok_or_else(|| anyhow::anyhow!("lerp(.into() t must be a number"))?;
    
    let result = start + t * (end - start);
    Ok(Value::Float(result))
}