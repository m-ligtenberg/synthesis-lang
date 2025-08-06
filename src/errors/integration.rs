/// Integration utilities for the error translation system
/// 
/// This module provides helper functions and macros to easily integrate
/// the error translation system throughout the codebase.

use super::{SynthesisError, ErrorKind, get_error_translator};

/// Helper macro to create context-aware Synthesis errors from any error type
#[macro_export]
macro_rules! synthesis_error_from {
    ($err:expr, $context:expr) => {{
        let error_str = $err.to_string();
        
        // Try to translate using the error translator first
        if let Some(translated) = $crate::errors::get_error_translator().translate_rust_error(&error_str) {
            translated.with_suggestion(format!("Context: {}", $context))
        } else {
            // Fallback to generic error with context
            $crate::errors::SynthesisError::new(
                $crate::errors::ErrorKind::CompilationFailed,
                format!("Error in {}: {}", $context, error_str)
            )
            .with_suggestion("Try simplifying the code around this area")
            .with_suggestion("Check for syntax errors or type issues")
        }
    }};
}

/// Helper function to safely execute Rust operations and translate errors
pub fn execute_with_translation<T, F>(operation: F, context: &str) -> crate::Result<T>
where
    F: FnOnce() -> anyhow::Result<T>,
{
    match operation() {
        Ok(result) => Ok(result),
        Err(err) => {
            let error_str = err.to_string();
            
            // Try error translation first
            if let Some(translated) = get_error_translator().translate_rust_error(&error_str) {
                Err(translated.with_suggestion(format!("Context: {}", context)))
            } else {
                // Fallback error creation
                Err(SynthesisError::new(
                    ErrorKind::CompilationFailed,
                    format!("Error in {}: {}", context, error_str)
                )
                .with_suggestion("Try simplifying the code in this area")
                .with_suggestion("Check for syntax or type compatibility issues"))
            }
        }
    }
}

/// Helper function to translate standard library errors to Synthesis errors
pub fn translate_std_error<E: std::error::Error>(error: E, context: &str) -> SynthesisError {
    let error_str = error.to_string();
    
    // Try specific patterns for common std errors
    if error_str.contains("No such file") || error_str.contains("not found") {
        SynthesisError::file_not_found(context)
    } else if error_str.contains("Permission denied") {
        SynthesisError::new(
            ErrorKind::PermissionDenied,
            format!("Permission denied: {}", context)
        )
        .with_suggestion("Check file permissions")
        .with_suggestion("Try running with appropriate privileges")
    } else if error_str.contains("parse") {
        SynthesisError::new(
            ErrorKind::SyntaxError,
            format!("Parse error in {}", context)
        )
        .with_suggestion("Check syntax and format")
        .with_suggestion("Look for typos or missing punctuation")
    } else {
        // Try the Rust error translator
        if let Some(translated) = get_error_translator().translate_rust_error(&error_str) {
            translated.with_suggestion(format!("Context: {}", context))
        } else {
            // Generic fallback
            SynthesisError::new(
                ErrorKind::CompilationFailed,
                format!("Error in {}: {}", context, error_str)
            )
            .with_suggestion("Check the operation you're trying to perform")
        }
    }
}

/// Create user-friendly errors for compilation context
pub fn compilation_error(phase: &str, details: &str) -> SynthesisError {
    let error_str = format!("{}: {}", phase, details);
    
    // Try translation first
    if let Some(translated) = get_error_translator().translate_rust_error(details) {
        return translated;
    }
    
    // Phase-specific error handling
    match phase {
        "parsing" => SynthesisError::new(
            ErrorKind::SyntaxError,
            "There's a syntax error in your Synthesis code"
        )
        .with_suggestion("Check for missing brackets, parentheses, or punctuation")
        .with_suggestion("Make sure all import statements are at the top")
        .with_docs("https://synthesis-lang.org/docs/syntax"),
        
        "type_checking" => SynthesisError::type_inference_error(
            "automatic type detection failed"
        ),
        
        "code_generation" => SynthesisError::new(
            ErrorKind::CodeGenerationFailed,
            "Failed to generate executable code"
        )
        .with_suggestion("Try using simpler expressions")
        .with_suggestion("Some advanced features may not be available for your target")
        .with_docs("https://synthesis-lang.org/docs/compilation"),
        
        "optimization" => SynthesisError::new(
            ErrorKind::OptimizationFailed,
            "Code optimization failed"
        )
        .with_suggestion("Try disabling optimizations with --no-optimize")
        .with_suggestion("Break complex expressions into simpler parts"),
        
        _ => SynthesisError::new(
            ErrorKind::CompilationFailed,
            format!("Compilation failed during {}", phase)
        )
        .with_suggestion("Try using --debug flag for more information")
        .with_suggestion("Simplify your code to isolate the issue")
    }
}

