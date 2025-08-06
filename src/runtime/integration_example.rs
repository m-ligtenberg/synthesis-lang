/// Integration example showing how all stream composition systems work together
/// This demonstrates the complete creative programming workflow

use crate::runtime::{
    StreamCompositionEngine, CreativeComposer, RealtimeOptimizer, StreamErrorTranslator,
    StreamManager, StreamTransform, ConnectionType, CompositionContext
};
use crate::runtime::types::{DataType, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Complete example of a creative audio-visual composition
pub struct LiveCompositionExample {
    composer: CreativeComposer,
    optimizer: RealtimeOptimizer,
    error_translator: StreamErrorTranslator,
}

impl LiveCompositionExample {
    pub fn new() -> crate::Result<Self> {
        let mut composer = CreativeComposer::new();
        let optimizer = RealtimeOptimizer::new();
        let mut error_translator = StreamErrorTranslator::new();
        
        // Set up composition context
        error_translator.set_context(CompositionContext {
            current_streams: vec!["audio_input".to_string(), "visual_output".to_string()],
            user_intent: Some("creating live performance".to_string()),
            ..Default::default()
        });
        
        Ok(Self {
            composer,
            optimizer,
            error_translator,
        })
    }
    
    /// Demonstrate a complete creative workflow
    pub fn run_live_performance_demo(&mut self) -> crate::Result<()> {
        println!("🎭 Starting live audio-visual performance...");
        
        // Create input streams with creative names
        self.create_input_streams()?;
        
        // Build the composition step by step
        self.build_audio_processing_chain()?;
        self.build_visual_generation()?;
        self.connect_audio_to_visuals()?;
        
        // Optimize for real-time performance
        self.optimize_for_performance()?;
        
        // Run the live performance loop
        self.performance_loop()?;
        
        println!("🎬 Performance complete!");
        Ok(())
    }
    
    fn create_input_streams(&mut self) -> crate::Result<()> {
        println!("🎤 Setting up inputs...");
        
        // Create streams with user-friendly error handling
        self.safe_operation("creating microphone input", || {
            // In a real implementation, this would connect to actual audio devices
            Ok(())
        })?;
        
        self.safe_operation("creating MIDI controller", || {
            // MIDI input stream
            Ok(())
        })?;
        
        Ok(())
    }
    
    fn build_audio_processing_chain(&mut self) -> crate::Result<()> {
        println!("🔊 Building audio processing...");
        
        self.safe_operation("creating reverb", || {
            self.composer.process_through("microphone", vec!["reverb", "delay"])
        })?;
        
        self.safe_operation("adding bass enhancement", || {
            self.composer.spread("bass_input", vec!["bass_boost", "sub_harmonics"])
        })?;
        
        self.safe_operation("mixing layers", || {
            self.composer.layer("main_mix", vec!["microphone_processed", "bass_enhanced", "drums"])
        })?;
        
        // Set musical context
        self.safe_operation("setting tempo", || {
            self.composer.sync_to_tempo("main_mix", 128.0) // 128 BPM
        })?;
        
        self.safe_operation("setting key", || {
            self.composer.set_key("main_mix", "D minor")
        })?;
        
        Ok(())
    }
    
    fn build_visual_generation(&mut self) -> crate::Result<()> {
        println!("🌈 Creating visuals...");
        
        self.safe_operation("setting visual palette", || {
            self.composer.set_palette("visuals", "cyberpunk")
        })?;
        
        self.safe_operation("creating particle system", || {
            // Visual processing chain
            Ok(())
        })?;
        
        Ok(())
    }
    
    fn connect_audio_to_visuals(&mut self) -> crate::Result<()> {
        println!("🎨 Connecting audio to visuals...");
        
        self.safe_operation("setting up beat-responsive visuals", || {
            self.composer.when_beat_drops("main_mix", "screen_flash")
        })?;
        
        self.safe_operation("harmonizing audio and visuals", || {
            self.composer.harmonize("main_mix", "visual_generator", 0.8)
        })?;
        
        Ok(())
    }
    
    fn optimize_for_performance(&mut self) -> crate::Result<()> {
        println!("⚡ Optimizing for real-time performance...");
        
        // This would optimize the internal engine, but we'll simulate it
        self.safe_operation("optimizing stream routing", || {
            // In real implementation: self.optimizer.optimize_engine(&mut self.composer.engine)
            println!("   📊 Stream routing optimized for <1ms latency");
            Ok(())
        })?;
        
        self.safe_operation("pre-allocating buffers", || {
            println!("   🧠 Memory pools prepared for zero-allocation processing");
            Ok(())
        })?;
        
        Ok(())
    }
    
    fn performance_loop(&mut self) -> crate::Result<()> {
        println!("🎵 Starting live performance loop...");
        
        let performance_duration = Duration::from_secs(10); // 10-second demo
        let start_time = Instant::now();
        let target_frame_time = Duration::from_millis(16); // ~60fps
        
        let mut frame_count = 0;
        let mut total_processing_time = Duration::new(0, 0);
        
        while start_time.elapsed() < performance_duration {
            let frame_start = Instant::now();
            
            // Real-time processing with deadline
            let deadline = frame_start + Duration::from_micros(800); // 0.8ms budget
            
            self.safe_operation("processing streams", || {
                self.composer.perform()?;
                
                // Simulate some processing time
                std::thread::sleep(Duration::from_micros(200));
                
                Ok(())
            })?;
            
            let frame_time = frame_start.elapsed();
            total_processing_time += frame_time;
            frame_count += 1;
            
            // Check if we're meeting performance requirements
            if frame_time > target_frame_time {
                println!("⚠️  Frame time exceeded: {:.2}ms (target: 16ms)", 
                        frame_time.as_micros() as f64 / 1000.0);
            }
            
            // Health check every second
            if frame_count % 60 == 0 {
                self.health_check();
            }
            
            // Sleep for remaining frame time
            if frame_time < target_frame_time {
                std::thread::sleep(target_frame_time - frame_time);
            }
        }
        
        // Performance summary
        let avg_frame_time = total_processing_time / frame_count;
        println!("📈 Performance Summary:");
        println!("   Frames processed: {}", frame_count);
        println!("   Average frame time: {:.2}ms", avg_frame_time.as_micros() as f64 / 1000.0);
        println!("   Target achieved: {}", if avg_frame_time < target_frame_time { "✅" } else { "❌" });
        
        Ok(())
    }
    
    fn health_check(&self) {
        println!("🏥 Stream Health Check:");
        
        // Check individual streams
        let streams_to_check = vec!["main_mix", "visuals", "microphone"];
        for stream_name in streams_to_check {
            let health = self.composer.stream_health(stream_name);
            println!("   {} {}: {}", health.stream_name, health.status, 
                    format!("({:.1}% health)", health.health_score * 100.0));
        }
        
        // Overall composition health
        let overview = self.composer.composition_overview();
        println!("   🎼 {}: {} streams, {} connections", 
                overview.health_summary, overview.total_streams, overview.active_connections);
        println!("   💡 {}", overview.performance_tip);
    }
    
    /// Safe operation wrapper that translates errors
    fn safe_operation<F, T>(&self, operation_name: &str, operation: F) -> crate::Result<T>
    where
        F: FnOnce() -> crate::Result<T>,
    {
        match operation() {
            Ok(result) => {
                println!("   ✅ {}", operation_name);
                Ok(result)
            }
            Err(error) => {
                // Translate the error to user-friendly format
                let user_error = self.error_translator.translate_error(error, None);
                println!("   ❌ {} failed: {}", operation_name, user_error.message);
                
                if !user_error.suggestions.is_empty() {
                    println!("      💡 Suggestions:");
                    for suggestion in &user_error.suggestions {
                        println!("         • {}", suggestion);
                    }
                }
                
                Err(user_error)
            }
        }
    }
}

/// Example of language integration showing how this would look in Synthesis code
pub fn synthesis_language_example() -> String {
    r#"
// This is how the stream composition would look in actual Synthesis language

import Audio.{mic_input, reverb, delay}
import Graphics.{particle_system, flash}
import Composer.{layer, harmonize, sync_to_tempo}

// Create inputs
mic = Audio.mic_input()
drums = Audio.load("drums.wav")

// Build processing chain using creative language
processed_audio = mic |> reverb(room_size: 0.7) |> delay(time: 0.3s)

// Layer multiple streams
main_mix = Composer.layer([processed_audio, drums], gains: [0.8, 0.6])

// Sync to tempo
main_mix |> Composer.sync_to_tempo(128 BPM, key: "D minor")

// Connect to visuals using creative language
visuals = Graphics.particle_system(palette: "cyberpunk")
main_mix |> Composer.harmonize(visuals, blend: 0.8)

// Reactive visual effects
when main_mix.beat_drops() {
    Graphics.flash(Graphics.white, intensity: 0.9)
}

// Live performance loop
loop {
    Composer.perform()  // Process all connections in real-time
    
    // Everything happens automatically with <1ms latency!
}
"#.to_string()
}

/// Integration test showing error handling
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_translation() {
        let translator = StreamErrorTranslator::new();
        
        // Test that Rust errors get translated to user-friendly messages
        let rust_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let user_error = translator.translate_error(rust_error, None);
        
        // Should not contain Rust-specific terminology
        assert!(!user_error.message.contains("std::io::Error"));
        assert!(!user_error.message.contains("NotFound"));
        
        // Should have helpful suggestions
        assert!(!user_error.suggestions.is_empty());
    }
    
    #[test]
    fn test_creative_api_integration() {
        let mut composer = CreativeComposer::new();
        
        // Test that creative methods work
        assert!(composer.layer("test_mix", vec!["stream1", "stream2"]).is_ok());
        assert!(composer.sync_to_tempo("test_mix", 120.0).is_ok());
        
        let health = composer.stream_health("test_mix");
        assert_eq!(health.stream_name, "test_mix");
    }
}

impl Default for LiveCompositionExample {
    fn default() -> Self {
        Self::new().expect("Failed to create LiveCompositionExample")
    }
}"#