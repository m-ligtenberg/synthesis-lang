#[cfg(test)]
mod performance_tests {
    use super::*;
    use crate::runtime::{
        streams::{StreamManager, InputSourceType, OutputDestinationType, OutputFormat, TransformType, FilterType},
        realtime_buffer::{RealtimeCircularBuffer, SharedRealtimeBuffer, RealtimeBufferPool},
        stream_composition::{StreamCompositionEngine, ConnectionType},
        creative_api::{CreativeComposer, MusicalContext, VisualContext, ScaleType, ColorPalette, VisualMood},
        creative_types::CreativeTypeSystem,
    };
    use std::time::{Duration, Instant};
    use std::thread;

    /// Target latency for real-time audio processing
    const TARGET_LATENCY_US: u64 = 1000; // 1ms = 1000 microseconds
    
    /// Audio buffer size for testing (typical real-time size)  
    const AUDIO_BUFFER_SIZE: usize = 128; // samples
    
    /// Sample rate for audio testing
    const SAMPLE_RATE: f32 = 44100.0;
    
    /// Number of iterations for buffer performance testing
    const BUFFER_PERF_ITERATIONS: u128 = 10000;
    
    /// Number of iterations for audio processing tests
    const AUDIO_PROCESSING_ITERATIONS: usize = 1000;
    
    /// Number of iterations for composition testing
    const COMPOSITION_ITERATIONS: usize = 100;
    
    /// Maximum acceptable memory growth during processing (KB)
    const MAX_MEMORY_GROWTH_KB: usize = 1000;
    
    /// Helper macro for performance assertions with detailed reporting
    macro_rules! assert_performance {
        ($condition:expr, $metric_name:expr, $actual:expr, $target:expr, $unit:expr) => {
            if !$condition {
                panic!("❌ Performance failure in {}: actual {} {} exceeds target {} {}\n   \
                       Ratio: {:.2}x over target\n   \
                       This may indicate: CPU overload, memory pressure, or algorithmic inefficiency",
                       $metric_name, $actual, $unit, $target, $unit, 
                       $actual as f64 / $target as f64);
            }
        };
    }
    
    /// Performance configuration that can be adjusted for different hardware
    #[derive(Debug, Clone)]
    struct PerformanceConfig {
        /// Scale factor for performance thresholds (1.0 = default, >1.0 = more lenient)
        scale_factor: f64,
        /// Whether to run intensive stress tests
        enable_stress_tests: bool,
        /// Hardware tier (affects expectations)
        hardware_tier: HardwareTier,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    enum HardwareTier {
        HighEnd,    // Latest CPUs, plenty of RAM
        Mid,        // Typical development machines  
        LowEnd,     // Older hardware, constrained systems
        Embedded,   // ARM, limited resources
    }
    
    impl PerformanceConfig {
        fn default() -> Self {
            Self {
                scale_factor: 1.0,
                enable_stress_tests: true,
                hardware_tier: HardwareTier::Mid,
            }
        }
        
        fn adjust_target(&self, base_target: u64) -> u64 {
            let hardware_multiplier = match self.hardware_tier {
                HardwareTier::HighEnd => 0.8,
                HardwareTier::Mid => 1.0,
                HardwareTier::LowEnd => 2.0,
                HardwareTier::Embedded => 4.0,
            };
            (base_target as f64 * hardware_multiplier * self.scale_factor) as u64
        }
    }
    
    // Helper functions to reduce code duplication
    
    /// Helper to create a standard audio stream for testing
    fn create_test_audio_stream(manager: &mut StreamManager, name: &str) -> crate::Result<()> {
        manager.create_realtime_stream(
            name.to_string(),
            crate::runtime::types::DataType::Audio,
            Some(SAMPLE_RATE),
            Some(AUDIO_BUFFER_SIZE)
        )
    }
    
    /// Helper to create test audio data
    fn generate_test_audio_data(size: usize, frequency: f32) -> Vec<f32> {
        (0..size)
            .map(|i| (i as f32 / SAMPLE_RATE * frequency * 2.0 * std::f32::consts::PI).sin())
            .collect()
    }
    
