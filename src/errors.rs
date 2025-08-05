use std::fmt;

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
    
    // Runtime errors
    AudioDeviceError,
    GraphicsContextError,
    StreamBufferOverflow,
    PerformanceConstraintViolation,
    
    // Compilation errors
    CompilationFailed,
    OptimizationFailed,
    CodeGenerationFailed,
    
    // System errors
    FileNotFound,
    PermissionDenied,
    OutOfMemory,
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

    pub fn file_not_found(filename: impl Into<String>) -> Self {
        let file = filename.into();
        Self::new(
            ErrorKind::FileNotFound,
            format!("Cannot find file '{}'", file)
        )
        .with_suggestion("Make sure the file exists and the path is correct")
        .with_suggestion("Synthesis files should have .syn extension")
    }
}

impl fmt::Display for SynthesisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Friendly header with emoji
        let emoji = match self.kind {
            ErrorKind::SyntaxError | ErrorKind::UnexpectedToken | ErrorKind::MissingToken => "🎵",
            ErrorKind::UnknownModule | ErrorKind::UnknownFunction => "🔍",
            ErrorKind::TypeMismatch => "🔄",
            ErrorKind::AudioDeviceError => "🎧",
            ErrorKind::GraphicsContextError => "🎨",
            ErrorKind::StreamBufferOverflow => "🌊",
            ErrorKind::PerformanceConstraintViolation => "⚡",
            ErrorKind::CompilationFailed => "🔧",
            ErrorKind::FileNotFound => "📁",
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

// Never expose internal Rust errors to users
impl From<anyhow::Error> for SynthesisError {
    fn from(err: anyhow::Error) -> Self {
        // Convert anyhow errors to friendly Synthesis errors
        let error_str = err.to_string();
        
        if error_str.contains("parse") || error_str.contains("syntax") {
            SynthesisError::new(ErrorKind::SyntaxError, "Syntax error in your Synthesis code")
                .with_suggestion("Check your brackets, parentheses, and syntax")
        } else if error_str.contains("type") {
            SynthesisError::new(ErrorKind::TypeMismatch, "Type compatibility issue")
                .with_suggestion("Synthesis handles most type conversions automatically")
        } else {
            SynthesisError::new(ErrorKind::CompilationFailed, "Something went wrong during compilation")
                .with_suggestion("Try simplifying your code and compile again")
                .with_suggestion("Report this issue if it persists")
        }
    }
}

pub type Result<T> = std::result::Result<T, SynthesisError>;

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