/// Macro to easily catch and translate Rust panics
#[macro_export]
macro_rules! catch_rust_panic {
    ($operation:expr, $context:expr) => {{
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $operation))
            .map_err(|_| {
                $crate::errors::SynthesisError::new(
                    $crate::errors::ErrorKind::CompilationFailed,
                    format!("Internal error in {}", $context)
                )
                .with_suggestion("Try simplifying your code") 
                .with_suggestion("This might be a bug - please report it if it persists")
            })?
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_error_context() {
        let error = compilation_error("parsing", "unexpected token");
        assert!(matches!(error.kind, ErrorKind::SyntaxError));
        assert!(error.message.contains("syntax error"));
    }

    #[test]
    fn test_std_error_translation() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let translated = translate_std_error(io_error, "loading config file");
        assert!(matches!(translated.kind, ErrorKind::FileNotFound));
    }

    #[test]
    fn test_execute_with_translation_success() {
        let result = execute_with_translation(|| Ok(42), "test operation");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_execute_with_translation_error() {
        let result = execute_with_translation(
            || Err(anyhow::anyhow!("test error")), 
            "test operation"
        );
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.suggestions.iter().any(|s| s.contains("Context: test operation")));
    }

    #[test]
    fn test_compilation_error_phases() {
        let parse_error = compilation_error("parsing", "syntax error");
        assert!(matches!(parse_error.kind, ErrorKind::SyntaxError));
        
        let type_error = compilation_error("type_checking", "type mismatch");
        assert!(matches!(type_error.kind, ErrorKind::TypeInferenceError));
        
        let codegen_error = compilation_error("code_generation", "failed to generate");
        assert!(matches!(codegen_error.kind, ErrorKind::CodeGenerationFailed));
        
        let opt_error = compilation_error("optimization", "optimization failed");
        assert!(matches!(opt_error.kind, ErrorKind::OptimizationFailed));
    }

    #[test] 
    fn test_stream_specific_errors() {
        let stream_error = SynthesisError::stream_connection_error("audio_input", "graphics_output");
        assert!(matches!(stream_error.kind, ErrorKind::StreamConnectionError));
        assert!(stream_error.message.contains("audio_input"));
        assert!(stream_error.message.contains("graphics_output"));
        
        let underrun_error = SynthesisError::stream_buffer_underrun("main_audio");
        assert!(matches!(underrun_error.kind, ErrorKind::StreamBufferUnderrun));
        assert!(underrun_error.suggestions.iter().any(|s| s.contains("buffer size")));
    }

    #[test]
    fn test_real_time_errors() {
        let rt_error = SynthesisError::real_time_violation("audio_processing", 2.5);
        assert!(matches!(rt_error.kind, ErrorKind::RealTimeViolation));
        assert!(rt_error.message.contains("2.50ms"));
        
        let buffer_error = SynthesisError::buffer_size_error(4096, "64, 128, 256, 512, 1024");
        assert!(matches!(buffer_error.kind, ErrorKind::BufferSizeError));
        assert!(buffer_error.message.contains("4096"));
    }

    #[test]
    fn test_invalid_stream_format() {
        let format_error = SynthesisError::invalid_stream_format("audio_stream", "48kHz 16-bit", "44.1kHz 24-bit");
        assert!(matches!(format_error.kind, ErrorKind::InvalidStreamFormat));
        assert!(format_error.message.contains("48kHz"));
        assert!(format_error.message.contains("44.1kHz"));
    }

    #[test]
    fn test_context_aware_suggestions() {
        use crate::errors::generate_context_suggestions;
        
        let import_suggestions = generate_context_suggestions(
            &ErrorKind::SyntaxError, 
            "error parsing import statement"
        );
        assert!(import_suggestions.iter().any(|s| s.contains("import ModuleName")));
        
        let audio_suggestions = generate_context_suggestions(
            &ErrorKind::AudioDeviceError,
            "audio device connection failed"
        );
        assert!(audio_suggestions.iter().any(|s| s.contains("audio device")));
        
        let type_suggestions = generate_context_suggestions(
            &ErrorKind::TypeMismatch,
            "Expected Number but found Text"
        );
        assert!(type_suggestions.iter().any(|s| s.contains("Number.parse")));
    }
}