use synthesis::parser::{lexer::tokenize, Parser};
use synthesis::runtime::Interpreter;

#[test]
fn test_hello_world_example() {
    let input = r#"
        // Simple hello world example
        import Graphics.{clear, plasma, flash}
        import Audio.{mic_input, analyze_fft, beat_detect}

        audio = Audio.mic_input()
        fft_data = Audio.analyze_fft(audio, 8)
        beat = Audio.beat_detect(audio)

        Graphics.clear(Graphics.black)
        Graphics.plasma(speed: fft_data[0], palette: Graphics.neon)

        if beat {
            Graphics.flash(Graphics.white, 0.1)
        }
    "#;
    
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    // Should execute without errors
    interpreter.execute(&program).unwrap();
    
    // Verify key variables exist
    assert!(interpreter.variables.contains_key("audio"));
    assert!(interpreter.variables.contains_key("fft_data"));
    assert!(interpreter.variables.contains_key("beat"));
}

#[test]
fn test_audio_visualizer_workflow() {
    let input = r#"
        import Audio.{mic_input, analyze_fft}
        import Graphics.{clear, plasma}
        
        // Initialize audio input
        audio = Audio.mic_input()
        
        // Analyze audio frequency spectrum
        frequencies = Audio.analyze_fft(audio, 16)
        
        // Extract bass and treble components
        bass = frequencies[0]
        mid = frequencies[8]
        treble = frequencies[15]
        
        // Clear the screen
        Graphics.clear(Graphics.black)
        
        // Create visualizations based on frequency data
        Graphics.plasma(speed: bass, intensity: mid, color: treble)
    "#;
    
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    // Verify all expected variables are set
    assert!(interpreter.variables.contains_key("audio"));
    assert!(interpreter.variables.contains_key("frequencies"));
    assert!(interpreter.variables.contains_key("bass"));
    assert!(interpreter.variables.contains_key("mid"));
    assert!(interpreter.variables.contains_key("treble"));
}

#[test]
fn test_conditional_logic() {
    let input = r#"
        import Audio.{mic_input, beat_detect}
        import Graphics.{flash, clear}
        
        audio = Audio.mic_input()
        beat = Audio.beat_detect(audio)
        
        counter = 0
        
        if beat {
            counter = counter + 1
            Graphics.flash(Graphics.white, 0.2)
        }
        
        // Test nested conditions
        strong_beat = false
        if counter > 5 {
            strong_beat = true
            if strong_beat {
                Graphics.clear(Graphics.white)
            }
        }
    "#;
    
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    // Since beat is false in mock, counter should remain 0
    assert!(interpreter.variables.contains_key("counter"));
    assert!(interpreter.variables.contains_key("strong_beat"));
}

#[test]
fn test_complex_expressions() {
    let input = r#"
        // Test complex mathematical expressions
        a = 10
        b = 5
        c = 2
        
        // Test operator precedence
        result1 = a + b * c  // Should be 10 + (5 * 2) = 20
        result2 = (a + b) * c  // Should be (10 + 5) * 2 = 30
        result3 = a / b + c  // Should be (10 / 5) + 2 = 4.0
        
        // Test with floating point
        x = 3.14
        y = 2.0
        result4 = x * y + 1.0  // Should be (3.14 * 2.0) + 1.0 = 7.28
    "#;
    
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    use synthesis::runtime::Value;
    
    assert_eq!(interpreter.variables.get("result1"), Some(&Value::Integer(20)));
    assert_eq!(interpreter.variables.get("result2"), Some(&Value::Integer(30)));
    assert_eq!(interpreter.variables.get("result3"), Some(&Value::Float(4.0)));
    
    // Check floating point result with tolerance
    if let Some(Value::Float(val)) = interpreter.variables.get("result4") {
        assert!((val - 7.28).abs() < 0.001);
    } else {
        panic!("Expected float result for result4");
    }
}

#[test]
fn test_array_operations() {
    let input = r#"
        import Audio.{mic_input, analyze_fft}
        
        audio = Audio.mic_input()
        spectrum = Audio.analyze_fft(audio, 32)
        
        // Test multiple array accesses
        low_freq = spectrum[0]
        mid_freq = spectrum[16]
        high_freq = spectrum[31]
        
        // Test array access in expressions
        energy = spectrum[0] + spectrum[1] + spectrum[2]
        
        // Test array access in function calls
        Graphics.plasma(bass: spectrum[0], mid: spectrum[15], treble: spectrum[31])
    "#;
    
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program).unwrap();
    
    // Verify array access results
    assert!(interpreter.variables.contains_key("low_freq"));
    assert!(interpreter.variables.contains_key("mid_freq"));
    assert!(interpreter.variables.contains_key("high_freq"));
    assert!(interpreter.variables.contains_key("energy"));
}

#[test]
fn test_error_handling() {
    // Test array bounds checking
    let input = r#"
        import Audio.{mic_input, analyze_fft}
        
        audio = Audio.mic_input()
        spectrum = Audio.analyze_fft(audio, 8)
        
        // This should cause an error - index out of bounds
        invalid = spectrum[100]
    "#;
    
    let (_, tokens) = tokenize(input).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse().unwrap();
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&program);
    
    // Should return an error due to array bounds
    assert!(result.is_err());
}