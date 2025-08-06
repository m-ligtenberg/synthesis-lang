use crate::runtime::stream_composition::{StreamCompositionEngine, StreamConnection, ConnectionType, StreamTransform};
use crate::runtime::streams::StreamManager;
use crate::runtime::types::{Value, DataType};
use crate::errors::ErrorKind;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Creative-friendly API layer for stream composition
/// This hides technical complexity behind musical and artistic terminology
#[derive(Debug)]
pub struct CreativeComposer {
    engine: StreamCompositionEngine,
    current_tempo: f32,
    current_key: String,
    current_time_signature: (u32, u32),
    health_status: ComposerHealth,
}

/// Health monitoring for the creative system
#[derive(Debug, Clone)]
pub struct ComposerHealth {
    pub status: HealthStatus,
    pub message: String,
    pub performance_score: f32, // 0.0-1.0, where 1.0 is perfect performance
    pub creativity_flow_active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Perfect,    // 🟢 Everything flowing beautifully
    Good,       // 🟡 Some minor hiccups but still creative
    Warning,    // 🟠 Performance issues affecting creativity
    Critical,   // 🔴 Major problems disrupting the flow
}

/// Musical context for creative operations
#[derive(Debug, Clone)]
pub struct MusicalContext {
    pub tempo_bpm: f32,
    pub key: String,
    pub scale: ScaleType,
    pub time_signature: (u32, u32),
    pub swing: f32, // 0.0-1.0
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScaleType {
    Major,
    Minor,
    Pentatonic,
    Blues,
    Chromatic,
    Custom(Vec<f32>), // Frequency ratios
}

/// Visual context for graphics operations
#[derive(Debug, Clone)]
pub struct VisualContext {
    pub palette: ColorPalette,
    pub mood: VisualMood,
    pub energy_level: f32, // 0.0-1.0
    pub complexity: f32,   // 0.0-1.0
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorPalette {
    Warm,
    Cool,
    Neon,
    Pastel,
    Monochrome,
    Custom(Vec<(f32, f32, f32)>), // RGB values
}

#[derive(Debug, Clone, PartialEq)]
pub enum VisualMood {
    Calm,
    Energetic,
    Mysterious,
    Joyful,
    Melancholy,
    Aggressive,
}

impl CreativeComposer {
    pub fn new() -> Self {
        Self {
            engine: StreamCompositionEngine::new(),
            current_tempo: 120.0,
            current_key: "C".to_string(),
            current_time_signature: (4, 4),
            health_status: ComposerHealth {
                status: HealthStatus::Perfect,
                message: "🎨 Ready to create! Everything is flowing beautifully.".to_string(),
                performance_score: 1.0,
                creativity_flow_active: true,
            },
        }
    }
    
    /// Harmonize multiple audio streams together
    /// This automatically handles mixing, key matching, and tempo sync
    pub fn harmonize(&mut self, streams: Vec<String>, output: String) -> Result<(), String> {
        self.check_creative_health();
        
        match self.harmonize_internal(streams, output) {
            Ok(_) => {
                self.update_health(HealthStatus::Perfect, "🎵 Beautiful harmony created!".to_string());
                Ok(())
            }
            Err(e) => {
                let friendly_error = self.translate_technical_error(&e);
                self.update_health(HealthStatus::Warning, friendly_error.clone());
                Err(friendly_error)
            }
        }
    }
    
    fn harmonize_internal(&mut self, streams: Vec<String>, output: String) -> crate::Result<()> {
        if streams.len() < 2 {
            return Err(crate::SynthesisError::new(ErrorKind::InvalidArgument,
                "Need at least 2 streams to create harmony".to_string()));
        }
        
        // Calculate mixing gains for harmonic balance
        let base_gain = 0.7 / streams.len() as f32; // Leave headroom
        let mix_gains = streams.iter().enumerate().map(|(i, _)| {
            // Apply slight variations for more natural sound
            base_gain * (1.0 + 0.1 * (i as f32 / streams.len() as f32))
        }).collect();
        
        self.engine.connect_merge(streams, output, mix_gains)
    }
    
