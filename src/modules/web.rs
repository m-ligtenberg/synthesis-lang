use crate::runtime::Value;

pub fn export_webapp(args: &[Value]) -> crate::Result<Value> {
    println!("Web.export_webapp called with {} args", args.len());
    
    if let Some(Value::String(name)) = args.get(0) {
        println!("Exporting webapp: {}", name);
        Ok(Value::Boolean(true))
    } else {
        Err(anyhow::anyhow!("export_webapp() requires a name as first argument"))
    }
}