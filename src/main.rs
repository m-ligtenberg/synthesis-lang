use std::env;
use std::fs;
use synthesis::parser::{lexer, Parser};
use synthesis::runtime::Interpreter;

fn main() -> synthesis::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Synthesis Language Interpreter v0.1.0");
        println!("Usage: {} <script.syn>", args[0]);
        println!("\nAvailable commands:");
        println!("  --version    Show version information");
        println!("  --help       Show this help message");
        return Ok(());
    }
    
    match args[1].as_str() {
        "--version" => {
            println!("Synthesis Language v0.1.0");
            println!("A universal creative programming language");
            return Ok(());
        }
        "--help" => {
            println!("Synthesis Language Interpreter");
            println!("Usage: {} <script.syn>", args[0]);
            println!("\nOptions:");
            println!("  --version    Show version information");
            println!("  --help       Show this help message");
            println!("\nExamples:");
            println!("  {} examples/plasma.syn", args[0]);
            return Ok(());
        }
        _ => {}
    }
    
    let filename = &args[1];
    
    if !filename.ends_with(".syn") {
        eprintln!("Error: Synthesis files must have a .syn extension");
        return Ok(());
    }
    
    let source_code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            return Ok(());
        }
    };
    
    println!("Parsing {}...", filename);
    
    let (_, tokens) = lexer::tokenize(&source_code)
        .map_err(|e| anyhow::anyhow!("Lexer error: {:?}", e))?;
    
    let mut parser = Parser::new(&tokens);
    let program = parser.parse()?;
    
    println!("Running {}...", filename);
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program)?;
    
    println!("Program completed successfully.");
    Ok(())
}