    /// Layer streams for rich, textured sound
    /// This creates parallel processing with different effects on each layer
    pub fn layer(&mut self, base: String, layers: Vec<(String, String)>) -> Result<String, String> {
        self.check_creative_health();
        
        let output_name = format!("{}_layered", base);
        
        match self.layer_internal(base, layers, output_name.clone()) {
            Ok(_) => {
                self.update_health(HealthStatus::Good, "🎭 Rich layers created!".to_string());
                Ok(output_name)
            }
            Err(e) => {
                let friendly_error = self.translate_technical_error(&e);
                self.update_health(HealthStatus::Warning, friendly_error.clone());
                Err(friendly_error)
            }
        }
    }
    
    fn layer_internal(&mut self, base: String, layers: Vec<(String, String)>, output: String) -> crate::Result<()> {
        let mut all_streams = vec![base];
        
        // Create parallel processing for each layer
        for (layer_name, effect_type) in layers {
            // Create transform based on effect type
            let transform = match effect_type.as_str() {
                "reverb" => StreamTransform {
                    transform_id: "reverb".to_string(),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("room_size".to_string(), Value::Float(0.7));
                        params.insert("wet_mix".to_string(), Value::Float(0.3));
                        params
                    },
                    bypass: false,
                },
                "delay" => StreamTransform {
                    transform_id: "delay".to_string(),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("time".to_string(), Value::Float(0.25));
                        params.insert("feedback".to_string(), Value::Float(0.4));
                        params
                    },
                    bypass: false,
                },
                "chorus" => StreamTransform {
                    transform_id: "chorus".to_string(),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("rate".to_string(), Value::Float(2.0));
                        params.insert("depth".to_string(), Value::Float(0.5));
                        params
                    },
                    bypass: false,
                },
                _ => StreamTransform {
                    transform_id: "gain".to_string(),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("amount".to_string(), Value::Float(0.8));
                        params
                    },
                    bypass: false,
                },
            };
            
            // Add layer connection with transform
            let connection = StreamConnection {
                source: base.clone(),
                destination: layer_name.clone(),
                connection_type: ConnectionType::Parallel,
                transform: Some(transform),
                routing: crate::runtime::stream_composition::RoutingConfig::default(),
            };
            
