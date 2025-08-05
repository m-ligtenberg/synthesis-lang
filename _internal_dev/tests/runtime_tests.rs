use synthesis::runtime::{Interpreter, Value};
use synthesis::parser::{lexer::tokenize, Parser};

#[test]
fn test_simple_assignment_and_lookup() {
    let input = "x = 42";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    assert_eq!(interpreter.variables.get("x"), Some(&Value::Integer(42)));
}

#[test]
fn test_arithmetic_operations() {
    let input = "result = 10 + 5 * 2";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    // Should be 10 + (5 * 2) = 20
    assert_eq!(interpreter.variables.get("result"), Some(&Value::Integer(20)));
}

#[test]
fn test_audio_mic_input() {
    let input = "audio = Audio.mic_input()";
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    // Should create a stream value
    match interpreter.variables.get("audio") {
        Some(Value::Stream(stream)) => {
            assert_eq!(stream.name, "microphone");
            assert_eq!(stream.sample_rate, Some(44100.0));
        }
        _ => panic!("Expected stream value"),
    }
}

#[test]
fn test_audio_analyze_fft() {
    let input = r#"
        audio = Audio.mic_input()
        fft_data = Audio.analyze_fft(audio, 8)
    "#;
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    // Should create an array with 8 elements
    match interpreter.variables.get("fft_data") {
        Some(Value::Array(arr)) => {
            assert_eq!(arr.len(), 8);
            // All elements should be floats
            for element in arr {
                match element {
                    Value::Float(_) => {},
                    _ => panic!("Expected float values in FFT array"),
                }
            }
        }
        _ => panic!("Expected array value"),
    }
}

#[test]
fn test_array_access() {
    let input = r#"
        audio = Audio.mic_input()
        fft_data = Audio.analyze_fft(audio, 8)
        first_value = fft_data[0]
    "#;
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    // Should access the first element of the array
    match interpreter.variables.get("first_value") {
        Some(Value::Float(_)) => {},
        _ => panic!("Expected float value from array access"),
    }
}

#[test]
fn test_if_statement_true() {
    let input = r#"
        condition = true
        result = 0
        if condition {
            result = 42
        }
    "#;
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    assert_eq!(interpreter.variables.get("result"), Some(&Value::Integer(42)));
}

#[test]
fn test_if_statement_false() {
    let input = r#"
        condition = false
        result = 0
        if condition {
            result = 42
        }
    "#;
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    assert_eq!(interpreter.variables.get("result"), Some(&Value::Integer(0)));
}

#[test]
fn test_graphics_constants() {
    let input = r#"
        black_color = Graphics.black
        white_color = Graphics.white
        neon_palette = Graphics.neon
    "#;
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    assert_eq!(interpreter.variables.get("black_color"), Some(&Value::Integer(0x000000)));
    assert_eq!(interpreter.variables.get("white_color"), Some(&Value::Integer(0xFFFFFF)));
    assert_eq!(interpreter.variables.get("neon_palette"), Some(&Value::String("neon".to_string())));
}

#[test]
fn test_function_call_execution() {
    let input = r#"
        Graphics.clear(Graphics.black)
        Graphics.plasma(speed: 1.0, palette: Graphics.neon)
        Graphics.flash(Graphics.white, 0.1)
    "#;
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    // Should not panic and execute successfully
    interpreter.execute(&program).unwrap();
}

#[test]
fn test_type_conversions() {
    let input = r#"
        int_result = 5 + 3
        float_result = 5.0 + 3.0
        mixed_result = 5 + 3.0
    "#;
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    assert_eq!(interpreter.variables.get("int_result"), Some(&Value::Integer(8)));
    assert_eq!(interpreter.variables.get("float_result"), Some(&Value::Float(8.0)));
    assert_eq!(interpreter.variables.get("mixed_result"), Some(&Value::Float(8.0)));
}

#[test]
fn test_beat_detection() {
    let input = r#"
        audio = Audio.mic_input()
        beat = Audio.beat_detect(audio)
    "#;
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    // Should return a boolean (false for now in mock implementation)
    assert_eq!(interpreter.variables.get("beat"), Some(&Value::Boolean(false)));
}