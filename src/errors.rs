use std::fmt;
use std::process::{Command, Stdio};
use regex::Regex;

pub mod integration;

/// Synthesis Language Error System
/// All errors are presented in creative, user-friendly language
#[derive(Debug, Clone)]
pub struct SynthesisError {
    pub kind: ErrorKind,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub suggestions: Vec<String>,
    pub related_docs: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    // Parse errors
    SyntaxError,
    UnexpectedToken,
    MissingToken,
    InvalidExpression,
    
    // Semantic errors  
    UnknownModule,
    UnknownFunction,
    TypeMismatch,
    InvalidStreamConnection,
    TypeInferenceError,
    MissingTypeAnnotation,
    TraitBoundError,
    
    // Runtime errors
    AudioDeviceError,
    GraphicsContextError,
    StreamBufferOverflow,
    PerformanceConstraintViolation,
    
    // Compilation errors
    CompilationFailed,
    OptimizationFailed,
    CodeGenerationFailed,
    RustCompilerError,
    
    // System errors
    FileNotFound,
    PermissionDenied,
    OutOfMemory,
    
    // Stream-specific errors
    StreamConnectionError,
    StreamBufferUnderrun,
    StreamTimeout,
    InvalidStreamFormat,
    
    // Performance and real-time errors
    RealTimeViolation,
    BufferSizeError,
    SampleRateError,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub filename: String,
}

