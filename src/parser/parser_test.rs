use crate::parser::{ast::*, lexer::tokenize, parser::Parser};

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_expression_from_str(input: &str) -> crate::Result<Expression> {
        let (_, tokens) = tokenize(input).map_err(|_| {
            crate::errors::SynthesisError::new(
                crate::errors::ErrorKind::SyntaxError,
                "Failed to tokenize input"
            )
        })?;
        let mut parser = Parser::new(&tokens);
        parser.parse_expression()
    }

    fn parse_program_from_str(input: &str) -> crate::Result<Program> {
        let (_, tokens) = tokenize(input).map_err(|_| {
            crate::errors::SynthesisError::new(
                crate::errors::ErrorKind::SyntaxError,
                "Failed to tokenize input"
            )
        })?;
        let mut parser = Parser::new(&tokens);
        parser.parse()
    }

    #[test]
    fn test_basic_literals() {
        // Integer literal
        let expr = parse_expression_from_str("42").unwrap();
        assert!(matches!(expr, Expression::Literal(Literal::Integer(42))));

        // Float literal
        let expr = parse_expression_from_str("3.14").unwrap();
        assert!(matches!(expr, Expression::Literal(Literal::Float(f)) if (f - 3.14).abs() < f64::EPSILON));

        // Percentage literal
        let expr = parse_expression_from_str("50%").unwrap();
        assert!(matches!(expr, Expression::Literal(Literal::Percentage(p)) if (p - 0.5).abs() < f64::EPSILON));

        // String literal
        let expr = parse_expression_from_str("\"hello\"").unwrap();
        assert!(matches!(expr, Expression::Literal(Literal::String(s)) if s == "hello"));

        // Boolean literals
        let expr = parse_expression_from_str("true").unwrap();
        assert!(matches!(expr, Expression::Literal(Literal::Boolean(true))));

        let expr = parse_expression_from_str("false").unwrap();
        assert!(matches!(expr, Expression::Literal(Literal::Boolean(false))));
    }

    #[test]
    fn test_arithmetic_expressions() {
        let expr = parse_expression_from_str("2 + 3 * 4").unwrap();
        // Should parse as 2 + (3 * 4) due to operator precedence
        assert!(matches!(expr, Expression::BinaryOp { op: BinaryOperator::Add, .. }));
    }

    #[test]
    fn test_comparison_expressions() {
        let expr = parse_expression_from_str("x > 5").unwrap();
        assert!(matches!(expr, Expression::BinaryOp { op: BinaryOperator::GreaterThan, .. }));

        let expr = parse_expression_from_str("y <= 10.0").unwrap();
        assert!(matches!(expr, Expression::BinaryOp { op: BinaryOperator::LessThanOrEqual, .. }));
    }

    #[test]
    fn test_ranges() {
        let expr = parse_expression_from_str("1..10").unwrap();
        assert!(matches!(expr, Expression::Range { inclusive: false, .. }));

        let expr = parse_expression_from_str("0..=32").unwrap();
        assert!(matches!(expr, Expression::Range { inclusive: true, .. }));
    }

    #[test]
    fn test_function_calls() {
        // Simple function call
        let expr = parse_expression_from_str("function_name()").unwrap();
        assert!(matches!(expr, Expression::FunctionCall { module: None, .. }));

        // Module function call
        let expr = parse_expression_from_str("Audio.mic_input()").unwrap();
        assert!(matches!(expr, Expression::FunctionCall { module: Some(m), .. } if m == "Audio"));

        // Function call with arguments
        let expr = parse_expression_from_str("Audio.analyze_fft(data, 8)").unwrap();
        if let Expression::FunctionCall { args, .. } = expr {
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected function call");
        }
    }

    #[test]
    fn test_named_arguments() {
        let expr = parse_expression_from_str("Graphics.plasma(speed: 2.0, palette: Graphics.neon)").unwrap();
        if let Expression::FunctionCall { named_args, .. } = expr {
            assert!(named_args.contains_key("speed"));
            assert!(named_args.contains_key("palette"));
        } else {
            panic!("Expected function call with named arguments");
        }
    }

    #[test]
    fn test_array_literals() {
        let expr = parse_expression_from_str("[1, 2, 3, 4]").unwrap();
        if let Expression::ArrayLiteral(elements) = expr {
            assert_eq!(elements.len(), 4);
        } else {
            panic!("Expected array literal");
        }

        let expr = parse_expression_from_str("[]").unwrap();
        if let Expression::ArrayLiteral(elements) = expr {
            assert_eq!(elements.len(), 0);
        } else {
            panic!("Expected empty array literal");
        }
    }

    #[test]
    fn test_array_access() {
        let expr = parse_expression_from_str("frequencies[0]").unwrap();
        assert!(matches!(expr, Expression::ArrayAccess { .. }));
    }

    #[test]
    fn test_method_calls() {
        let expr = parse_expression_from_str("Graphics.width").unwrap();
        assert!(matches!(expr, Expression::MethodCall { .. }));
    }

    #[test]
    fn test_pipe_operations() {
        let expr = parse_expression_from_str("data |> process |> output").unwrap();
        // Should parse as nested pipes
        assert!(matches!(expr, Expression::Pipe { .. }));
    }

    #[test]
    fn test_blocks() {
        let expr = parse_expression_from_str("{ x: 10, y: 20 }").unwrap();
        if let Expression::Block { fields } = expr {
            assert!(fields.contains_key("x"));
            assert!(fields.contains_key("y"));
        } else {
            panic!("Expected block expression");
        }
    }

    #[test]
    fn test_import_statements() {
        let program = parse_program_from_str("import Audio").unwrap();
        if let Some(Item::Import(import)) = program.items.first() {
            assert_eq!(import.module, "Audio");
            assert!(import.items.is_none());
        } else {
            panic!("Expected import item");
        }

        let program = parse_program_from_str("import Audio.{mic_input, analyze_fft}").unwrap();
        if let Some(Item::Import(import)) = program.items.first() {
            assert_eq!(import.module, "Audio");
            if let Some(items) = &import.items {
                assert_eq!(items.len(), 2);
                assert!(items.contains(&"mic_input".to_string()));
                assert!(items.contains(&"analyze_fft".to_string()));
            } else {
                panic!("Expected import items");
            }
        } else {
            panic!("Expected import item");
        }
    }

    #[test]
    fn test_assignment_statements() {
        let program = parse_program_from_str("frequency = 440.0").unwrap();
        if let Some(Item::Statement(Statement::Assignment { name, .. })) = program.items.first() {
            assert_eq!(name, "frequency");
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_let_statements() {
        let program = parse_program_from_str("let x = 10").unwrap();
        if let Some(Item::Statement(Statement::Let { name, value, .. })) = program.items.first() {
            assert_eq!(name, "x");
            assert!(value.is_some());
        } else {
            panic!("Expected let statement");
        }

        let program = parse_program_from_str("let y: Number = 3.14").unwrap();
        if let Some(Item::Statement(Statement::Let { name, type_annotation, value })) = program.items.first() {
            assert_eq!(name, "y");
            assert!(type_annotation.is_some());
            assert!(value.is_some());
        } else {
            panic!("Expected let statement with type annotation");
        }
    }

    #[test]
    fn test_if_statements() {
        let program = parse_program_from_str("if x > 5 { y = 10 }").unwrap();
        if let Some(Item::Statement(Statement::If { condition, then_branch, else_branch })) = program.items.first() {
            assert!(matches!(condition, Expression::BinaryOp { .. }));
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_none());
        } else {
            panic!("Expected if statement");
        }

        let program = parse_program_from_str("if condition { do_this } else { do_that }").unwrap();
        if let Some(Item::Statement(Statement::If { else_branch, .. })) = program.items.first() {
            assert!(else_branch.is_some());
        } else {
            panic!("Expected if-else statement");
        }
    }

    #[test]
    fn test_for_loops() {
        let program = parse_program_from_str("for i in 0..10 { print(i) }").unwrap();
        if let Some(Item::Statement(Statement::For { variable, iterable, body })) = program.items.first() {
            assert_eq!(variable, "i");
            assert!(matches!(iterable, Expression::Range { .. }));
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected for loop");
        }
    }

    #[test]
    fn test_while_loops() {
        let program = parse_program_from_str("while running { update() }").unwrap();
        if let Some(Item::Statement(Statement::While { condition, body })) = program.items.first() {
            assert!(matches!(condition, Expression::Identifier(_)));
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected while loop");
        }
    }

    #[test]
    fn test_loop_blocks() {
        let program = parse_program_from_str("loop { update() }").unwrap();
        if let Some(Item::Loop(loop_block)) = program.items.first() {
            assert_eq!(loop_block.body.len(), 1);
        } else {
            panic!("Expected loop block");
        }
    }

    #[test]
    fn test_temporal_statements() {
        let program = parse_program_from_str("every(1.0) { tick() }").unwrap();
        if let Some(Item::Statement(Statement::Every { duration, body })) = program.items.first() {
            assert!(matches!(duration, Expression::Literal(Literal::Float(_))));
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected every statement");
        }

        let program = parse_program_from_str("after(5.0) { finish() }").unwrap();
        if let Some(Item::Statement(Statement::After { .. })) = program.items.first() {
            // Test passed
        } else {
            panic!("Expected after statement");
        }
    }

    #[test]
    fn test_unit_values() {
        let expr = parse_expression_from_str("3.seconds").unwrap();
        if let Expression::UnitValue { value, unit } = expr {
            assert!(matches!(*value, Expression::Literal(Literal::Integer(3))));
            assert_eq!(unit, "seconds");
        } else {
            panic!("Expected unit value");
        }

        let expr = parse_expression_from_str("440.5.hz").unwrap();
        if let Expression::UnitValue { value, unit } = expr {
            assert!(matches!(*value, Expression::Literal(Literal::Float(_))));
            assert_eq!(unit, "hz");
        } else {
            panic!("Expected unit value");
        }
    }

    #[test]
    fn test_complex_expressions() {
        // Test a complex expression from the examples
        let expr = parse_expression_from_str("(analysis.fft_data[0] + analysis.fft_data[1] + analysis.fft_data[2]) / 3.0").unwrap();
        assert!(matches!(expr, Expression::BinaryOp { op: BinaryOperator::Divide, .. }));
    }

    #[test]
    fn test_creative_syntax_features() {
        // Test percentage coordinates  
        let expr = parse_expression_from_str("50%").unwrap();
        if let Expression::Literal(Literal::Percentage(p)) = expr {
            assert!((p - 0.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected percentage literal");
        }

        // Test Graphics properties access
        let expr = parse_expression_from_str("Graphics.width").unwrap();
        assert!(matches!(expr, Expression::MethodCall { .. }));
    }

    #[test] 
    fn test_error_recovery() {
        // Test that parser can handle some malformed input gracefully
        let result = parse_program_from_str("let x = ; let y = 5");
        // Should either recover or provide a helpful error message
        assert!(result.is_err());
    }
}