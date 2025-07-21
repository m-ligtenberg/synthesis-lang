use crate::runtime::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now(_args: &[Value]) -> crate::Result<Value> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs_f64();
    Ok(Value::Float(timestamp))
}

pub fn delta_time(_args: &[Value]) -> crate::Result<Value> {
    Ok(Value::Float(0.016667))
}

pub fn fps(_args: &[Value]) -> crate::Result<Value> {
    Ok(Value::Float(60.0))
}