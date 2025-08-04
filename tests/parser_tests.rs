use synthesis::parser::{lexer::tokenize, Parser, ast::*};

#[test]
fn test_simple_assignment() {
    let input = "x = 42";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Statement(Statement::Assignment { name, value }) => {
            assert_eq!(name, "x");
            assert_eq!(*value, Expression::Literal(Literal::Integer(42)));
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_import_statement() {
    let input = "import Graphics.{clear, plasma}";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Import(import) => {
            assert_eq!(import.module, "Graphics");
            assert_eq!(import.items, Some(vec!["clear".to_string(), "plasma".to_string()]));
        }
        _ => panic!("Expected import statement"),
    }
}

#[test]
fn test_function_call() {
    let input = "Audio.mic_input()";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Statement(Statement::Expression(Expression::FunctionCall { module, name, args })) => {
            assert_eq!(module, &Some("Audio".to_string()));
            assert_eq!(name, "mic_input");
            assert_eq!(args.len(), 0);
        }
        _ => panic!("Expected function call expression"),
    }
}

#[test]
fn test_function_call_with_args() {
    let input = "Audio.analyze_fft(audio, 8)";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Statement(Statement::Expression(Expression::FunctionCall { module, name, args })) => {
            assert_eq!(module, &Some("Audio".to_string()));
            assert_eq!(name, "analyze_fft");
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], Expression::Identifier("audio".to_string()));
            assert_eq!(args[1], Expression::Literal(Literal::Integer(8)));
        }
        _ => panic!("Expected function call expression"),
    }
}

#[test]
fn test_array_access() {
    let input = "data[0]";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Statement(Statement::Expression(Expression::ArrayAccess { array, index })) => {
            assert_eq!(**array, Expression::Identifier("data".to_string()));
            assert_eq!(**index, Expression::Literal(Literal::Integer(0)));
        }
        _ => panic!("Expected array access expression"),
    }
}

#[test]
fn test_named_arguments() {
    let input = "Graphics.plasma(speed: fft_data[0])";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Statement(Statement::Expression(Expression::FunctionCall { module, name, args })) => {
            assert_eq!(module, &Some("Graphics".to_string()));
            assert_eq!(name, "plasma");
            assert_eq!(args.len(), 1);
            
            // Named argument should be parsed as a block
            match &args[0] {
                Expression::Block { fields } => {
                    assert_eq!(fields.len(), 1);
                    assert!(fields.contains_key("speed"));
                }
                _ => panic!("Expected block expression for named argument"),
            }
        }
        _ => panic!("Expected function call expression"),
    }
}

#[test]
fn test_if_statement() {
    let input = "if beat { Graphics.flash(white, 0.1) }";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Statement(Statement::If { condition, then_branch, else_branch }) => {
            assert_eq!(*condition, Expression::Identifier("beat".to_string()));
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_loop_statement() {
    let input = "loop { x = x + 1 }";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Loop(loop_block) => {
            assert_eq!(loop_block.body.len(), 1);
            match &loop_block.body[0] {
                Statement::Assignment { name, .. } => {
                    assert_eq!(name, "x");
                }
                _ => panic!("Expected assignment in loop body"),
            }
        }
        _ => panic!("Expected loop statement"),
    }
}

#[test]
fn test_binary_operations() {
    let input = "result = a + b * c";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Statement(Statement::Assignment { name, value }) => {
            assert_eq!(name, "result");
            // Should parse as a + (b * c) due to operator precedence
            match value {
                Expression::BinaryOp { left, op, right } => {
                    assert_eq!(**left, Expression::Identifier("a".to_string()));
                    assert_eq!(*op, BinaryOperator::Add);
                    // Right side should be b * c
                    match right.as_ref() {
                        Expression::BinaryOp { left, op, right } => {
                            assert_eq!(**left, Expression::Identifier("b".to_string()));
                            assert_eq!(*op, BinaryOperator::Multiply);
                            assert_eq!(**right, Expression::Identifier("c".to_string()));
                        }
                        _ => panic!("Expected binary operation on right side"),
                    }
                }
                _ => panic!("Expected binary operation"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}