impl SynthesisError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            location: None,
            suggestions: Vec::new(),
            related_docs: None,
        }
    }

    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    pub fn with_docs(mut self, docs_url: impl Into<String>) -> Self {
        self.related_docs = Some(docs_url.into());
        self
    }

    // Create user-friendly error messages
    pub fn syntax_error(message: impl Into<String>, line: usize, column: usize, filename: impl Into<String>) -> Self {
        Self::new(ErrorKind::SyntaxError, message)
            .with_location(SourceLocation {
                line,
                column,
                filename: filename.into(),
            })
            .with_suggestion("Check your syntax - Synthesis uses clean, readable patterns")
            .with_docs("https://synthesis-lang.org/docs/syntax")
    }

    pub fn unknown_module(module_name: impl Into<String>) -> Self {
        let module = module_name.into();
        Self::new(
            ErrorKind::UnknownModule, 
            format!("Unknown module '{}'", module)
        )
        .with_suggestion(format!("Did you mean one of these? Audio, Graphics, GUI, Hardware, Math, Time"))
        .with_suggestion("Make sure to import the module at the top of your file")
        .with_docs("https://synthesis-lang.org/docs/modules")
    }

    pub fn unknown_function(module: impl Into<String>, function: impl Into<String>) -> Self {
        let mod_name = module.into();
        let func_name = function.into();
        
        let suggestions = match mod_name.as_str() {
            "Audio" => vec![
                "mic_input()", "analyze_fft(audio, bands)", "beat_detect(audio)",
                "apply_reverb(audio, room_size)", "synthesize_sine(frequency)"
            ],
            "Graphics" => vec![
                "clear(color)", "plasma(speed, palette)", "starfield(count, speed)",
                "draw_circle(x, y, radius)", "flash(color, intensity)"
            ],
            "GUI" => vec![
                "window(title, content)", "slider(label, min, max, default)",
                "button(label)", "dropdown(label, options, default)"
            ],
            _ => vec!["Check the documentation for available functions"]
        };

        Self::new(
            ErrorKind::UnknownFunction,
            format!("Function '{}' doesn't exist in module '{}'", func_name, mod_name)
        )
        .with_suggestions(suggestions.into_iter().map(String::from).collect())
        .with_docs(format!("https://synthesis-lang.org/docs/api/{}", mod_name.to_lowercase()))
    }

    pub fn type_mismatch(expected: impl Into<String>, found: impl Into<String>) -> Self {
        let exp = expected.into();
        let found = found.into();
        
        Self::new(
            ErrorKind::TypeMismatch,
            format!("Expected {} but found {}", exp, found)
        )
        .with_suggestion("Synthesis automatically converts between compatible types")
        .with_suggestion("Try using explicit conversion functions if needed")
        .with_docs("https://synthesis-lang.org/docs/types")
    }

    pub fn audio_device_error(details: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::AudioDeviceError,
            format!("Audio system error: {}", details.into())
        )
        .with_suggestion("Check that your audio device is working and not used by other apps")
        .with_suggestion("Try adjusting buffer size with --buffer-size option")
        .with_docs("https://synthesis-lang.org/docs/audio-troubleshooting")
    }

    pub fn stream_overflow(stream_name: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::StreamBufferOverflow,
            format!("Stream '{}' buffer overflow - data is coming in faster than it can be processed", stream_name.into())
        )
        .with_suggestion("Increase buffer size or optimize your processing loop")
        .with_suggestion("Consider using stream.throttle() to control data flow")
        .with_docs("https://synthesis-lang.org/docs/streams")
    }

    pub fn performance_violation(function: impl Into<String>, latency_ms: f32, max_ms: f32) -> Self {
        Self::new(
            ErrorKind::PerformanceConstraintViolation,
            format!("Function '{}' took {:.2}ms but real-time limit is {:.2}ms", 
                function.into(), latency_ms, max_ms)
        )
        .with_suggestion("Use lighter processing or increase buffer size")
        .with_suggestion("Enable optimization with -O creative flag")
        .with_docs("https://synthesis-lang.org/docs/performance")
    }

    pub fn compilation_failed(details: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::CompilationFailed,
            format!("Compilation failed: {}", details.into())
        )
        .with_suggestion("Check your syntax and try again")
        .with_suggestion("Use --debug flag for more detailed information")
    }

    pub fn type_inference_error(context: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::TypeInferenceError,
            format!("Synthesis couldn't figure out the type automatically: {}", context.into())
        )
        .with_suggestion("Add a type hint like `let result: Audio = ...`")
        .with_suggestion("Use explicit function calls to clarify intent")
        .with_docs("https://synthesis-lang.org/docs/types#type-hints")
    }

    pub fn missing_type_annotation(variable: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::MissingTypeAnnotation,
            format!("Variable '{}' needs a type hint", variable.into())
        )
        .with_suggestion("Add a type like: `let variable: Audio = ...`")
        .with_suggestion("Common types: Audio, Graphics, Number, Text, Stream")
        .with_docs("https://synthesis-lang.org/docs/types")
    }

    pub fn trait_bound_error(type_name: impl Into<String>, missing_trait: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::TraitBoundError,
            format!("Type '{}' doesn't support the operation you're trying to perform", type_name.into())
        )
        .with_suggestion("Check if you're using the right type for this operation")
        .with_suggestion("Some operations only work with specific data types")
        .with_docs("https://synthesis-lang.org/docs/types#operations")
    }

    pub fn file_not_found(filename: impl Into<String>) -> Self {
        let file = filename.into();
        Self::new(
            ErrorKind::FileNotFound,
            format!("Cannot find file '{}'", file)
        )
        .with_suggestion("Make sure the file exists and the path is correct")
        .with_suggestion("Synthesis files should have .syn extension")
    }

    pub fn stream_connection_error(from_stream: impl Into<String>, to_stream: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::StreamConnectionError,
            format!("Cannot connect stream '{}' to '{}'", from_stream.into(), to_stream.into())
        )
        .with_suggestion("Check that the stream types are compatible")
        .with_suggestion("Audio streams connect to Audio, Graphics to Graphics, etc.")
        .with_suggestion("Use conversion functions if you need to change stream types")
        .with_docs("https://synthesis-lang.org/docs/streams#connections")
    }

    pub fn stream_buffer_underrun(stream_name: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::StreamBufferUnderrun,
            format!("Audio stream '{}' buffer underrun - not enough data to maintain real-time", stream_name.into())
        )
        .with_suggestion("Increase buffer size with --buffer-size option")
        .with_suggestion("Optimize your processing code for better performance") 
        .with_suggestion("Reduce sample rate if high quality isn't needed")
        .with_docs("https://synthesis-lang.org/docs/performance#audio-buffers")
    }

    pub fn real_time_violation(operation: impl Into<String>, time_taken_ms: f32) -> Self {
        Self::new(
            ErrorKind::RealTimeViolation,
            format!("Operation '{}' took {:.2}ms which violates real-time constraints", operation.into(), time_taken_ms)
        )
        .with_suggestion("Use lighter processing or increase buffer size")
        .with_suggestion("Enable optimizations with -O creative flag")
        .with_suggestion("Consider using background processing for heavy operations")
        .with_docs("https://synthesis-lang.org/docs/performance#real-time")
    }

    pub fn invalid_stream_format(stream_name: impl Into<String>, expected: impl Into<String>, found: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::InvalidStreamFormat,
            format!("Stream '{}' has wrong format: expected {} but found {}", 
                stream_name.into(), expected.into(), found.into())
        )
        .with_suggestion("Check that your audio device supports the requested format")
        .with_suggestion("Try different sample rates: 44100, 48000, 96000")
        .with_suggestion("Verify bit depth compatibility (16-bit, 24-bit, 32-bit)")
        .with_docs("https://synthesis-lang.org/docs/audio#formats")
    }

    pub fn buffer_size_error(requested: usize, supported: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::BufferSizeError,
            format!("Requested buffer size {} is not supported", requested)
        )
        .with_suggestion(format!("Supported buffer sizes: {}", supported.into()))
        .with_suggestion("Common sizes: 64, 128, 256, 512, 1024 samples")
        .with_suggestion("Smaller buffers = lower latency but higher CPU usage")
        .with_docs("https://synthesis-lang.org/docs/audio#buffer-sizes")
    }
}

