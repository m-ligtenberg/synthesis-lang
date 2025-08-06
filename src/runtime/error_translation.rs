/// Error translation layer for stream composition system
/// Converts all Rust errors into creative, user-friendly messages
use crate::errors::{SynthesisError, ErrorKind, SourceLocation};
use crate::runtime::stream_composition::*;
use std::collections::HashMap;

/// Translates technical errors into creative, user-friendly messages
pub struct StreamErrorTranslator {
    /// Map of technical error patterns to creative messages
    error_patterns: HashMap<String, ErrorTemplate>,
    /// Context about the current composition for better error messages
    composition_context: CompositionContext,
}

#[derive(Debug, Clone)]
pub struct ErrorTemplate {
    pub user_message: String,
    pub technical_hint: String,
    pub suggestions: Vec<String>,
    pub emoji: String,
}

#[derive(Debug, Clone, Default)]
pub struct CompositionContext {
    pub current_streams: Vec<String>,
    pub active_connections: usize,
    pub last_successful_operation: Option<String>,
    pub user_intent: Option<String>, // e.g., "creating beat", "adding reverb"
}

impl StreamErrorTranslator {
    pub fn new() -> Self {
        let mut error_patterns = HashMap::new();
        
        // Stream connection errors
        error_patterns.insert("stream_not_found".to_string(), ErrorTemplate {
            user_message: "I can't find that stream in your composition".to_string(),
            technical_hint: "Stream name doesn't exist in the stream manager".to_string(),
            suggestions: vec![
                "Check if you've created the stream first".to_string(),
                "Verify the stream name spelling".to_string(),
                "Use 'list_streams()' to see available streams".to_string(),
            ],
            emoji: "🔍".to_string(),
        });
        
        error_patterns.insert("circular_dependency".to_string(), ErrorTemplate {
            user_message: "Your stream connections create a loop - audio would feed back infinitely!".to_string(),
            technical_hint: "Circular dependency detected in stream graph".to_string(),
            suggestions: vec![
                "Check your connections for loops (A→B→C→A)".to_string(),
                "Consider using a delay buffer to break the feedback loop".to_string(),
                "Rethink your signal flow design".to_string(),
            ],
            emoji: "🔄".to_string(),
        });
        
        error_patterns.insert("buffer_overflow".to_string(), ErrorTemplate {
            user_message: "Your stream is producing data faster than it can be consumed".to_string(),
            technical_hint: "Stream buffer exceeded maximum capacity".to_string(),
            suggestions: vec![
                "Reduce the data rate of your input".to_string(),
                "Increase buffer size if you have enough memory".to_string(),
                "Check if the receiving stream is processing properly".to_string(),
            ],
            emoji: "💧".to_string(),
        });
        
        error_patterns.insert("performance_constraint".to_string(), ErrorTemplate {
            user_message: "Your composition is too complex for real-time performance".to_string(),
            technical_hint: "Processing time exceeded real-time constraints".to_string(),
            suggestions: vec![
                "Simplify your stream network".to_string(),
                "Use fewer simultaneous effects".to_string(),
                "Consider processing some streams at lower sample rates".to_string(),
            ],
            emoji: "⚡".to_string(),
        });
        
        error_patterns.insert("type_mismatch".to_string(), ErrorTemplate {
            user_message: "These streams speak different languages - like connecting audio to graphics".to_string(),
            technical_hint: "Stream data types are incompatible".to_string(),
            suggestions: vec![
                "Use a converter stream to translate between types".to_string(),
                "Check that audio connects to audio, graphics to graphics".to_string(),
                "Consider using a multi-modal stream if mixing types intentionally".to_string(),
            ],
            emoji: "🔌".to_string(),
        });
        
        error_patterns.insert("deadlock".to_string(), ErrorTemplate {
            user_message: "Your streams are waiting for each other - nobody can move!".to_string(),
            technical_hint: "Mutex deadlock detected in stream processing".to_string(),
            suggestions: vec![
                "Simplify your connection pattern".to_string(),
                "Avoid bidirectional connections without buffers".to_string(),
                "Use async processing for complex routing".to_string(),
            ],
            emoji: "🔒".to_string(),
        });
        
        error_patterns.insert("memory_exhausted".to_string(), ErrorTemplate {
            user_message: "Your composition is using too much memory".to_string(),
            technical_hint: "Memory allocation failed".to_string(),
            suggestions: vec![
                "Reduce buffer sizes".to_string(),
                "Use streaming processing instead of buffering everything".to_string(),
                "Close unused streams".to_string(),
            ],
            emoji: "🧠".to_string(),
        });
        
        error_patterns.insert("audio_device_error".to_string(), ErrorTemplate {
            user_message: "Can't connect to your audio device".to_string(),
            technical_hint: "Audio device initialization or I/O failed".to_string(),
            suggestions: vec![
                "Check that your audio device is connected".to_string(),
                "Try restarting the audio system".to_string(),
                "Verify audio device permissions".to_string(),
            ],
            emoji: "🎧".to_string(),
        });
        
        Self {
            error_patterns,
            composition_context: CompositionContext::default(),
        }
    }
    
