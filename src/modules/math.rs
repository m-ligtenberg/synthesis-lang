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
        .ok_or_else(|| crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "📊 Math.max() needs numbers to compare"
        )
        .with_suggestion("Try: Math.max(3.5, 1.2, 4.8)")
        .with_suggestion("All arguments must be numbers, not text or other types"))?
    
    for arg in &args[1..] {
        let val = arg.as_number()
            .ok_or_else(|| crate::errors::synthesis_error(
                crate::errors::ErrorKind::TypeMismatch,
                "📊 Math.max() found a non-number value"
            )
            .with_suggestion("All arguments to Math.max() must be numbers")
            .with_suggestion("Check that you're not mixing numbers with text or other types"))?;
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
        Err(crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "📊 Math.floor() needs a number to round down"
        )
        .with_suggestion("Try: Math.floor(3.7) → 3")
        .with_suggestion("Use numbers, not text or other types"))
    }
}

pub fn ceil(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.ceil()))
    } else {
        Err(crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "📊 Math.ceil() needs a number to round up"
        )
        .with_suggestion("Try: Math.ceil(3.2) → 4")
        .with_suggestion("Use numbers, not text or other types"))
    }
}

pub fn round(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.round()))
    } else {
        Err(crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "📊 Math.round() needs a number to round to nearest whole"
        )
        .with_suggestion("Try: Math.round(3.6) → 4")
        .with_suggestion("Use numbers, not text or other types"))
    }
}

pub fn pow(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(crate::errors::synthesis_error(
            crate::errors::ErrorKind::InvalidExpression,
            "⚡ Math.pow() needs both base and exponent values"
        )
        .with_suggestion("Try: Math.pow(2, 3) → 8 (2 to the power of 3)")
        .with_suggestion("First number is the base, second is the power"));
    }
    
    let base = args[0].as_number()
        .ok_or_else(|| crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "⚡ Math.pow() base value must be a number"
        )
        .with_suggestion("Try: Math.pow(3.0, 2) for 3 squared")
        .with_suggestion("The first argument (base) must be a number"))?;
    let exponent = args[1].as_number()
        .ok_or_else(|| crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "⚡ Math.pow() exponent value must be a number"
        )
        .with_suggestion("Try: Math.pow(5, 2.0) for 5 to the power of 2")
        .with_suggestion("The second argument (exponent) must be a number"))?;
    
    Ok(Value::Float(base.powf(exponent)))
}

pub fn log(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        if value <= 0.0 {
            Err(crate::errors::synthesis_error(
                crate::errors::ErrorKind::InvalidExpression,
                "📊 Math.log() needs a positive number"
            )
            .with_suggestion("Try: Math.log(10) or Math.log(2.5)")
            .with_suggestion("Logarithms only work with numbers greater than 0"))
        } else {
            Ok(Value::Float(value.ln()))
        }
    } else {
        Err(crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "📊 Math.log() needs a number to calculate logarithm"
        )
        .with_suggestion("Try: Math.log(10) → natural logarithm")
        .with_suggestion("Use numbers, not text or other types"))
    }
}

pub fn exp(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.exp()))
    } else {
        Err(crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "📊 Math.exp() needs a number for exponential calculation"
        )
        .with_suggestion("Try: Math.exp(2) → e to the power of 2")
        .with_suggestion("Use numbers, not text or other types"))
    }
}

pub fn tan(args: &[Value]) -> crate::Result<Value> {
    if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
        Ok(Value::Float(value.tan()))
    } else {
        Err(crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "📐 Math.tan() needs a number for tangent calculation"
        )
        .with_suggestion("Try: Math.tan(0.5) → tangent of 0.5 radians")
        .with_suggestion("Use numbers, not text or other types"))
    }
}

pub fn clamp(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 3 {
        return Err(crate::errors::synthesis_error(
            crate::errors::ErrorKind::InvalidExpression,
            "🎯 Math.clamp() needs value, minimum, and maximum"
        )
        .with_suggestion("Try: Math.clamp(7, 0, 10) → 7 (stays within 0-10 range)")
        .with_suggestion("Math.clamp(15, 0, 10) → 10 (gets limited to max)"));
    }
    
    let value = args[0].as_number()
        .ok_or_else(|| crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "🎯 Math.clamp() value (first argument) must be a number"
        )
        .with_suggestion("Try: Math.clamp(5.5, 0, 10)")
        .with_suggestion("The value to clamp must be a number"))?;
    let min_val = args[1].as_number()
        .ok_or_else(|| crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "🎯 Math.clamp() minimum (second argument) must be a number"
        )
        .with_suggestion("Try: Math.clamp(value, 0.0, max)")
        .with_suggestion("The minimum bound must be a number"))?;
    let max_val = args[2].as_number()
        .ok_or_else(|| crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "🎯 Math.clamp() maximum (third argument) must be a number"
        )
        .with_suggestion("Try: Math.clamp(value, min, 10.0)")
        .with_suggestion("The maximum bound must be a number"))?;
    
    if min_val > max_val {
        return Err(crate::errors::synthesis_error(
            crate::errors::ErrorKind::InvalidExpression,
            "🎯 Math.clamp() minimum value is larger than maximum"
        )
        .with_suggestion(&format!("You have min={}, max={} - try swapping them", min_val, max_val))
        .with_suggestion("Example: Math.clamp(value, 0, 100) - min should be smaller"));
    }
    
    let clamped = value.max(min_val).min(max_val);
    Ok(Value::Float(clamped))
}

pub fn lerp(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 3 {
        return Err(crate::errors::synthesis_error(
            crate::errors::ErrorKind::InvalidExpression,
            "🎨 Math.lerp() needs start, end, and blend values"
        )
        .with_suggestion("Try: Math.lerp(0, 10, 0.5) → 5 (halfway between 0 and 10)")
        .with_suggestion("Linear interpolation: lerp(start, end, 0.0=start, 1.0=end)"));
    }
    
    let start = args[0].as_number()
        .ok_or_else(|| crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "🎨 Math.lerp() start value must be a number"
        )
        .with_suggestion("Try: Math.lerp(0.0, end, t)")
        .with_suggestion("The starting value must be a number"))?;
    let end = args[1].as_number()
        .ok_or_else(|| crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "🎨 Math.lerp() end value must be a number"
        )
        .with_suggestion("Try: Math.lerp(start, 10.0, t)")
        .with_suggestion("The ending value must be a number"))?;
    let t = args[2].as_number()
        .ok_or_else(|| crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "🎨 Math.lerp() blend factor must be a number"
        )
        .with_suggestion("Try: Math.lerp(start, end, 0.5) for halfway blend")
        .with_suggestion("Blend factor: 0.0=start, 0.5=middle, 1.0=end"))?;
    
    let result = start + t * (end - start);
    Ok(Value::Float(result))
}