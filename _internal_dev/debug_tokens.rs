use std::fs;
use synthesis::parser::lexer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string("examples/hello.syn")?;
    println!("Source code:");
    println!("{}", content);
    println!("\nTokens:");
    
    match lexer::tokenize(&content) {
        Ok((remaining, tokens)) => {
            for (i, token) in tokens.iter().enumerate() {
                println!("{}: {:?}", i, token);
            }
            if !remaining.is_empty() {
                println!("Remaining unparsed: {:?}", remaining);
            }
        }
        Err(e) => {
            println!("Tokenization error: {:?}", e);
        }
    }
    
    Ok(())
}