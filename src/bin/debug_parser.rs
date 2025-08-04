use std::env;
use std::fs;
use synthesis::parser::{lexer, Parser};

fn main() -> synthesis::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <script.syn>", args[0]);
        return Ok(());
    }
    
    let filename = &args[1];
    let source_code = fs::read_to_string(filename)?;
    
    println!("Source code:");
    println!("{}", source_code);
    println!("\n=== TOKENIZATION ===");
    
    match lexer::tokenize(&source_code) {
        Ok((remaining, tokens)) => {
            for (i, token) in tokens.iter().enumerate() {
                println!("{}: {:?}", i, token);
            }
            if !remaining.is_empty() {
                println!("Remaining unparsed: '{}'", remaining);
            }
            
            println!("\n=== PARSING ===");
            let mut parser = Parser::new(&tokens);
            match parser.parse() {
                Ok(program) => {
                    println!("Parsed successfully!");
                    println!("Program: {:#?}", program);
                }
                Err(e) => {
                    println!("Parse error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Tokenization error: {:?}", e);
        }
    }
    
    Ok(())
}