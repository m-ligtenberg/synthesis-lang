use synthesis::parser::lexer::tokenize;

fn main() {
    println!("=== test_basic_tokens ===");
    let input = "import Audio Graphics 123 45.67 \"hello\" true false";
    let (_, tokens) = tokenize(input).unwrap();
    
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }
    println!("Total: {}", tokens.len());
    
    println!("\n=== test_comments ===");
    let input = r#"
        // This is a comment
        import Graphics
        // Another comment
        audio = Audio.mic_input()
    "#;
    let (_, tokens) = tokenize(input).unwrap();
    
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }
    println!("Total: {}", tokens.len());
    
    println!("\n=== test_function_call_with_named_args ===");
    let input = "Graphics.plasma(speed: 1.0, color: red)";
    let (_, tokens) = tokenize(input).unwrap();
    
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }
    println!("Total: {}", tokens.len());
}