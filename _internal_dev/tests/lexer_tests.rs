use synthesis::parser::lexer::{tokenize, Token};

#[test]
fn test_basic_tokens() {
    let input = "import Audio Graphics 123 45.67 \"hello\" true false";
    let (_, tokens) = tokenize(input).unwrap();
    
    assert_eq!(tokens.len(), 8);
    assert_eq!(tokens[0], Token::Import);
    assert_eq!(tokens[1], Token::Identifier("Audio".to_string()));
    assert_eq!(tokens[2], Token::Identifier("Graphics".to_string()));
    assert_eq!(tokens[3], Token::Integer(123));
    assert_eq!(tokens[4], Token::Float(45.67));
    assert_eq!(tokens[5], Token::String("hello".to_string()));
    assert_eq!(tokens[6], Token::Boolean(true));
}

#[test]
fn test_operators() {
    let input = "+ - * / = == != < <= > >=";
    let (_, tokens) = tokenize(input).unwrap();
    
    assert_eq!(tokens.len(), 11);
    assert_eq!(tokens[0], Token::Plus);
    assert_eq!(tokens[1], Token::Minus);
    assert_eq!(tokens[2], Token::Multiply);
    assert_eq!(tokens[3], Token::Divide);
    assert_eq!(tokens[4], Token::Assignment);
    assert_eq!(tokens[5], Token::Equals);
    assert_eq!(tokens[6], Token::NotEqual);
    assert_eq!(tokens[7], Token::LessThan);
    assert_eq!(tokens[8], Token::LessThanOrEqual);
    assert_eq!(tokens[9], Token::GreaterThan);
    assert_eq!(tokens[10], Token::GreaterThanOrEqual);
}

#[test]
fn test_punctuation() {
    let input = "( ) { } [ ] , : .";
    let (_, tokens) = tokenize(input).unwrap();
    
    assert_eq!(tokens.len(), 9);
    assert_eq!(tokens[0], Token::LeftParen);
    assert_eq!(tokens[1], Token::RightParen);
    assert_eq!(tokens[2], Token::LeftBrace);
    assert_eq!(tokens[3], Token::RightBrace);
    assert_eq!(tokens[4], Token::LeftBracket);
    assert_eq!(tokens[5], Token::RightBracket);
    assert_eq!(tokens[6], Token::Comma);
    assert_eq!(tokens[7], Token::Colon);
    assert_eq!(tokens[8], Token::Dot);
}

#[test]
fn test_keywords() {
    let input = "import loop if else";
    let (_, tokens) = tokenize(input).unwrap();
    
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0], Token::Import);
    assert_eq!(tokens[1], Token::Loop);
    assert_eq!(tokens[2], Token::If);
    assert_eq!(tokens[3], Token::Else);
}

#[test]
fn test_comments() {
    let input = r#"
        // This is a comment
        import Graphics
        // Another comment
        audio = Audio.mic_input()
    "#;
    let (_, tokens) = tokenize(input).unwrap();
    
    // Should only have the non-comment tokens
    assert_eq!(tokens.len(), 9);
    assert_eq!(tokens[0], Token::Import);
    assert_eq!(tokens[1], Token::Identifier("Graphics".to_string()));
    assert_eq!(tokens[2], Token::Identifier("audio".to_string()));
    assert_eq!(tokens[3], Token::Assignment);
    assert_eq!(tokens[4], Token::Identifier("Audio".to_string()));
    assert_eq!(tokens[5], Token::Dot);
    assert_eq!(tokens[6], Token::Identifier("mic_input".to_string()));
    assert_eq!(tokens[7], Token::LeftParen);
    assert_eq!(tokens[8], Token::RightParen);
}

#[test]
fn test_array_syntax() {
    let input = "data[0]";
    let (_, tokens) = tokenize(input).unwrap();
    
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0], Token::Identifier("data".to_string()));
    assert_eq!(tokens[1], Token::LeftBracket);
    assert_eq!(tokens[2], Token::Integer(0));
    assert_eq!(tokens[3], Token::RightBracket);
}

#[test]
fn test_function_call_with_named_args() {
    let input = "Graphics.plasma(speed: 1.0, color: red)";
    let (_, tokens) = tokenize(input).unwrap();
    
    assert_eq!(tokens.len(), 12);
    assert_eq!(tokens[0], Token::Identifier("Graphics".to_string()));
    assert_eq!(tokens[1], Token::Dot);
    assert_eq!(tokens[2], Token::Identifier("plasma".to_string()));
    assert_eq!(tokens[3], Token::LeftParen);
    assert_eq!(tokens[4], Token::Identifier("speed".to_string()));
    assert_eq!(tokens[5], Token::Colon);
    assert_eq!(tokens[6], Token::Float(1.0));
    assert_eq!(tokens[7], Token::Comma);
    assert_eq!(tokens[8], Token::Identifier("color".to_string()));
    assert_eq!(tokens[9], Token::Colon);
    assert_eq!(tokens[10], Token::Identifier("red".to_string()));
    assert_eq!(tokens[11], Token::RightParen);
}