impl fmt::Display for SynthesisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Friendly header with emoji
        let emoji = match self.kind {
            ErrorKind::SyntaxError | ErrorKind::UnexpectedToken | ErrorKind::MissingToken | ErrorKind::InvalidExpression => "🎵",
            ErrorKind::UnknownModule | ErrorKind::UnknownFunction => "🔍",
            ErrorKind::TypeMismatch | ErrorKind::TypeInferenceError | ErrorKind::MissingTypeAnnotation => "🔄",
            ErrorKind::TraitBoundError => "🔗",
            ErrorKind::AudioDeviceError | ErrorKind::BufferSizeError | ErrorKind::SampleRateError => "🎧",
            ErrorKind::GraphicsContextError => "🎨",
            ErrorKind::StreamBufferOverflow | ErrorKind::StreamBufferUnderrun => "🌊",
            ErrorKind::StreamConnectionError | ErrorKind::InvalidStreamConnection => "🔌",
            ErrorKind::StreamTimeout | ErrorKind::InvalidStreamFormat => "⏱️",
            ErrorKind::PerformanceConstraintViolation | ErrorKind::RealTimeViolation => "⚡",
            ErrorKind::CompilationFailed | ErrorKind::RustCompilerError | ErrorKind::CodeGenerationFailed => "🔧",
            ErrorKind::OptimizationFailed => "⚙️",
            ErrorKind::FileNotFound => "📁",
            ErrorKind::PermissionDenied => "🔒",
            ErrorKind::OutOfMemory => "💾",
            _ => "❗",
        };

        writeln!(f, "{} Synthesis Error: {}", emoji, self.message)?;

        // Show location if available
        if let Some(loc) = &self.location {
            writeln!(f, "   at {}:{}:{}", loc.filename, loc.line, loc.column)?;
        }

        // Show suggestions
        if !self.suggestions.is_empty() {
            writeln!(f, "\n💡 Suggestions:")?;
            for suggestion in &self.suggestions {
                writeln!(f, "   • {}", suggestion)?;
            }
        }

        // Show documentation link
        if let Some(docs) = &self.related_docs {
            writeln!(f, "\n📚 Learn more: {}", docs)?;
        }

        Ok(())
    }
}

impl std::error::Error for SynthesisError {}

// Convert from common error types while maintaining Synthesis branding
impl From<std::io::Error> for SynthesisError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                SynthesisError::file_not_found("File not found")
            }
            std::io::ErrorKind::PermissionDenied => {
                SynthesisError::new(
                    ErrorKind::PermissionDenied,
                    "Permission denied - cannot access file"
                )
                .with_suggestion("Check file permissions")
                .with_suggestion("Try running as administrator if needed")
            }
            _ => {
                SynthesisError::new(
                    ErrorKind::FileNotFound,
                    format!("File system error: {}", err)
                )
            }
        }
    }
}

// Handle graphics-related errors
impl From<winit::error::OsError> for SynthesisError {
    fn from(err: winit::error::OsError) -> Self {
        SynthesisError::new(
            ErrorKind::GraphicsContextError, 
            format!("Window creation error: {}", err)
        )
        .with_suggestion("Ensure your graphics drivers are up to date")
        .with_suggestion("Check system graphics configuration")
    }
}

// Handle WebGPU create surface errors
impl From<wgpu::CreateSurfaceError> for SynthesisError {
    fn from(err: wgpu::CreateSurfaceError) -> Self {
        SynthesisError::new(
            ErrorKind::GraphicsContextError,
            format!("GPU surface creation error: {}", err)
        )
        .with_suggestion("Check graphics card compatibility")
        .with_suggestion("Update graphics drivers")
    }
}

// Handle device request errors
impl From<wgpu::RequestDeviceError> for SynthesisError {
    fn from(err: wgpu::RequestDeviceError) -> Self {
        SynthesisError::new(
            ErrorKind::GraphicsContextError,
            format!("GPU device request error: {}", err)
        )
        .with_suggestion("Ensure your graphics card supports WebGPU")
        .with_suggestion("Update graphics drivers")
    }
}