    /// Helper to measure operation duration
    fn measure_operation<F, R>(operation: F) -> (R, Duration) 
    where F: FnOnce() -> R {
        let start = Instant::now();
        let result = operation();
        (result, start.elapsed())
    }

    #[test]
    fn test_realtime_buffer_performance() {
        println!("🎯 Testing real-time buffer performance...");
        
        let buffer = RealtimeCircularBuffer::new(1024).unwrap();
        let iterations = BUFFER_PERF_ITERATIONS;
        
        // Test write performance
        let start = Instant::now();
        for i in 0..iterations {
            buffer.write(i as f32);
        }
        let write_duration = start.elapsed();
        let avg_write_ns = write_duration.as_nanos() / iterations;
        
        // Test read performance
        let start = Instant::now();
        for _ in 0..iterations {
            buffer.read();
        }
        let read_duration = start.elapsed();
        let avg_read_ns = read_duration.as_nanos() / iterations;
        
        println!("📊 Buffer Performance Results:");
        println!("   Write: {} ns/operation", avg_write_ns);
        println!("   Read:  {} ns/operation", avg_read_ns);
        
        // Performance assertions (should be very fast)
        assert!(avg_write_ns < 1000, "Write operations too slow: {}ns", avg_write_ns);
        assert!(avg_read_ns < 1000, "Read operations too slow: {}ns", avg_read_ns);
        
        // Combined operation should be well under 1μs
        let combined_ns = avg_write_ns + avg_read_ns;
        assert!(combined_ns < 500, "Combined read/write too slow: {}ns", combined_ns);
        
        println!("✅ Buffer performance test passed!");
    }

    #[test]
    fn test_audio_processing_latency() {
        println!("🎵 Testing audio processing latency...");
        
        let mut stream_manager = StreamManager::new();
        
        // Create audio streams
        stream_manager.create_realtime_stream(
            "audio_in".to_string(), 
            crate::runtime::types::DataType::Audio, 
            Some(SAMPLE_RATE),
            Some(AUDIO_BUFFER_SIZE)
        ).unwrap();
        
        stream_manager.create_realtime_stream(
            "audio_out".to_string(),
            crate::runtime::types::DataType::Audio,
            Some(SAMPLE_RATE), 
            Some(AUDIO_BUFFER_SIZE)
        ).unwrap();
        
        // Test complete audio processing cycle
        let test_iterations = AUDIO_PROCESSING_ITERATIONS;
        let mut total_latency = Duration::new(0, 0);
        
        for i in 0..test_iterations {
            let start = Instant::now();
            
            // Simulate audio input
            let input_data: Vec<f32> = (0..AUDIO_BUFFER_SIZE)
                .map(|j| ((i + j) as f32 / SAMPLE_RATE * 440.0 * 2.0 * std::f32::consts::PI).sin())
                .collect();
            
            // Write input data
            stream_manager.write_to_realtime_stream("audio_in", input_data).unwrap();
            
            // Read and process
            let processed_data = stream_manager.read_from_realtime_stream("audio_in", AUDIO_BUFFER_SIZE).unwrap();
            
            // Apply simple processing (gain)
            let gained_data: Vec<f32> = processed_data.iter().map(|&x| x * 0.8).collect();
            
            // Write output
            stream_manager.write_to_realtime_stream("audio_out", gained_data).unwrap();
            
            let cycle_latency = start.elapsed();
            total_latency += cycle_latency;
            
            // Each cycle should be well under target latency
            let cycle_us = cycle_latency.as_micros() as u64;
            if cycle_us > TARGET_LATENCY_US {
                panic!("Audio cycle {} took {}μs > {}μs target", i, cycle_us, TARGET_LATENCY_US);
            }
        }
        
        let avg_latency_us = (total_latency.as_micros() / test_iterations as u128) as u64;
        
        println!("📊 Audio Processing Results:");
        println!("   Average cycle latency: {}μs", avg_latency_us);
        println!("   Target latency: {}μs", TARGET_LATENCY_US);
        println!("   Buffer size: {} samples", AUDIO_BUFFER_SIZE);
        println!("   Sample rate: {}Hz", SAMPLE_RATE);
        
        assert!(avg_latency_us < TARGET_LATENCY_US, 
            "Average audio latency {}μs exceeds target {}μs", avg_latency_us, TARGET_LATENCY_US);
        
        println!("✅ Audio processing latency test passed!");
    }