    /// Update context to provide better error messages
    pub fn set_context(&mut self, context: CompositionContext) {
        self.composition_context = context;
    }
    
    /// Translate any error into a user-friendly Synthesis error
    pub fn translate_error<E>(&self, error: E, location: Option<SourceLocation>) -> SynthesisError
    where
        E: std::fmt::Debug + std::fmt::Display,
    {
        let error_str = format!("{:?}", error);
        let display_str = format!("{}", error);
        
        // Try to match against known patterns
        if let Some(template) = self.match_error_pattern(&error_str, &display_str) {
            return SynthesisError {
                kind: self.determine_error_kind(&error_str),
                message: self.contextualize_message(&template.user_message),
                location,
                suggestions: template.suggestions.clone(),
                related_docs: Some("https://synthesis-lang.org/docs/streams".to_string()),
            };
        }
        
        // Handle Rust-specific errors that might leak through
        if error_str.contains("thread") && error_str.contains("panicked") {
            return SynthesisError {
                kind: ErrorKind::PerformanceConstraintViolation,
                message: "🚫 Something went wrong in your stream processing - the system had to stop to prevent damage".to_string(),
                location,
                suggestions: vec![
                    "Try simplifying your composition".to_string(),
                    "Check for infinite loops in your connections".to_string(),
                    "Report this as a bug if it keeps happening".to_string(),
                ],
                related_docs: Some("https://synthesis-lang.org/docs/troubleshooting".to_string()),
            };
        }
        
        if error_str.contains("borrow") || error_str.contains("RefCell") {
            return SynthesisError {
                kind: ErrorKind::InvalidStreamConnection,
                message: "🔄 Multiple parts of your composition are trying to use the same stream simultaneously".to_string(),
                location,
                suggestions: vec![
                    "Use stream forking to create independent copies".to_string(),
                    "Simplify your connection pattern".to_string(),
                    "Consider using async processing".to_string(),
                ],
                related_docs: Some("https://synthesis-lang.org/docs/stream-sharing".to_string()),
            };
        }
        
        if error_str.contains("index out of bounds") || error_str.contains("slice index") {
            return SynthesisError {
                kind: ErrorKind::StreamBufferOverflow,
                message: "📊 Trying to access data beyond what's available in your stream buffer".to_string(),
                location,
                suggestions: vec![
                    "Check that your stream has enough data".to_string(),
                    "Use buffer size queries before accessing data".to_string(),
                    "Consider using streaming reads instead of bulk access".to_string(),
                ],
                related_docs: Some("https://synthesis-lang.org/docs/buffer-management".to_string()),
            };
        }
        
        if error_str.contains("send") || error_str.contains("channel") {
            return SynthesisError {
                kind: ErrorKind::InvalidStreamConnection,
                message: "📡 Communication between your streams broke down".to_string(),
                location,
                suggestions: vec![
                    "Check if all connected streams are still active".to_string(),
                    "Verify your connection setup".to_string(),
                    "Try recreating the problematic connections".to_string(),
                ],
                related_docs: Some("https://synthesis-lang.org/docs/stream-communication".to_string()),
            };
        }
        
        // Generic fallback that hides all technical details
        SynthesisError {
            kind: ErrorKind::PerformanceConstraintViolation,
            message: format!("🎛️ Something unexpected happened while processing your streams: {}", 
                self.make_user_friendly(&display_str)),
            location,
            suggestions: vec![
                "Try a simpler version of what you were doing".to_string(),
                "Check the documentation for examples".to_string(),
                "Report this if it keeps happening".to_string(),
            ],
            related_docs: Some("https://synthesis-lang.org/docs/getting-help".to_string()),
        }
    }
    
    fn match_error_pattern(&self, error_debug: &str, error_display: &str) -> Option<&ErrorTemplate> {
        // Check for specific error patterns
        if error_debug.contains("Stream") && error_debug.contains("not found") {
            return self.error_patterns.get("stream_not_found");
        }
        
        if error_debug.contains("circular") || error_debug.contains("dependency") {
            return self.error_patterns.get("circular_dependency");
        }
        
        if error_debug.contains("buffer") && (error_debug.contains("overflow") || error_debug.contains("full")) {
            return self.error_patterns.get("buffer_overflow");
        }
        
        if error_debug.contains("deadline") || error_debug.contains("timeout") || error_debug.contains("performance") {
            return self.error_patterns.get("performance_constraint");
        }
        
        if error_debug.contains("type") && error_debug.contains("mismatch") {
            return self.error_patterns.get("type_mismatch");
        }
        
        if error_debug.contains("deadlock") || error_debug.contains("would block") {
            return self.error_patterns.get("deadlock");
        }
        
        if error_debug.contains("memory") || error_debug.contains("allocation") || error_display.contains("out of memory") {
            return self.error_patterns.get("memory_exhausted");
        }
        
        if error_debug.contains("audio") || error_debug.contains("cpal") || error_debug.contains("device") {
            return self.error_patterns.get("audio_device_error");
        }
        
        None
    }
    