// Handle OSC errors
impl From<rosc::OscError> for SynthesisError {
    fn from(err: rosc::OscError) -> Self {
        SynthesisError::new(
            ErrorKind::AudioDeviceError,
            format!("OSC communication error: {}", err)
        )
        .with_suggestion("Check network configuration")
        .with_suggestion("Verify OSC server/client settings")
    }
}

// Handle audio system errors
impl From<cpal::DefaultStreamConfigError> for SynthesisError {
    fn from(err: cpal::DefaultStreamConfigError) -> Self {
        SynthesisError::audio_device_error(format!("Audio configuration error: {}", err))
    }
}

impl From<cpal::BuildStreamError> for SynthesisError {
    fn from(err: cpal::BuildStreamError) -> Self {
        SynthesisError::audio_device_error(format!("Audio stream error: {}", err))
    }
}

impl From<cpal::StreamError> for SynthesisError {
    fn from(err: cpal::StreamError) -> Self {
        SynthesisError::audio_device_error(format!("Audio stream error: {}", err))
    }
}

impl From<cpal::PlayStreamError> for SynthesisError {
    fn from(err: cpal::PlayStreamError) -> Self {
        SynthesisError::audio_device_error(format!("Audio playback error: {}", err))
    }
}

// Handle graphics surface errors
impl From<wgpu::SurfaceError> for SynthesisError {
    fn from(err: wgpu::SurfaceError) -> Self {
        SynthesisError::new(
            ErrorKind::GraphicsContextError,
            format!("Graphics surface error: {}", err)
        )
        .with_suggestion("Check graphics card and driver compatibility")
        .with_suggestion("Try restarting the application")
    }
}

// Handle serial port errors
impl From<serialport::Error> for SynthesisError {
    fn from(err: serialport::Error) -> Self {
        SynthesisError::new(
            ErrorKind::AudioDeviceError,
            format!("Hardware connection error: {}", err)
        )
        .with_suggestion("Check that your hardware is connected")
        .with_suggestion("Verify correct port and baud rate")
    }
}

// Handle MIDI errors
impl From<midir::InitError> for SynthesisError {
    fn from(err: midir::InitError) -> Self {
        SynthesisError::audio_device_error(format!("MIDI initialization error: {}", err))
    }
}

impl From<midir::ConnectError<midir::MidiInput>> for SynthesisError {
    fn from(err: midir::ConnectError<midir::MidiInput>) -> Self {
        SynthesisError::audio_device_error(format!("MIDI input connection error: {}", err))
    }
}

impl From<midir::ConnectError<midir::MidiOutput>> for SynthesisError {
    fn from(err: midir::ConnectError<midir::MidiOutput>) -> Self {
        SynthesisError::audio_device_error(format!("MIDI output connection error: {}", err))
    }
}

impl From<midir::SendError> for SynthesisError {
    fn from(err: midir::SendError) -> Self {
        SynthesisError::audio_device_error(format!("MIDI send error: {}", err))
    }
}

// Never expose internal Rust errors to users
impl From<anyhow::Error> for SynthesisError {
    fn from(err: anyhow::Error) -> Self {
        translate_anyhow_error(&err)
    }
}

// Handle Rust panics gracefully
impl From<Box<dyn std::any::Any + Send>> for SynthesisError {
    fn from(panic_info: Box<dyn std::any::Any + Send>) -> Self {
        let panic_message = if let Some(s) = panic_info.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.downcast_ref::<String>() {
            s.clone()
        } else {
            "Internal error occurred".to_string()
        };

        // Try to translate the panic message
        if let Some(translated) = get_error_translator().translate_rust_error(&panic_message) {
            return translated;
        }

        // Fallback to user-friendly panic handling
        SynthesisError::new(
            ErrorKind::CompilationFailed,
            "An unexpected error occurred while processing your Synthesis code"
        )
        .with_suggestion("Try simplifying your code to isolate the issue")
        .with_suggestion("This might be a bug - please report it if it persists")
        .with_suggestion("Use --debug flag for more technical details")
        .with_docs("https://synthesis-lang.org/docs/troubleshooting#unexpected-errors")
    }
}

pub type Result<T> = std::result::Result<T, SynthesisError>;

/// Rust Error Translation System
/// Intercepts and translates Rust compiler errors into user-friendly Synthesis errors
pub struct RustErrorTranslator {
    error_patterns: Vec<ErrorPattern>,
}

struct ErrorPattern {
    regex: Regex,
    error_kind: ErrorKind,
    message_transform: fn(&str) -> String,
    suggestions: Vec<String>,
    docs_url: Option<String>,
}