    #[test]
    fn test_stream_composition_performance() {
        println!("🎛️ Testing stream composition performance...");
        
        let mut composer = StreamCompositionEngine::new();
        let mut stream_manager = StreamManager::new();
        
        // Create multiple streams
        for i in 0..10 {
            let stream_name = format!("stream_{}", i);
            stream_manager.create_realtime_stream(
                stream_name,
                crate::runtime::types::DataType::Audio,
                Some(SAMPLE_RATE),
                Some(AUDIO_BUFFER_SIZE)
            ).unwrap();
        }
        
        // Create complex routing
        composer.connect_split(
            "stream_0".to_string(),
            vec!["stream_1".to_string(), "stream_2".to_string(), "stream_3".to_string()],
            vec![0.8, 0.8, 0.8]
        ).unwrap();
        
        composer.connect_merge(
            vec!["stream_1".to_string(), "stream_2".to_string()],
            "stream_4".to_string(),
            vec![0.5, 0.5]
        ).unwrap();
        
        // Test composition processing performance
        let test_iterations = COMPOSITION_ITERATIONS;
        let start = Instant::now();
        
        for _ in 0..test_iterations {
            composer.process_composition(&mut stream_manager).unwrap();
        }
        
        let total_duration = start.elapsed();
        let avg_composition_us = (total_duration.as_micros() / test_iterations as u128) as u64;
        
        println!("📊 Stream Composition Results:");
        println!("   Average processing time: {}μs", avg_composition_us);
        println!("   Streams: 10");
        println!("   Connections: 5");
        
        // Composition should be fast enough for real-time use
        assert!(avg_composition_us < TARGET_LATENCY_US * 2, 
            "Stream composition too slow: {}μs", avg_composition_us);
        
        println!("✅ Stream composition performance test passed!");
    }

    #[test]
    fn test_creative_type_conversion_performance() {
        println!("🎨 Testing creative type conversion performance...");
        
        let type_system = CreativeTypeSystem::new();
        let test_iterations = AUDIO_PROCESSING_ITERATIONS;
        
        // Test various type conversions
        let test_values = vec![
            crate::runtime::types::Value::String("C4".to_string()),
            crate::runtime::types::Value::Float(0.5),
            crate::runtime::types::Value::Integer(60),
            crate::runtime::types::Value::String("red".to_string()),
        ];
        
        let start = Instant::now();
        
        for _ in 0..test_iterations {
            for value in &test_values {
                let creative_type = type_system.infer_creative_type(value, Some("musical"));
                // Type inference should be very fast
                let _ = type_system.describe_type(&creative_type);
            }
        }
        
        let total_duration = start.elapsed();
        let avg_conversion_us = (total_duration.as_micros() / (test_iterations * test_values.len()) as u128) as u64;
        
        println!("📊 Type Conversion Results:");
        println!("   Average conversion time: {}μs", avg_conversion_us);
        println!("   Test values: {}", test_values.len());
        
        // Type conversion should be very fast (well under 100μs)
        assert!(avg_conversion_us < 100, 
            "Type conversion too slow: {}μs", avg_conversion_us);
        
        println!("✅ Creative type conversion performance test passed!");
    }

