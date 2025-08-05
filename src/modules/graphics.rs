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

// Advanced Effects Functions

pub fn particle_system(args: &[Value]) -> crate::Result<Value> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("particle_system requires a name argument"));
    }
    
    let name = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(anyhow::anyhow!("particle system name must be a string")),
    };
    
    println!("Graphics.particle_system: Creating '{}' particle system", name);
    
    let mut result = std::collections::HashMap::new();
    result.insert("type".to_string(), Value::String("particle_system".to_string()));
    result.insert("name".to_string(), Value::String(name));
    result.insert("active".to_string(), Value::Boolean(true));
    Ok(Value::Object(result))
}

pub fn bloom_effect(args: &[Value]) -> crate::Result<Value> {
    let threshold = args.get(0)
        .and_then(|v| v.as_number())
        .unwrap_or(0.8);
    
    let intensity = args.get(1)
        .and_then(|v| v.as_number())
        .unwrap_or(1.0);
    
    let radius = args.get(2)
        .and_then(|v| v.as_number())
        .unwrap_or(5.0);
    
    println!("Graphics.bloom_effect: threshold={:.2}, intensity={:.2}, radius={:.1}", 
             threshold, intensity, radius);
    
    let mut result = std::collections::HashMap::new();
    result.insert("type".to_string(), Value::String("bloom_effect".to_string()));
    result.insert("threshold".to_string(), Value::Float(threshold));
    result.insert("intensity".to_string(), Value::Float(intensity));
    result.insert("radius".to_string(), Value::Float(radius));
    Ok(Value::Object(result))
}

pub fn depth_of_field(args: &[Value]) -> crate::Result<Value> {
    let focus_distance = args.get(0)
        .and_then(|v| v.as_number())
        .unwrap_or(5.0);
    
    let aperture = args.get(1)
        .and_then(|v| v.as_number())
        .unwrap_or(2.8);
    
    println!("Graphics.depth_of_field: focus_distance={:.1}, aperture=f/{:.1}", 
             focus_distance, aperture);
    
    let mut result = std::collections::HashMap::new();
    result.insert("type".to_string(), Value::String("depth_of_field".to_string()));
    result.insert("focus_distance".to_string(), Value::Float(focus_distance));
    result.insert("aperture".to_string(), Value::Float(aperture));
    Ok(Value::Object(result))
}

pub fn screen_shake(args: &[Value]) -> crate::Result<Value> {
    let intensity = args.get(0)
        .and_then(|v| v.as_number())
        .unwrap_or(10.0);
    
    let duration = args.get(1)
        .and_then(|v| v.as_number())
        .unwrap_or(0.5);
    
    println!("Graphics.screen_shake: intensity={:.1}, duration={:.2}s", 
             intensity, duration);
    
    let mut result = std::collections::HashMap::new();
    result.insert("type".to_string(), Value::String("screen_shake".to_string()));
    result.insert("intensity".to_string(), Value::Float(intensity));
    result.insert("duration".to_string(), Value::Float(duration));
    Ok(Value::Object(result))
}

pub fn wind_effect(args: &[Value]) -> crate::Result<Value> {
    let direction = args.get(0)
        .and_then(|v| v.as_number())
        .unwrap_or(0.0);
    
    let strength = args.get(1)
        .and_then(|v| v.as_number())
        .unwrap_or(1.0);
    
    let particles = args.get(2)
        .and_then(|v| match v { Value::Boolean(b) => Some(*b), _ => None })
        .unwrap_or(false);
    
    println!("Graphics.wind_effect: direction={:.1}°, strength={:.2}, particles={}", 
             direction, strength, particles);
    
    let mut result = std::collections::HashMap::new();
    result.insert("type".to_string(), Value::String("wind_effect".to_string()));
    result.insert("direction".to_string(), Value::Float(direction));
    result.insert("strength".to_string(), Value::Float(strength));
    result.insert("particles".to_string(), Value::Boolean(particles));
    Ok(Value::Object(result))
}

pub fn flash(args: &[Value]) -> crate::Result<Value> {
    let color = args.get(0)
        .and_then(|v| v.as_number())
        .unwrap_or(0xFFFFFF as f64) as i64;
    
    let duration = args.get(1)
        .and_then(|v| v.as_number())
        .unwrap_or(0.1);
    
    println!("Graphics.flash: color=0x{:06X}, duration={:.2}s", color, duration);
    
    let mut result = std::collections::HashMap::new();
    result.insert("type".to_string(), Value::String("flash".to_string()));
    result.insert("color".to_string(), Value::Integer(color));
    result.insert("duration".to_string(), Value::Float(duration));
    Ok(Value::Object(result))
}

pub fn lightning_strike(args: &[Value]) -> crate::Result<Value> {
    let position_x = args.get(0)
        .and_then(|v| v.as_number())
        .unwrap_or(50.0);
    
    let position_y = args.get(1)
        .and_then(|v| v.as_number())
        .unwrap_or(0.0);
    
    let branches = args.get(2)
        .and_then(|v| v.as_number())
        .unwrap_or(3.0) as i32;
    
    let color = args.get(3)
        .and_then(|v| v.as_number())
        .unwrap_or(0x87CEEB as f64) as i64;
    
    println!("Graphics.lightning_strike: position=({:.1},{:.1}), branches={}, color=0x{:06X}", 
             position_x, position_y, branches, color);
    
    let mut result = std::collections::HashMap::new();
    result.insert("type".to_string(), Value::String("lightning_strike".to_string()));
    result.insert("position_x".to_string(), Value::Float(position_x));
    result.insert("position_y".to_string(), Value::Float(position_y));
    result.insert("branches".to_string(), Value::Integer(branches as i64));
    result.insert("color".to_string(), Value::Integer(color));
    Ok(Value::Object(result))
}

pub fn rainbow_arc(args: &[Value]) -> crate::Result<Value> {
    let center_x = args.get(0)
        .and_then(|v| v.as_number())
        .unwrap_or(50.0);
    
    let center_y = args.get(1)
        .and_then(|v| v.as_number())
        .unwrap_or(50.0);
    
    let radius = args.get(2)
        .and_then(|v| v.as_number())
        .unwrap_or(200.0);
    
    let fade_duration = args.get(3)
        .and_then(|v| v.as_number())
        .unwrap_or(3.0);
    
    println!("Graphics.rainbow_arc: center=({:.1},{:.1}), radius={:.1}, fade={:.1}s", 
             center_x, center_y, radius, fade_duration);
    
    let mut result = std::collections::HashMap::new();
    result.insert("type".to_string(), Value::String("rainbow_arc".to_string()));
    result.insert("center_x".to_string(), Value::Float(center_x));
    result.insert("center_y".to_string(), Value::Float(center_y));
    result.insert("radius".to_string(), Value::Float(radius));
    result.insert("fade_duration".to_string(), Value::Float(fade_duration));
    Ok(Value::Object(result))
}

pub fn rain_effect(args: &[Value]) -> crate::Result<Value> {
    let intensity = args.get(0)
        .and_then(|v| v.as_number())
        .unwrap_or(0.5);
    
    let duration = args.get(1)
        .and_then(|v| v.as_number())
        .unwrap_or(2.0);
    
    println!("Graphics.rain_effect: intensity={:.2}, duration={:.1}s", 
             intensity, duration);
    
    let mut result = std::collections::HashMap::new();
    result.insert("type".to_string(), Value::String("rain_effect".to_string()));
    result.insert("intensity".to_string(), Value::Float(intensity));
    result.insert("duration".to_string(), Value::Float(duration));
    Ok(Value::Object(result))
}