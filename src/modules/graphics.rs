use crate::runtime::Value;
use std::collections::HashMap;

pub fn clear(args: &[Value]) -> crate::Result<Value> {
    let color = args.get(0)
        .and_then(|v| v.as_number())
        .unwrap_or(0x000000 as f64) as i64; // Default to black
    
    println!("Graphics.clear called with color: 0x{:06X}", color);
    Ok(Value::Null)
}

pub fn plasma(args: &[Value]) -> crate::Result<Value> {
    let mut params = HashMap::new();
    
    // Parse named arguments from block expressions
    for arg in args {
        if let Value::Object(fields) = arg {
            for (key, value) in fields {
                params.insert(key.clone(), value.clone());
            }
        }
    }
    
    let speed = params.get("speed")
        .and_then(|v| v.as_number())
        .unwrap_or(1.0);
    
    let intensity = params.get("intensity")
        .and_then(|v| v.as_number())
        .unwrap_or(0.5);
    
    let palette = params.get("palette")
        .map(|v| match v {
            Value::String(s) => s.clone(),
            _ => "default".to_string(),
        })
        .unwrap_or_else(|| "default".to_string());
    
    println!("Graphics.plasma: speed={:.2}, intensity={:.2}, palette={}", speed, intensity, palette);
    Ok(Value::Null)
}

pub fn starfield(args: &[Value]) -> crate::Result<Value> {
    let mut params = HashMap::new();
    
    for arg in args {
        if let Value::Object(fields) = arg {
            for (key, value) in fields {
                params.insert(key.clone(), value.clone());
            }
        }
    }
    
    let count = params.get("count")
        .and_then(|v| v.as_number())
        .unwrap_or(100.0) as i64;
    
    let speed = params.get("speed")
        .and_then(|v| v.as_number())
        .unwrap_or(1.0);
    
    println!("Graphics.starfield: count={}, speed={:.2}", count, speed);
    Ok(Value::Null)
}

pub fn flash(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(anyhow::anyhow!("flash requires color and duration arguments"));
    }
    
    let color = args[0].as_number()
        .unwrap_or(0xFFFFFF as f64) as i64; // Default to white
    
    let duration = args[1].as_number()
        .ok_or_else(|| anyhow::anyhow!("flash duration must be a number"))?;
    
    if duration <= 0.0 {
        return Err(anyhow::anyhow!("flash duration must be positive"));
    }
    
    println!("Graphics.flash: color=0x{:06X}, duration={:.2}s", color, duration);
    Ok(Value::Null)
}

pub fn rect(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 4 {
        return Err(anyhow::anyhow!("rect requires x, y, width, height arguments"));
    }
    
    let x = args[0].as_number()
        .ok_or_else(|| anyhow::anyhow!("rect x must be a number"))?;
    let y = args[1].as_number()
        .ok_or_else(|| anyhow::anyhow!("rect y must be a number"))?;
    let width = args[2].as_number()
        .ok_or_else(|| anyhow::anyhow!("rect width must be a number"))?;
    let height = args[3].as_number()
        .ok_or_else(|| anyhow::anyhow!("rect height must be a number"))?;
    
    let color = args.get(4)
        .and_then(|v| v.as_number())
        .unwrap_or(0xFFFFFF as f64) as i64;
    
    println!("Graphics.rect: x={:.1}, y={:.1}, w={:.1}, h={:.1}, color=0x{:06X}", 
             x, y, width, height, color);
    Ok(Value::Null)
}

pub fn circle(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 3 {
        return Err(anyhow::anyhow!("circle requires x, y, radius arguments"));
    }
    
    let x = args[0].as_number()
        .ok_or_else(|| anyhow::anyhow!("circle x must be a number"))?;
    let y = args[1].as_number()
        .ok_or_else(|| anyhow::anyhow!("circle y must be a number"))?;
    let radius = args[2].as_number()
        .ok_or_else(|| anyhow::anyhow!("circle radius must be a number"))?;
    
    if radius <= 0.0 {
        return Err(anyhow::anyhow!("circle radius must be positive"));
    }
    
    let color = args.get(3)
        .and_then(|v| v.as_number())
        .unwrap_or(0xFFFFFF as f64) as i64;
    
    println!("Graphics.circle: x={:.1}, y={:.1}, radius={:.1}, color=0x{:06X}", 
             x, y, radius, color);
    Ok(Value::Null)
}

pub fn line(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 4 {
        return Err(anyhow::anyhow!("line requires x1, y1, x2, y2 arguments"));
    }
    
    let x1 = args[0].as_number()
        .ok_or_else(|| anyhow::anyhow!("line x1 must be a number"))?;
    let y1 = args[1].as_number()
        .ok_or_else(|| anyhow::anyhow!("line y1 must be a number"))?;
    let x2 = args[2].as_number()
        .ok_or_else(|| anyhow::anyhow!("line x2 must be a number"))?;
    let y2 = args[3].as_number()
        .ok_or_else(|| anyhow::anyhow!("line y2 must be a number"))?;
    
    let color = args.get(4)
        .and_then(|v| v.as_number())
        .unwrap_or(0xFFFFFF as f64) as i64;
    
    println!("Graphics.line: ({:.1},{:.1}) to ({:.1},{:.1}), color=0x{:06X}", 
             x1, y1, x2, y2, color);
    Ok(Value::Null)
}

pub fn text(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 3 {
        return Err(anyhow::anyhow!("text requires text, x, y arguments"));
    }
    
    let text_content = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(anyhow::anyhow!("text content must be a string")),
    };
    
    let x = args[1].as_number()
        .ok_or_else(|| anyhow::anyhow!("text x must be a number"))?;
    let y = args[2].as_number()
        .ok_or_else(|| anyhow::anyhow!("text y must be a number"))?;
    
    let color = args.get(3)
        .and_then(|v| v.as_number())
        .unwrap_or(0xFFFFFF as f64) as i64;
    
    let size = args.get(4)
        .and_then(|v| v.as_number())
        .unwrap_or(16.0);
    
    println!("Graphics.text: '{}' at ({:.1},{:.1}), color=0x{:06X}, size={:.1}", 
             text_content, x, y, color, size);
    Ok(Value::Null)
}