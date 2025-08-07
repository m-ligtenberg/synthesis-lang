use crate::runtime::Value;

pub fn export_webapp(args: &[Value]) -> crate::Result<Value> {
    println!("Web.export_webapp called with {} args", args.len());
    
    if let Some(Value::String(name)) = args.get(0) {
        println!("Exporting webapp: {}", name);
        Ok(Value::Boolean(true))
    } else {
        Err(crate::errors::synthesis_error(
            crate::errors::ErrorKind::TypeMismatch,
            "🌐 Web.export_webapp() needs a name for your creative app"
        )
        .with_suggestion("Try: Web.export_webapp(\"MyAudioVisualizer\")")
        .with_suggestion("Use a text name to identify your web app"))
    }
}