impl RustErrorTranslator {
    pub fn new() -> Self {
        let mut translator = Self {
            error_patterns: Vec::new(),
        };
        translator.register_patterns();
        translator
    }

    fn register_patterns(&mut self) {
        // E0283: type annotations needed
        self.add_pattern(
            r"E0283.*type annotations needed",
            ErrorKind::TypeInferenceError,
            |_| "Synthesis needs a hint about what type you want here".to_string(),
            vec![
                "Add a type annotation like: `let value: Audio = ...`".to_string(),
                "Use more specific function calls to help with type inference".to_string(),
                "Common types: Audio, Graphics, Number, Text, Stream".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/types#inference".to_string()),
        );

        // Type annotations needed (general case)
        self.add_pattern(
            r"type annotations needed.*cannot infer type",
            ErrorKind::MissingTypeAnnotation,
            |_| "Your code needs a type hint to work properly".to_string(),
            vec![
                "Try adding `: TypeName` after your variable".to_string(),
                "Example: `let audio_data: Audio = mic_input()`".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/types".to_string()),
        );

        // Trait bound errors
        self.add_pattern(
            r"the trait bound.*is not satisfied",
            ErrorKind::TraitBoundError,
            |_| "This type doesn't support the operation you're trying to use".to_string(),
            vec![
                "Check if you're using compatible types".to_string(),
                "Some operations only work with specific data types".to_string(),
                "Try using conversion functions if needed".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/types#compatibility".to_string()),
        );

        // Method not found
        self.add_pattern(
            r"no method named.*found for type",
            ErrorKind::UnknownFunction,
            |msg| {
                if let Some(method_match) = Regex::new(r"no method named `([^`]+)`").unwrap().captures(msg) {
                    format!("The method '{}' doesn't exist for this type", &method_match[1])
                } else {
                    "Method not found for this type".to_string()
                }
            },
            vec![
                "Check the spelling of the method name".to_string(),
                "Make sure you're calling methods on the right type".to_string(),
                "Look at the documentation for available methods".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/api".to_string()),
        );

        // Cannot find value/variable
        self.add_pattern(
            r"cannot find value `([^`]+)` in this scope",
            ErrorKind::UnknownFunction,
            |msg| {
                if let Some(var_match) = Regex::new(r"cannot find value `([^`]+)`").unwrap().captures(msg) {
                    format!("Variable or function '{}' is not defined", &var_match[1])
                } else {
                    "Undefined variable or function".to_string()
                }
            },
            vec![
                "Check the spelling of the name".to_string(),
                "Make sure the variable is defined before using it".to_string(),
                "For functions, check if you need to import a module".to_string(),
            ],
            None,
        );

        // Cannot find module
        self.add_pattern(
            r"cannot find.*module.*in this scope",
            ErrorKind::UnknownModule,
            |msg| {
                if let Some(mod_match) = Regex::new(r"cannot find.*`([^`]+)`").unwrap().captures(msg) {
                    format!("Module '{}' is not available", &mod_match[1])
                } else {
                    "Module not found".to_string()
                }
            },
            vec![
                "Available modules: Audio, Graphics, GUI, Hardware, Math, Time".to_string(),
                "Make sure to import the module: `import ModuleName`".to_string(),
                "Check the module name spelling".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/modules".to_string()),
        );

        // Mismatched types
        self.add_pattern(
            r"mismatched types.*expected.*found",
            ErrorKind::TypeMismatch,
            |msg| {
                // Try to extract type information from error message
                if let Some(types) = extract_type_mismatch(msg) {
                    format!("Expected {} but got {}", types.0, types.1)
                } else {
                    "Type mismatch - the types don't work together".to_string()
                }
            },
            vec![
                "Synthesis usually converts types automatically".to_string(),
                "Try using explicit conversion if needed".to_string(),
                "Check if you're using the right function for the data type".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/types#conversion".to_string()),
        );

        // E0061: Wrong number of function arguments
        self.add_pattern(
            r"E0061.*takes \d+ argument.*but \d+ argument.*supplied",
            ErrorKind::InvalidExpression,
            |msg| {
                if let Some(func_match) = Regex::new(r"function `([^`]+)`").unwrap().captures(msg) {
                    format!("Function '{}' called with wrong number of arguments", &func_match[1])
                } else {
                    "Function called with wrong number of arguments".to_string()
                }
            },
            vec![
                "Check the function documentation for correct parameters".to_string(),
                "Make sure you're passing the right number of arguments".to_string(),
                "Some parameters might be optional or have default values".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/functions".to_string()),
        );

        // E0277: Trait bound not satisfied  
        self.add_pattern(
            r"E0277.*the trait bound.*is not satisfied",
            ErrorKind::TraitBoundError,
            |_| "This operation isn't supported for this type of data".to_string(),
            vec![
                "Check if you're using compatible data types".to_string(),
                "Some operations only work with specific types like Numbers or Audio".to_string(),
                "Try converting your data to the right type first".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/types#operations".to_string()),
        );

        // E0382: Use after move (borrow checker)
        self.add_pattern(
            r"E0382.*use of moved value",
            ErrorKind::InvalidExpression,
            |msg| {
                if let Some(var_match) = Regex::new(r"value `([^`]+)`").unwrap().captures(msg) {
                    format!("Variable '{}' was already used and can't be used again", &var_match[1])
                } else {
                    "Variable was already used and can't be used again".to_string()
                }
            },
            vec![
                "In Synthesis, some operations consume their inputs".to_string(),
                "Try using .clone() if you need to use the same data multiple times".to_string(),
                "Consider restructuring your code to avoid reusing consumed values".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/variables#ownership".to_string()),
        );

        // E0384: Cannot assign twice to immutable variable
        self.add_pattern(
            r"E0384.*cannot assign twice to immutable variable",
            ErrorKind::InvalidExpression,
            |msg| {
                if let Some(var_match) = Regex::new(r"variable `([^`]+)`").unwrap().captures(msg) {
                    format!("Variable '{}' cannot be changed after it's set", &var_match[1])
                } else {
                    "Variable cannot be changed after it's set".to_string()
                }
            },
            vec![
                "Use 'mut' keyword when creating the variable: `let mut variable = ...`".to_string(),
                "Variables in Synthesis are unchangeable by default for safety".to_string(),
                "Consider using a new variable name if you don't need mutability".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/variables#mutability".to_string()),
        );

        // nom parser errors (common in parsing)
        self.add_pattern(
            r"incomplete|incomplete input|take|tag",
            ErrorKind::SyntaxError,
            |_| "Incomplete or unexpected syntax in your Synthesis code".to_string(),
            vec![
                "Check that all brackets, parentheses, and braces are properly closed".to_string(),
                "Make sure function calls have all required parameters".to_string(),
                "Verify that import statements are complete".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/syntax".to_string()),
        );

        // WebAssembly compilation errors
        self.add_pattern(
            r"wasm32|WebAssembly|WASM",
            ErrorKind::CodeGenerationFailed,
            |_| "Issue compiling for web browser (WebAssembly)".to_string(),
            vec![
                "Some features may not be available in web browsers".to_string(),
                "Try compiling for native desktop instead: `--target native`".to_string(),
                "Check that all modules you're using support WebAssembly".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/web-export".to_string()),
        );

        // Panic-related error patterns
        self.add_pattern(
            r"panic|panicked|thread.*panicked",
            ErrorKind::CompilationFailed,
            |_| "An unexpected error occurred while processing your code".to_string(),
            vec![
                "This is likely a bug in Synthesis - please report it".to_string(),
                "Try simplifying your code to isolate the issue".to_string(),
                "Use --debug flag for more technical details".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/troubleshooting#bugs".to_string()),
        );

        // Index out of bounds errors
        self.add_pattern(
            r"index out of bounds|slice index|range end index.*out of range",
            ErrorKind::InvalidExpression,
            |_| "Trying to access data that doesn't exist".to_string(),
            vec![
                "Check that your list or array has enough items".to_string(),
                "Array indices start at 0, not 1".to_string(),
                "Use .len() to check the size before accessing elements".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/data#arrays".to_string()),
        );

        // Division by zero
        self.add_pattern(
            r"divide by zero|division by zero|attempt to divide.*by zero",
            ErrorKind::InvalidExpression,
            |_| "Cannot divide by zero".to_string(),
            vec![
                "Check that your divisor is not zero before dividing".to_string(),
                "Use conditional logic: `if divisor != 0 { result = a / divisor }`".to_string(),
                "Consider using small non-zero values instead of exact zero".to_string(),
            ],
            None,
        );

        // Unwrap on None/Err panics  
        self.add_pattern(
            r"unwrap.*None|unwrap.*Err",
            ErrorKind::CompilationFailed,
            |_| "Expected value was missing or invalid".to_string(),
            vec![
                "This usually indicates a problem with data flow in your code".to_string(),
                "Check that all required inputs are provided".to_string(),
                "Verify that file paths and device names are correct".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/troubleshooting#missing-values".to_string()),
        );

        // Compilation errors (general)
        self.add_pattern(
            r"error\[E\d+\]:",
            ErrorKind::RustCompilerError,
            |_| "Something went wrong during compilation".to_string(),
            vec![
                "Check your syntax for any typos or missing parts".to_string(),
                "Try simplifying your code to isolate the issue".to_string(),
                "Use --debug flag for more detailed error information".to_string(),
            ],
            Some("https://synthesis-lang.org/docs/troubleshooting".to_string()),
        );
    }

    fn add_pattern(
        &mut self,
        pattern: &str,
        kind: ErrorKind,
        transform: fn(&str) -> String,
        suggestions: Vec<String>,
        docs_url: Option<String>,
    ) {
        if let Ok(regex) = Regex::new(pattern) {
            self.error_patterns.push(ErrorPattern {
                regex,
                error_kind: kind,
                message_transform: transform,
                suggestions,
                docs_url,
            });
        }
    }

    /// Translate Rust compiler output into user-friendly Synthesis errors
    pub fn translate_rust_error(&self, rust_output: &str) -> Option<SynthesisError> {
        for pattern in &self.error_patterns {
            if pattern.regex.is_match(rust_output) {
                let friendly_message = (pattern.message_transform)(rust_output);
                let mut error = SynthesisError::new(pattern.error_kind.clone(), friendly_message)
                    .with_suggestions(pattern.suggestions.clone());
                
                if let Some(docs) = &pattern.docs_url {
                    error = error.with_docs(docs.clone());
                }
                
                return Some(error);
            }
        }
        None
    }

    /// Run rustc and capture/translate any errors
    pub fn compile_and_translate(&self, source_file: &str) -> Result<()> {
        let output = Command::new("rustc")
            .arg("--crate-type")
            .arg("bin")
            .arg("--emit")
            .arg("metadata")
            .arg(source_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    
                    // Try to translate the error
                    if let Some(translated_error) = self.translate_rust_error(&stderr) {
                        return Err(translated_error);
                    }
                    
                    // Fallback to generic compilation error
                    return Err(SynthesisError::compilation_failed(
                        "Compilation failed with errors"
                    ));
                }
                Ok(())
            }
            Err(e) => Err(SynthesisError::compilation_failed(
                format!("Could not run Rust compiler: {}", e)
            )),
        }
    }
}

/// Extract type names from Rust type mismatch errors
fn extract_type_mismatch(error_msg: &str) -> Option<(String, String)> {
    let re = Regex::new(r"expected `([^`]+)`.*found `([^`]+)`").ok()?;
    let captures = re.captures(error_msg)?;
    
    let expected = simplify_rust_type(&captures[1]);
    let found = simplify_rust_type(&captures[2]);
    
    Some((expected, found))
}

/// Convert Rust types to user-friendly Synthesis types
fn simplify_rust_type(rust_type: &str) -> String {
    match rust_type {
        s if s.contains("f32") || s.contains("f64") => "Number".to_string(),
        s if s.contains("String") || s.contains("&str") => "Text".to_string(),
        s if s.contains("Vec") => "List".to_string(),
        s if s.contains("HashMap") => "Map".to_string(),
        s if s.contains("bool") => "Boolean".to_string(),
        s if s.contains("AudioBuffer") => "Audio".to_string(),
        s if s.contains("GraphicsContext") => "Graphics".to_string(),
        s if s.contains("Stream") => "Stream".to_string(),
        s if s.contains("()") => "Nothing".to_string(),
        _ => {
            // Remove generic parameters and crate prefixes
            let simplified = rust_type
                .split('<').next().unwrap_or(rust_type)
                .split(':').last().unwrap_or(rust_type)
                .trim();
            
            if simplified.is_empty() {
                "Unknown".to_string()
            } else {
                simplified.to_string()
            }
        }
    }
}

/// Global error translator instance
static ERROR_TRANSLATOR: std::sync::OnceLock<RustErrorTranslator> = std::sync::OnceLock::new();

pub fn get_error_translator() -> &'static RustErrorTranslator {
    ERROR_TRANSLATOR.get_or_init(|| RustErrorTranslator::new())
}

/// Context-aware suggestion generator
pub fn generate_context_suggestions(error_kind: &ErrorKind, message: &str) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    // Add context-specific suggestions based on error content
    match error_kind {
        ErrorKind::SyntaxError | ErrorKind::UnexpectedToken => {
            if message.contains("import") {
                suggestions.push("Import statements should be at the top of your file".to_string());
                suggestions.push("Use: `import ModuleName` or `import ModuleName.{function1, function2}`".to_string());
            } else if message.contains("loop") {
                suggestions.push("Loop blocks need curly braces: `loop { ... }`".to_string());
                suggestions.push("Make sure your loop has a body with actual code".to_string());
            } else if message.contains("function") || message.contains("call") {
                suggestions.push("Function calls need parentheses: `function_name()`".to_string());
                suggestions.push("Check that all function parameters are provided".to_string());
            }
        }
        
        ErrorKind::UnknownModule => {
            suggestions.push("Available built-in modules: Audio, Graphics, GUI, Hardware, Math, Time".to_string());
            suggestions.push("Check for typos in module names".to_string());
            suggestions.push("Module names are case-sensitive".to_string());
        }
        
        ErrorKind::AudioDeviceError => {
            if message.contains("device") {
                suggestions.push("Try plugging/unplugging your audio device".to_string());
                suggestions.push("Check audio device settings in your system".to_string());
                suggestions.push("Close other applications that might be using audio".to_string());
            }
        }
        
        ErrorKind::TypeMismatch => {
            if message.contains("Number") && message.contains("Text") {
                suggestions.push("Use Number.parse(text) to convert text to numbers".to_string());
                suggestions.push("Or use text interpolation: `\"The value is: ${number}\"`".to_string());
            } else if message.contains("Audio") {
                suggestions.push("Audio data comes from mic_input() or file loading functions".to_string());
                suggestions.push("Use Audio.convert() to change between audio formats".to_string());
            }
        }
        
        _ => {}
    }
    
    // Add general helpful suggestions based on common patterns
    if message.contains("not found") && !suggestions.is_empty() {
        suggestions.insert(0, "Double-check the spelling of names".to_string());
    }
    
    if message.contains("buffer") {
        suggestions.push("Buffer issues often indicate timing problems".to_string());
        suggestions.push("Try adjusting --buffer-size or --sample-rate options".to_string());
    }
    
    suggestions
}

/// Helper function to translate common anyhow errors
pub fn translate_anyhow_error(err: &anyhow::Error) -> SynthesisError {
    let error_str = err.to_string();
    
    // Try the error translator first
    if let Some(translated) = get_error_translator().translate_rust_error(&error_str) {
        return translated;
    }
    
    // Fallback patterns for anyhow errors
    let mut error = if error_str.contains("parse") || error_str.contains("syntax") {
        SynthesisError::new(ErrorKind::SyntaxError, "There's a syntax error in your Synthesis code")
            .with_suggestion("Check your brackets, parentheses, and punctuation")
            .with_suggestion("Make sure all statements end properly")
            .with_docs("https://synthesis-lang.org/docs/syntax")
    } else if error_str.contains("type") {
        SynthesisError::type_inference_error("automatic type detection failed")
    } else if error_str.contains("not found") {
        SynthesisError::new(ErrorKind::UnknownFunction, "Something you're trying to use doesn't exist")
            .with_suggestion("Check spelling of function and variable names")
            .with_suggestion("Make sure all modules are properly imported")
    } else {
        SynthesisError::new(ErrorKind::CompilationFailed, "Something went wrong during compilation")
            .with_suggestion("Try simplifying your code to isolate the issue")
            .with_suggestion("Use --debug flag for more detailed information")
            .with_suggestion("Report this issue if it persists")
    };
    
    // Add context-aware suggestions
    let context_suggestions = generate_context_suggestions(&error.kind, &error.message);
    for suggestion in context_suggestions {
        error = error.with_suggestion(suggestion);
    }
    
    error
}

// Helper function to create SynthesisError from string messages
pub fn synthesis_error(kind: ErrorKind, message: impl Into<String>) -> SynthesisError {
    SynthesisError::new(kind, message)
}

// Convenient error creation function that returns Result
pub fn error_result<T>(kind: ErrorKind, message: impl Into<String>) -> Result<T> {
    Err(SynthesisError::new(kind, message))
}

// Helper macros for creating errors
#[macro_export]
macro_rules! synthesis_error {
    ($kind:expr, $msg:expr) => {
        SynthesisError::new($kind, $msg)
    };
    ($kind:expr, $msg:expr, $($suggestions:expr),+) => {
        SynthesisError::new($kind, $msg)
            $(.with_suggestion($suggestions))+
    };
}

#[macro_export]
macro_rules! bail_synthesis {
    ($kind:expr, $msg:expr) => {
        return Err(SynthesisError::new($kind, $msg))
    };
    ($kind:expr, $msg:expr, $($suggestions:expr),+) => {
        return Err(SynthesisError::new($kind, $msg)
            $(.with_suggestion($suggestions))+)
    };
}