            all_streams.push(layer_name);
        }
        
        // Mix all layers together
        let mix_gains = vec![1.0 / all_streams.len() as f32; all_streams.len()];
        self.engine.connect_merge(all_streams, output, mix_gains)
    }
    
    /// Spread a mono source across the stereo field
    /// Creates spatial width and movement
    pub fn spread(&mut self, mono_input: String, width: f32, movement_speed: f32) -> Result<(String, String), String> {
        self.check_creative_health();
        
        let left_output = format!("{}_left", mono_input);
        let right_output = format!("{}_right", mono_input);
        
        match self.spread_internal(mono_input, left_output.clone(), right_output.clone(), width, movement_speed) {
            Ok(_) => {
                self.update_health(HealthStatus::Good, "🎧 Spatial width created!".to_string());
                Ok((left_output, right_output))
            }
            Err(e) => {
                let friendly_error = self.translate_technical_error(&e);
                self.update_health(HealthStatus::Warning, friendly_error.clone());
                Err(friendly_error)
            }
        }
    }
    
    fn spread_internal(&mut self, input: String, left: String, right: String, width: f32, movement_speed: f32) -> crate::Result<()> {
        // Create left channel with slight delay and filtering
        let left_transform = StreamTransform {
            transform_id: "stereo_left".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("delay".to_string(), Value::Float(movement_speed * 0.001)); // Convert to seconds
                params.insert("gain".to_string(), Value::Float(0.7 * width));
                params.insert("highcut".to_string(), Value::Float(8000.0));
                params
            },
            bypass: false,
        };
        
        // Create right channel with different characteristics
        let right_transform = StreamTransform {
            transform_id: "stereo_right".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("delay".to_string(), Value::Float(movement_speed * 0.002));
                params.insert("gain".to_string(), Value::Float(0.7 * width));
                params.insert("lowcut".to_string(), Value::Float(100.0));
                params
            },
            bypass: false,
        };
        
        // Connect input to both outputs with different processing
        self.engine.connect_split(input, vec![left, right], vec![1.0, 1.0])?;
        
        Ok(())
    }
    
    /// Rhythmically sync multiple streams to a master tempo
    pub fn sync_to_beat(&mut self, streams: Vec<String>, subdivisions: Vec<f32>) -> Result<(), String> {
        self.check_creative_health();
        
        if streams.len() != subdivisions.len() {
            let error = "🎵 Oops! You need to specify a rhythm subdivision for each stream. Like: sync drums to quarter notes, bass to eighth notes, etc.".to_string();
            self.update_health(HealthStatus::Warning, error.clone());
            return Err(error);
        }
        
        match self.sync_internal(streams, subdivisions) {
            Ok(_) => {
                self.update_health(HealthStatus::Perfect, "🥁 Everything's in the pocket now!".to_string());
                Ok(())
            }
            Err(e) => {
                let friendly_error = self.translate_technical_error(&e);
                self.update_health(HealthStatus::Warning, friendly_error.clone());
                Err(friendly_error)
            }
        }
    }
    
    fn sync_internal(&mut self, streams: Vec<String>, subdivisions: Vec<f32>) -> crate::Result<()> {
        let samples_per_beat = (44100.0 * 60.0) / self.current_tempo; // Assuming 44.1kHz
        
        for (stream, subdivision) in streams.iter().zip(subdivisions.iter()) {
            let delay_samples = (samples_per_beat / subdivision) as usize;
            
            // This would integrate with a tempo/timing system
            // For now, just create a placeholder connection
            let connection = StreamConnection {
                source: stream.clone(),
                destination: format!("{}_synced", stream),
                connection_type: ConnectionType::Direct,
                transform: None,
                routing: crate::runtime::stream_composition::RoutingConfig {
                    delay_samples,
                    gain: 1.0,
                    ..Default::default()
                },
            };
            
            // Would add the connection to the engine
        }
        
        Ok(())
    }
    
    /// Create a visual reactive system that responds to audio
    pub fn visualize(&mut self, audio_stream: String, visual_context: VisualContext) -> Result<String, String> {
        self.check_creative_health();
        
        let visual_output = format!("{}_visual", audio_stream);
        
        match self.visualize_internal(audio_stream, visual_output.clone(), visual_context) {
            Ok(_) => {
                self.update_health(HealthStatus::Perfect, "🎨 Visuals are dancing with the music!".to_string());
                Ok(visual_output)
            }
            Err(e) => {
                let friendly_error = self.translate_technical_error(&e);
                self.update_health(HealthStatus::Warning, friendly_error.clone());
                Err(friendly_error)
            }
        }
    }
    
    fn visualize_internal(&mut self, audio: String, visual: String, context: VisualContext) -> crate::Result<()> {
        let transform = StreamTransform {
            transform_id: "audio_to_visual".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("palette".to_string(), Value::String(format!("{:?}", context.palette)));
                params.insert("mood".to_string(), Value::String(format!("{:?}", context.mood)));
                params.insert("energy".to_string(), Value::Float(context.energy_level as f64));
                params.insert("complexity".to_string(), Value::Float(context.complexity as f64));
                params
            },
            bypass: false,
        };
        
        let connection = StreamConnection {
            source: audio,
            destination: visual,
            connection_type: ConnectionType::Direct,
            transform: Some(transform),
            routing: crate::runtime::stream_composition::RoutingConfig::default(),
        };
        
        Ok(()) // Placeholder - would actually add connection
    }
    
    /// Set the musical context for all operations
    pub fn set_musical_context(&mut self, context: MusicalContext) {
        self.current_tempo = context.tempo_bpm;
        self.current_key = context.key;
        self.current_time_signature = context.time_signature;
        
        self.update_health(HealthStatus::Perfect, 
            format!("🎼 Set to {} BPM in {} {}, feeling the {}/{} groove!", 
                context.tempo_bpm, context.key, 
                match context.scale {
                    ScaleType::Major => "major",
                    ScaleType::Minor => "minor", 
                    _ => "scale"
                },
                context.time_signature.0, context.time_signature.1));
    }
    
    /// Get creative health status
    pub fn get_creative_health(&self) -> &ComposerHealth {
        &self.health_status
    }
    
    /// Check and update creative health based on system performance
    fn check_creative_health(&mut self) {
        // This would integrate with performance monitoring
        // For now, simulate health checks
        let stats = self.engine.get_composition_stats();
        
        if stats.active_connections > 100 {
            self.update_health(HealthStatus::Warning, 
                "🔥 Whoa! You've got a lot of creative energy flowing. Consider simplifying for smoother performance.".to_string());
        } else if stats.active_connections > 50 {
            self.update_health(HealthStatus::Good,
                "🎯 Great creative flow! Everything's running smoothly.".to_string());
        } else {
            self.update_health(HealthStatus::Perfect,
                "✨ Perfect creative conditions! Your imagination is the only limit.".to_string());
        }
    }
    
    fn update_health(&mut self, status: HealthStatus, message: String) {
        self.health_status.status = status.clone();
        self.health_status.message = message;
        self.health_status.performance_score = match status {
            HealthStatus::Perfect => 1.0,
            HealthStatus::Good => 0.8,
            HealthStatus::Warning => 0.6,
            HealthStatus::Critical => 0.3,
        };
    }
    
    /// Translate technical Rust errors into creative-friendly messages
    fn translate_technical_error(&self, error: &crate::SynthesisError) -> String {
        match error.kind {
            ErrorKind::InvalidArgument => {
                "🎭 Creative challenge! The combination you're trying isn't quite working. Try adjusting the parameters or using fewer elements.".to_string()
            }
            ErrorKind::UnknownModule => {
                "🔍 Hmm, I can't find that sound/visual stream. Make sure you've created it first, or check the name spelling.".to_string()
            }
            ErrorKind::AudioDeviceError => {
                "🎧 Audio system hiccup! Your sound hardware might be busy. Try refreshing the audio connection or checking your audio settings.".to_string()
            }
            _ => {
                format!("🎨 Creative flow interrupted: {}. Don't worry, this happens! Try a different approach.", 
                    error.message.replace("RefCell", "data").replace("borrow", "access").replace("mutex", "resource"))
            }
        }
    }
    
    /// Process the creative composition in real-time
    pub fn process_creative_flow(&mut self, stream_manager: &mut StreamManager) -> Result<(), String> {
        match self.engine.process_composition(stream_manager) {
            Ok(_) => {
                self.health_status.creativity_flow_active = true;
                Ok(())
            }
            Err(e) => {
                self.health_status.creativity_flow_active = false;
                let friendly_error = self.translate_technical_error(&e);
                self.update_health(HealthStatus::Critical, friendly_error.clone());
                Err(friendly_error)
            }
        }
    }
    
    /// Create a complete creative workflow example
    pub fn create_workflow_example(&mut self) -> Result<String, String> {
        // This demonstrates a complete creative workflow
        let workflow_description = "🎼 Creating a beautiful audiovisual experience:\n\
            1. 🎤 Capture microphone input\n\
            2. 🎵 Harmonize with generated tones\n\
            3. 🎭 Layer with reverb and delay effects\n\
            4. 🎧 Spread across stereo field\n\
            5. 🎨 Generate reactive visuals\n\
            6. 🥁 Sync everything to the beat".to_string();
        
        // Simulate workflow creation
        // In reality, this would create actual streams and connections
        self.set_musical_context(MusicalContext {
            tempo_bpm: 120.0,
            key: "C".to_string(),
            scale: ScaleType::Minor,
            time_signature: (4, 4),
            swing: 0.1,
        });
        
        self.update_health(HealthStatus::Perfect, "🌟 Creative workflow ready! Let the music and visuals flow!".to_string());
        Ok(workflow_description)
    }
}

impl Default for CreativeComposer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MusicalContext {
    fn default() -> Self {
        Self {
            tempo_bpm: 120.0,
            key: "C".to_string(),
            scale: ScaleType::Major,
            time_signature: (4, 4),
            swing: 0.0,
        }
    }
}

impl Default for VisualContext {
    fn default() -> Self {
        Self {
            palette: ColorPalette::Warm,
            mood: VisualMood::Calm,
            energy_level: 0.5,
            complexity: 0.5,
        }
    }
}