    fn determine_error_kind(&self, error_str: &str) -> ErrorKind {
        if error_str.contains("audio") || error_str.contains("device") {
            ErrorKind::AudioDeviceError
        } else if error_str.contains("buffer") {
            ErrorKind::StreamBufferOverflow
        } else if error_str.contains("performance") || error_str.contains("deadline") {
            ErrorKind::PerformanceConstraintViolation
        } else if error_str.contains("connection") || error_str.contains("stream") {
            ErrorKind::InvalidStreamConnection
        } else if error_str.contains("type") {
            ErrorKind::TypeMismatch
        } else {
            ErrorKind::PerformanceConstraintViolation // Safe default
        }
    }
    
    fn contextualize_message(&self, base_message: &str) -> String {
        let mut message = base_message.to_string();
        
        // Add context if available
        if let Some(ref intent) = self.composition_context.user_intent {
            message = format!("{} (while {})", message, intent);
        }
        
        if !self.composition_context.current_streams.is_empty() {
            message = format!("{}\n\nActive streams: {}", 
                message, 
                self.composition_context.current_streams.join(", "));
        }
        
        message
    }
    
    fn make_user_friendly(&self, technical_message: &str) -> String {
        // Remove Rust-specific terminology
        let user_message = technical_message
            .replace("RefCell", "data access")  
            .replace("Mutex", "synchronization")
            .replace("thread", "processing")
            .replace("panic", "unexpected stop")
            .replace("unwrap", "data access")
            .replace("expect", "validation")
            .replace("borrow", "access")
            .replace("lifetime", "data validity")
            .replace("trait", "capability")
            .replace("impl", "implementation")
            .replace("&mut", "mutable")
            .replace("&", "reference to")
            .replace("Result<", "operation result of ")
            .replace("Option<", "optional ")
            .replace("Vec<", "list of ")
            .replace("HashMap<", "mapping of ");
        
        // Limit length to avoid overwhelming users
        if user_message.len() > 100 {
            format!("{}...", &user_message[..97])
        } else {
            user_message
        }
    }
    
    /// Wrapper for Result types that automatically translates errors
    pub fn translate_result<T, E>(&self, result: Result<T, E>, location: Option<SourceLocation>) -> crate::Result<T>
    where
        E: std::fmt::Debug + std::fmt::Display,
    {
        match result {
            Ok(value) => Ok(value),
            Err(error) => Err(self.translate_error(error, location)),
        }
    }
}

/// Convenience macro for translating errors in stream operations
#[macro_export]
macro_rules! stream_try {
    ($translator:expr, $result:expr, $location:expr) => {
        $translator.translate_result($result, $location)?
    };
    ($translator:expr, $result:expr) => {
        $translator.translate_result($result, None)?
    };
}

/// Helper trait to add context to operations
pub trait StreamOperationContext {
    fn with_intent(self, intent: &str) -> Self;
    fn with_streams(self, streams: Vec<String>) -> Self;
}

impl StreamOperationContext for StreamErrorTranslator {
    fn with_intent(mut self, intent: &str) -> Self {
        self.composition_context.user_intent = Some(intent.to_string());
        self
    }
    
    fn with_streams(mut self, streams: Vec<String>) -> Self {
        self.composition_context.current_streams = streams;
        self
    }
}

impl Default for StreamErrorTranslator {
    fn default() -> Self {
        Self::new()
    }
}

// Integration with existing stream composition engine
impl StreamCompositionEngine {
    /// Create connections with error translation
    pub fn connect_direct_safe(
        &mut self, 
        source: String, 
        destination: String, 
        gain: f32,
        translator: &StreamErrorTranslator,
    ) -> crate::Result<()> {
        let context = CompositionContext {
            current_streams: vec![source.clone(), destination.clone()],
            user_intent: Some("connecting streams".to_string()),
            ..Default::default()
        };
        
        let mut contextual_translator = translator.clone();
        contextual_translator.set_context(context);
        
        contextual_translator.translate_result(
            self.connect_direct(source, destination, gain),
            None
        )
    }
}

impl Clone for StreamErrorTranslator {
    fn clone(&self) -> Self {
        Self {
            error_patterns: self.error_patterns.clone(),
            composition_context: self.composition_context.clone(),
        }
    }
}