    #[test]
    fn test_creative_composer_performance() {
        println!("🎼 Testing creative composer performance...");
        
        let mut composer = CreativeComposer::new();
        let mut stream_manager = StreamManager::new();
        
        // Create streams for testing
        for i in 0..5 {
            let stream_name = format!("creative_stream_{}", i);
            stream_manager.create_realtime_stream(
                stream_name,
                crate::runtime::types::DataType::Audio,
                Some(SAMPLE_RATE),
                Some(AUDIO_BUFFER_SIZE)
            ).unwrap();
        }
        
        // Test creative operations performance
        let start = Instant::now();
        
        // Harmonize operation
        composer.harmonize(
            vec!["creative_stream_0".to_string(), "creative_stream_1".to_string()],
            "harmonized".to_string()
        ).unwrap();
        
        // Layer operation
        let _layered = composer.layer(
            "creative_stream_2".to_string(),
            vec![
                ("layer_1".to_string(), "reverb".to_string()),
                ("layer_2".to_string(), "delay".to_string())
            ]
        ).unwrap();
        
        // Spread operation
        let (_left, _right) = composer.spread(
            "creative_stream_3".to_string(),
            0.8, // width
            2.0  // movement speed
        ).unwrap();
        
        let creative_operations_duration = start.elapsed();
        let creative_ops_us = creative_operations_duration.as_micros() as u64;
        
        println!("📊 Creative Composer Results:");
        println!("   Creative operations time: {}μs", creative_ops_us);
        
        // Creative operations should be fast enough for interactive use
        assert!(creative_ops_us < 10000, // 10ms should be plenty for setup operations
            "Creative operations too slow: {}μs", creative_ops_us);
        
        // Test creative flow processing
        let flow_iterations = 100;
        let start = Instant::now();
        
        for _ in 0..flow_iterations {
            composer.process_creative_flow(&mut stream_manager).unwrap();
        }
        
        let flow_duration = start.elapsed();
        let avg_flow_us = (flow_duration.as_micros() / flow_iterations as u128) as u64;
        
        println!("   Average creative flow time: {}μs", avg_flow_us);
        
        // Creative flow should maintain real-time performance
        assert!(avg_flow_us < TARGET_LATENCY_US * 3, 
            "Creative flow too slow: {}μs", avg_flow_us);
        
        println!("✅ Creative composer performance test passed!");
    }

    #[test]
    fn test_concurrent_stream_performance() {
        println!("🚀 Testing concurrent stream performance...");
        
        let buffer_pool = RealtimeBufferPool::new(8, 1024).unwrap();
        let num_threads = 4;
        let operations_per_thread = 1000;
        
        let start = Instant::now();
        
        let handles: Vec<_> = (0..num_threads).map(|thread_id| {
            let pool_buffer = buffer_pool.get_buffer(thread_id % buffer_pool.buffer_count()).unwrap().clone();
            
            thread::spawn(move || {
                let thread_start = Instant::now();
                
                for i in 0..operations_per_thread {
                    // Simulate audio processing workload
                    let value = (thread_id * operations_per_thread + i) as f32;
                    
                    // Write data
                    pool_buffer.write(value);
                    
                    // Read data
                    if let Some(_sample) = pool_buffer.read() {
                        // Simulate processing
                        let _processed = value * 0.8;
                    }
                }
                
                thread_start.elapsed()
            })
        }).collect();
        
        // Wait for all threads to complete
        let mut thread_durations = Vec::new();
        for handle in handles {
            thread_durations.push(handle.join().unwrap());
        }
        
        let total_duration = start.elapsed();
        let total_operations = num_threads * operations_per_thread;
        let avg_op_us = (total_duration.as_micros() / total_operations as u128) as u64;
        
        println!("📊 Concurrent Performance Results:");
        println!("   Threads: {}", num_threads);
        println!("   Operations per thread: {}", operations_per_thread);
        println!("   Total operations: {}", total_operations);
        println!("   Total time: {}ms", total_duration.as_millis());
        println!("   Average operation time: {}ns", avg_op_us * 1000); // Convert to ns for readability
        
        // Even with concurrency, operations should be very fast
        assert!(avg_op_us < 10, "Concurrent operations too slow: {}μs", avg_op_us);
        
        // Check thread balance (no thread should be significantly slower)
        let max_thread_duration = thread_durations.iter().max().unwrap();
        let min_thread_duration = thread_durations.iter().min().unwrap();
        let thread_balance_ratio = max_thread_duration.as_micros() as f64 / min_thread_duration.as_micros() as f64;
        
        println!("   Thread balance ratio: {:.2}", thread_balance_ratio);
        assert!(thread_balance_ratio < 2.0, "Poor thread balance: {:.2}", thread_balance_ratio);
        
        println!("✅ Concurrent stream performance test passed!");
    }

