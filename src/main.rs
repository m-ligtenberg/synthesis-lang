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
        Err(_) => {
            eprintln!("🎵 Can't find your creative file: {}", filename);
            eprintln!("💡 Make sure the file exists and you have permission to read it");
            return Ok(());
        }
    };
    
    println!("Parsing {}...", filename);
    
    let (_, tokens) = lexer::tokenize(&source_code)
        .map_err(|_| synthesis::errors::synthesis_error(
            synthesis::errors::ErrorKind::SyntaxError,
            "🎵 Oops! There's something unusual in your creative code"
        )
        .with_suggestion("Check for typos, missing quotes, or unusual characters")
        .with_suggestion("Try running with --verbose for more details")
        .with_docs("https://synthesis-lang.org/docs/syntax-basics"))?;
    
    let mut parser = Parser::new(&tokens);
    let program = parser.parse()?;
    
    println!("Running {}...", filename);
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program)?;
    
    println!("Program completed successfully.");
    Ok(())
}
