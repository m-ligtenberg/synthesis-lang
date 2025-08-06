use crate::errors::{get_error_translator, RustErrorTranslator};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_annotation_error_translation() {
        let translator = get_error_translator();
        
        let rust_error = "error[E0283]: type annotations needed
  --> src/main.rs:5:9
   |
5  |     let x = vec![];
   |         ^ cannot infer type for type parameter `T`";
        
        let translated = translator.translate_rust_error(rust_error);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("Synthesis needs a hint"));
    }

    #[test]
    fn test_method_not_found_translation() {
        let translator = get_error_translator();
        
        let rust_error = "error[E0599]: no method named `invalid_method` found for type `AudioBuffer`
  --> src/main.rs:10:15
   |
10 |     audio_data.invalid_method();
   |                ^^^^^^^^^^^^^^ method not found in `AudioBuffer`";
        
        let translated = translator.translate_rust_error(rust_error);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("method"));
        assert!(!error.message.contains("AudioBuffer")); // Should not leak Rust types
    }

    #[test]
    fn test_cannot_find_value_translation() {
        let translator = get_error_translator();
        
        let rust_error = "error[E0425]: cannot find value `undefined_var` in this scope
  --> src/main.rs:8:9
   |
8  |     let y = undefined_var;
   |             ^^^^^^^^^^^^^ not found in this scope";
        
        let translated = translator.translate_rust_error(rust_error);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("not defined"));
    }

    #[test]
    fn test_type_mismatch_translation() {
        let translator = get_error_translator();
        
        let rust_error = "error[E0308]: mismatched types
  --> src/main.rs:12:18
   |
12 |     let result: f32 = \"hello\";
   |                 ---   ^^^^^^^ expected `f32`, found `&str`
   |                 |
   |                 expected due to this";
        
        let translated = translator.translate_rust_error(rust_error);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("Expected"));
        assert!(error.message.contains("Number")); // f32 should be converted to Number
        assert!(error.message.contains("Text")); // &str should be converted to Text
    }

    #[test]
    fn test_wrong_number_of_arguments_translation() {
        let translator = get_error_translator();
        
        let rust_error = "error[E0061]: this function takes 2 arguments but 1 argument was supplied
  --> src/main.rs:5:5
   |
5  |     Audio.analyze_fft(audio_data);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected 2 arguments, found 1";
        
        let translated = translator.translate_rust_error(rust_error);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("wrong number of arguments"));
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_trait_bound_error_translation() {
        let translator = get_error_translator();
        
        let rust_error = "error[E0277]: the trait bound `Text: AudioProcessor` is not satisfied
  --> src/main.rs:8:5
   |
8  |     text_data.process_audio();
   |     ^^^^^^^^^ the trait `AudioProcessor` is not implemented for `Text`";
        
        let translated = translator.translate_rust_error(rust_error);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("operation isn't supported"));
        assert!(error.suggestions.len() >= 2);
    }

    #[test]  
    fn test_use_after_move_translation() {
        let translator = get_error_translator();
        
        let rust_error = "error[E0382]: use of moved value: `audio_stream`
  --> src/main.rs:10:5
   |
9  |     let processed = audio_stream.apply_reverb();
   |                     ------------ value moved here
10 |     let analyzed = audio_stream.analyze_fft();
   |                    ^^^^^^^^^^^^ value used here after move";
        
        let translated = translator.translate_rust_error(rust_error);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("already used"));
        assert!(error.message.contains("audio_stream"));
    }

    #[test]
    fn test_cannot_assign_twice_translation() {
        let translator = get_error_translator();
        
        let rust_error = "error[E0384]: cannot assign twice to immutable variable `frequency`
  --> src/main.rs:8:5
   |
6  |     let frequency = 440.0;
   |         --------- first assignment to `frequency`
7  |     // ... some code
8  |     frequency = 880.0;
   |     ^^^^^^^^^^^^^^^^^ cannot assign twice to immutable variable";
        
        let translated = translator.translate_rust_error(rust_error);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("cannot be changed"));
        assert!(error.message.contains("frequency"));
        assert!(error.suggestions.iter().any(|s| s.contains("mut")));
    }

    #[test]
    fn test_panic_translation() {
        let translator = get_error_translator();
        
        let panic_message = "thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 5'";
        
        let translated = translator.translate_rust_error(panic_message);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("unexpected error"));
    }

    #[test]
    fn test_index_out_of_bounds_translation() {
        let translator = get_error_translator();
        
        let error_message = "index out of bounds: the len is 8 but the index is 10";
        
        let translated = translator.translate_rust_error(error_message);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("access data that doesn't exist"));
        assert!(error.suggestions.iter().any(|s| s.contains("len()")));
    }

    #[test]
    fn test_division_by_zero_translation() {
        let translator = get_error_translator();
        
        let error_message = "attempt to divide by zero";
        
        let translated = translator.translate_rust_error(error_message);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("Cannot divide by zero"));
        assert!(error.suggestions.iter().any(|s| s.contains("divisor != 0")));
    }

    #[test]
    fn test_webassembly_error_translation() {
        let translator = get_error_translator();
        
        let wasm_error = "compilation failed for wasm32-unknown-unknown target";
        
        let translated = translator.translate_rust_error(wasm_error);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("web browser"));
        assert!(error.suggestions.iter().any(|s| s.contains("--target native")));
    }

    #[test]
    fn test_nom_parser_error_translation() {
        let translator = get_error_translator();
        
        let nom_error = "incomplete input: expected closing brace";
        
        let translated = translator.translate_rust_error(nom_error);
        assert!(translated.is_some());
        
        let error = translated.unwrap();
        assert!(error.message.contains("Incomplete or unexpected syntax"));
        assert!(error.suggestions.iter().any(|s| s.contains("brackets")));
    }
}