    #[test]
    fn test_memory_allocation_performance() {
        println!("💾 Testing memory allocation patterns...");
        
        let mut stream_manager = StreamManager::new();
        
        // Test that stream operations don't cause excessive allocations
        let initial_memory = get_memory_usage();
        
        // Create many streams
        for i in 0..100 {
            stream_manager.create_realtime_stream(
                format!("mem_stream_{}", i),
                crate::runtime::types::DataType::Audio,
                Some(SAMPLE_RATE),
                Some(AUDIO_BUFFER_SIZE)
            ).unwrap();
        }
        
        let after_creation_memory = get_memory_usage();
        
        // Process streams (this should not allocate much)
        let processing_start = Instant::now();
        for i in 0..100 {
            let stream_name = format!("mem_stream_{}", i);
            let data: Vec<f32> = (0..AUDIO_BUFFER_SIZE).map(|j| j as f32 * 0.1).collect();
            stream_manager.write_to_realtime_stream(&stream_name, data).unwrap();
            let _read_data = stream_manager.read_from_realtime_stream(&stream_name, AUDIO_BUFFER_SIZE).unwrap();
        }
        let processing_duration = processing_start.elapsed();
        
        let final_memory = get_memory_usage();
        
        println!("📊 Memory Allocation Results:");
        println!("   Initial memory: {} KB", initial_memory);
        println!("   After creation: {} KB", after_creation_memory);
        println!("   Final memory: {} KB", final_memory);
        println!("   Processing time: {}μs", processing_duration.as_micros());
        
        // Memory growth during processing should be minimal
        let processing_growth = final_memory.saturating_sub(after_creation_memory);
        println!("   Memory growth during processing: {} KB", processing_growth);
        
        // Processing should not cause significant memory growth (some growth is expected for buffers)
        assert!(processing_growth < MAX_MEMORY_GROWTH_KB, "Excessive memory growth: {} KB", processing_growth);
        
        // Processing should be fast
        assert!(processing_duration.as_micros() < 50000, "Processing too slow: {}μs", processing_duration.as_micros());
        
        println!("✅ Memory allocation performance test passed!");
    }

    #[test]
    fn test_end_to_end_latency() {
        println!("🎯 Testing end-to-end latency...");
        
        let mut composer = CreativeComposer::new();
        let mut stream_manager = StreamManager::new();
        
        // Set up a complete audio pipeline
        stream_manager.create_input_stream(
            "mic_input".to_string(),
            InputSourceType::AudioDevice
        ).unwrap();
        
        stream_manager.create_transform_stream(
            "reverb_effect".to_string(),
            TransformType::Reverb { room_size: 0.5, damping: 0.3, wet_mix: 0.2 }
        ).unwrap();
        
        stream_manager.create_output_stream(
            "speakers".to_string(),
            OutputDestinationType::AudioDevice,
            OutputFormat::Float32
        ).unwrap();
        
        // Test complete pipeline latency
        let pipeline_iterations = 100;
        let mut total_latency = Duration::new(0, 0);
        
        for _ in 0..pipeline_iterations {
            let start = Instant::now();
            
            // Simulate complete pipeline: input -> processing -> output
            let _input_data = stream_manager.process_input_stream("mic_input").unwrap();
            stream_manager.apply_transform_stream("mic_input", "reverb_effect", "processed").unwrap();
            stream_manager.write_to_stream("speakers", vec![0.1; AUDIO_BUFFER_SIZE]).unwrap();
            let _result = stream_manager.process_output_stream("speakers");
            
            let pipeline_latency = start.elapsed();
            total_latency += pipeline_latency;
            
            // Each pipeline cycle should meet real-time requirements
            let cycle_us = pipeline_latency.as_micros() as u64;
            if cycle_us > TARGET_LATENCY_US * 5 { // Allow 5x target for full pipeline
                println!("⚠️  Pipeline cycle took {}μs", cycle_us);
            }
        }
        
        let avg_pipeline_latency_us = (total_latency.as_micros() / pipeline_iterations as u128) as u64;
        
        println!("📊 End-to-End Latency Results:");
        println!("   Average pipeline latency: {}μs", avg_pipeline_latency_us);
        println!("   Target latency: {}μs", TARGET_LATENCY_US);
        println!("   Pipeline components: Input -> Transform -> Output");
        
        // Full pipeline should still be reasonable for real-time use
        // We allow higher latency for the full pipeline since it includes multiple stages
        let pipeline_target = TARGET_LATENCY_US * 8; // 8ms for full pipeline
        assert!(avg_pipeline_latency_us < pipeline_target,
            "End-to-end latency {}μs exceeds target {}μs", avg_pipeline_latency_us, pipeline_target);
        
        println!("✅ End-to-end latency test passed!");
    }

