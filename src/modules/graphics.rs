use crate::runtime::Value;

pub fn clear(args: &[Value]) -> crate::Result<Value> {
    println!("Graphics.clear called with {} args", args.len());
    Ok(Value::Null)
}

pub fn plasma(args: &[Value]) -> crate::Result<Value> {
    println!("Graphics.plasma called with {} args", args.len());
    Ok(Value::Null)
}

pub fn starfield(args: &[Value]) -> crate::Result<Value> {
    println!("Graphics.starfield called with {} args", args.len());
    Ok(Value::Null)
}

pub fn flash(args: &[Value]) -> crate::Result<Value> {
    println!("Graphics.flash called with {} args", args.len());
    Ok(Value::Null)
}