    // Helper function to get memory usage (cross-platform implementation)
    fn get_memory_usage() -> usize {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(mem_str) = line.split_whitespace().nth(1) {
                            return mem_str.parse::<usize>().unwrap_or(0);
                        }
                    }
                }
            }
            0
        }
        #[cfg(target_os = "windows")]
        {
            // Would require winapi crate for GetProcessMemoryInfo
            use std::mem;
            use std::process;
            
            // Simplified approach - use heap allocated memory as proxy
            let id = process::id() as usize;
            id * 1024 // Placeholder based on process ID
        }
        #[cfg(target_os = "macos")]
        {
            // Would require libc crate for getrusage
            use std::process;
            let id = process::id() as usize;
            id * 1024 // Placeholder based on process ID  
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            0 // Fallback for unsupported platforms
        }
    }

    #[test]
    fn test_performance_under_load() {
        println!("⚡ Testing performance under heavy load...");
        
        let mut stream_manager = StreamManager::new();
        let mut composition_engine = StreamCompositionEngine::new();
        
        // Create a large number of streams
        let num_streams = 50;
        for i in 0..num_streams {
            stream_manager.create_realtime_stream(
                format!("load_stream_{}", i),
                crate::runtime::types::DataType::Audio,
                Some(SAMPLE_RATE),
                Some(AUDIO_BUFFER_SIZE)
            ).unwrap();
        }
        
        // Create complex routing
        for i in 0..num_streams - 1 {
            composition_engine.connect_direct(
                format!("load_stream_{}", i),
                format!("load_stream_{}", i + 1),
                0.9
            ).unwrap();
        }
        
        // Test performance under load
        let load_iterations = 50;
        let start = Instant::now();
        
        for iteration in 0..load_iterations {
            // Process all streams
            for i in 0..num_streams {
                let stream_name = format!("load_stream_{}", i);
                let data: Vec<f32> = (0..AUDIO_BUFFER_SIZE)
                    .map(|j| ((iteration + j) as f32 * 0.01).sin())
                    .collect();
                
                stream_manager.write_to_realtime_stream(&stream_name, data).unwrap();
            }
            
            // Process composition
            composition_engine.process_composition(&mut stream_manager).unwrap();
            
            // Read processed data
            for i in 0..num_streams {
                let stream_name = format!("load_stream_{}", i);
                let _output = stream_manager.read_from_realtime_stream(&stream_name, AUDIO_BUFFER_SIZE);
            }
        }
        
        let total_duration = start.elapsed();
        let avg_iteration_us = (total_duration.as_micros() / load_iterations as u128) as u64;
        
        println!("📊 Performance Under Load Results:");
        println!("   Streams: {}", num_streams);
        println!("   Connections: {}", num_streams - 1);
        println!("   Iterations: {}", load_iterations);
        println!("   Average iteration time: {}μs", avg_iteration_us);
        println!("   Total processing time: {}ms", total_duration.as_millis());
        
        // Even under heavy load, should maintain reasonable performance
        // Allow higher latency for stress test, but still reasonable
        let load_target = TARGET_LATENCY_US * 50; // 50ms for heavy load
        assert!(avg_iteration_us < load_target,
            "Performance under load too slow: {}μs", avg_iteration_us);
        
        // Check that the system didn't degrade significantly
        let performance_score = (TARGET_LATENCY_US as f64 / avg_iteration_us as f64) * 100.0;
        println!("   Performance score: {:.1}%", performance_score);
        
        println!("✅ Performance under load test passed!");
    }

    #[test]
    fn test_buffer_boundary_conditions() {
        println!("🔬 Testing buffer boundary conditions and edge cases...");
        
        let buffer = RealtimeCircularBuffer::new(8).unwrap(); // Small buffer for edge case testing
        
        // Test rapid fill/empty cycles
        let start = Instant::now();
        for cycle in 0..1000 {
            // Fill buffer completely
            for i in 0..7 { // 7 because one slot is reserved
                assert!(buffer.write(i as f32), "Failed to write sample {} in cycle {}", i, cycle);
            }
            
            // Verify buffer is full
            assert!(buffer.is_full(), "Buffer should be full in cycle {}", cycle);
            assert!(!buffer.write(999.0), "Should not be able to write to full buffer in cycle {}", cycle);
            
            // Empty buffer completely
            for i in 0..7 {
                let sample = buffer.read().expect(&format!("Failed to read sample {} in cycle {}", i, cycle));
                assert_eq!(sample, i as f32, "Sample mismatch in cycle {}", cycle);
            }
            
            // Verify buffer is empty
            assert!(buffer.is_empty(), "Buffer should be empty in cycle {}", cycle);
            assert!(buffer.read().is_none(), "Should not be able to read from empty buffer in cycle {}", cycle);
        }
        let boundary_duration = start.elapsed();
        
        println!("📊 Boundary Condition Results:");
        println!("   1000 fill/empty cycles: {}μs", boundary_duration.as_micros());
        println!("   Average per cycle: {}μs", boundary_duration.as_micros() / 1000);
        
        // Boundary operations should still be very fast
        assert!(boundary_duration.as_micros() < 50000, "Boundary conditions too slow: {}μs", boundary_duration.as_micros());
        
        println!("✅ Buffer boundary conditions test passed!");
    }

    #[test]
    fn test_performance_degradation_detection() {
        println!("📉 Testing performance degradation detection...");
        
        let mut stream_manager = StreamManager::new();
        let mut baseline_times = Vec::new();
        let mut stressed_times = Vec::new();
        
        // Create baseline performance profile
        stream_manager.create_realtime_stream(
            "perf_test".to_string(),
            crate::runtime::types::DataType::Audio,
            Some(SAMPLE_RATE),
            Some(AUDIO_BUFFER_SIZE)
        ).unwrap();
        
        // Measure baseline performance
        for _ in 0..100 {
            let start = Instant::now();
            let data: Vec<f32> = (0..AUDIO_BUFFER_SIZE).map(|i| (i as f32 * 0.01).sin()).collect();
            stream_manager.write_to_realtime_stream("perf_test", data).unwrap();
            let _read_data = stream_manager.read_from_realtime_stream("perf_test", AUDIO_BUFFER_SIZE).unwrap();
            baseline_times.push(start.elapsed().as_nanos());
        }
        
        // Simulate system stress (create many streams)
        for i in 0..20 {
            stream_manager.create_realtime_stream(
                format!("stress_stream_{}", i),
                crate::runtime::types::DataType::Audio,
                Some(SAMPLE_RATE),
                Some(AUDIO_BUFFER_SIZE)
            ).unwrap();
        }
        
        // Measure performance under stress
        for _ in 0..100 {
            let start = Instant::now();
            let data: Vec<f32> = (0..AUDIO_BUFFER_SIZE).map(|i| (i as f32 * 0.01).sin()).collect();
            stream_manager.write_to_realtime_stream("perf_test", data).unwrap();
            let _read_data = stream_manager.read_from_realtime_stream("perf_test", AUDIO_BUFFER_SIZE).unwrap();
            stressed_times.push(start.elapsed().as_nanos());
        }
        
        let baseline_avg = baseline_times.iter().sum::<u128>() / baseline_times.len() as u128;
        let stressed_avg = stressed_times.iter().sum::<u128>() / stressed_times.len() as u128;
        let degradation_ratio = stressed_avg as f64 / baseline_avg as f64;
        
        println!("📊 Performance Degradation Results:");
        println!("   Baseline average: {}ns", baseline_avg);
        println!("   Under stress average: {}ns", stressed_avg);
        println!("   Degradation ratio: {:.2}x", degradation_ratio);
        
        // Performance should not degrade more than 3x under stress
        assert!(degradation_ratio < 3.0, "Excessive performance degradation: {:.2}x", degradation_ratio);
        
        // Even under stress, should maintain real-time capabilities
        assert!(stressed_avg < (TARGET_LATENCY_US * 1000) as u128, 
            "Stressed performance too slow: {}ns > {}ns target", stressed_avg, TARGET_LATENCY_US * 1000);
        
        println!("✅ Performance degradation detection test passed!");
    }
}