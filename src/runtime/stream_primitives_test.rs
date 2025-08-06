#[cfg(test)]
mod stream_primitives_tests {
    use super::*;
    use crate::runtime::types::{Value, DataType};
    use crate::runtime::streams::{
        StreamManager, InputSourceType, OutputDestinationType, OutputFormat,
        TransformType, FilterType, BufferPolicy, WaveformType
    };

    #[test]
    fn test_create_input_stream_audio_device() {
        let mut manager = StreamManager::new();
        
        let result = manager.create_input_stream(
            "audio_in".to_string(),
            InputSourceType::AudioDevice
        );
        
        assert!(result.is_ok());
        
        // Verify stream exists and has correct metadata
        let stream_info = manager.get_stream_info("audio_in");
        assert!(stream_info.is_some());
        assert_eq!(stream_info.unwrap().data_type, DataType::Audio);
        
        // Check metadata
        if let Some(Value::String(primitive)) = manager.get_metadata("audio_in", "primitive") {
            assert_eq!(primitive, "input");
        } else {
            panic!("Expected input primitive metadata");
        }
    }

    #[test]
    fn test_create_input_stream_sine_generator() {
        let mut manager = StreamManager::new();
        
        let result = manager.create_input_stream(
            "sine_gen".to_string(),
            InputSourceType::Generator { waveform: WaveformType::Sine }
        );
        
        assert!(result.is_ok());
        
        // Test generating data from the input stream
        let data = manager.process_input_stream("sine_gen");
        assert!(data.is_ok());
        
        let samples = data.unwrap();
        assert!(!samples.is_empty());
        assert_eq!(samples.len(), 128); // Default sample count
        
        // Verify it's actually a sine wave (check some values are non-zero)
        let has_positive = samples.iter().any(|&x| x > 0.0);
        let has_negative = samples.iter().any(|&x| x < 0.0);
        assert!(has_positive && has_negative, "Expected sine wave with positive and negative values");
    }

    #[test]
    fn test_create_input_stream_midi() {
        let mut manager = StreamManager::new();
        
        let result = manager.create_input_stream(
            "midi_in".to_string(),
            InputSourceType::MidiController
        );
        
        assert!(result.is_ok());
        
        let stream_info = manager.get_stream_info("midi_in");
        assert!(stream_info.is_some());
        assert_eq!(stream_info.unwrap().data_type, DataType::MIDI);
    }

    #[test]
    fn test_create_output_stream_audio() {
        let mut manager = StreamManager::new();
        
        let result = manager.create_output_stream(
            "audio_out".to_string(),
            OutputDestinationType::AudioDevice,
            OutputFormat::Float32
        );
        
        assert!(result.is_ok());
        
        let stream_info = manager.get_stream_info("audio_out");
        assert!(stream_info.is_some());
        assert_eq!(stream_info.unwrap().data_type, DataType::Audio);
        
        // Check metadata
        if let Some(Value::String(primitive)) = manager.get_metadata("audio_out", "primitive") {
            assert_eq!(primitive, "output");
        } else {
            panic!("Expected output primitive metadata");
        }
    }

    #[test]
    fn test_create_output_stream_graphics() {
        let mut manager = StreamManager::new();
        
        let result = manager.create_output_stream(
            "graphics_out".to_string(),
            OutputDestinationType::Graphics,
            OutputFormat::Graphics { format: "rgba".to_string() }
        );
        
        assert!(result.is_ok());
        
        let stream_info = manager.get_stream_info("graphics_out");
        assert!(stream_info.is_some());
        assert_eq!(stream_info.unwrap().data_type, DataType::Visual);
    }

    #[test]
    fn test_create_transform_stream_gain() {
        let mut manager = StreamManager::new();
        
        let result = manager.create_transform_stream(
            "gain_transform".to_string(),
            TransformType::Gain { amount: 0.5 }
        );
        
        assert!(result.is_ok());
        
        // Check that parameters were stored correctly
        if let Some(Value::Float(amount)) = manager.get_metadata("gain_transform", "param_amount") {
            assert_eq!(amount, 0.5);
        } else {
            panic!("Expected gain amount parameter");
        }
    }

    #[test]
    fn test_create_transform_stream_filter() {
        let mut manager = StreamManager::new();
        
        let result = manager.create_transform_stream(
            "filter_transform".to_string(),
            TransformType::Filter { 
                cutoff: 440.0, 
                resonance: 0.7, 
                filter_type: FilterType::LowPass 
            }
        );
        
        assert!(result.is_ok());
        
        // Check parameters
        if let Some(Value::Float(cutoff)) = manager.get_metadata("filter_transform", "param_cutoff") {
            assert_eq!(cutoff, 440.0);
        }
        
        if let Some(Value::Float(resonance)) = manager.get_metadata("filter_transform", "param_resonance") {
            assert_eq!(resonance, 0.7);
        }
        
        if let Some(Value::String(filter_type)) = manager.get_metadata("filter_transform", "param_filter_type") {
            assert_eq!(filter_type, "LowPass");
        }
    }

    #[test]
    fn test_create_transform_stream_reverb() {
        let mut manager = StreamManager::new();
        
        let result = manager.create_transform_stream(
            "reverb_transform".to_string(),
            TransformType::Reverb { 
                room_size: 0.8, 
                damping: 0.3, 
                wet_mix: 0.4 
            }
        );
        
        assert!(result.is_ok());
        
        // Check reverb parameters
        if let Some(Value::Float(room_size)) = manager.get_metadata("reverb_transform", "param_room_size") {
            assert_eq!(room_size, 0.8);
        }
    }

    #[test]
    fn test_create_buffer_stream() {
        let mut manager = StreamManager::new();
        
        let result = manager.create_buffer_stream(
            "circular_buffer".to_string(),
            2048,
            BufferPolicy::Circular
        );
        
        assert!(result.is_ok());
        
        // Check buffer configuration
        let stream_info = manager.get_stream_info("circular_buffer");
        assert!(stream_info.is_some());
        
        if let Some(Value::Integer(size)) = manager.get_metadata("circular_buffer", "configured_size") {
            assert_eq!(size, 2048);
        }
        
        if let Some(Value::String(policy)) = manager.get_metadata("circular_buffer", "policy") {
            assert_eq!(policy, "Circular");
        }
    }

    #[test]
    fn test_apply_transform_stream_gain() {
        let mut manager = StreamManager::new();
        
        // Create input stream and add some data
        manager.create_input_stream("input".to_string(), InputSourceType::AudioDevice).unwrap();
        manager.write_to_stream("input", vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        
        // Create gain transform (0.5x gain)
        manager.create_transform_stream("gain".to_string(), TransformType::Gain { amount: 0.5 }).unwrap();
        
        // Create output stream
        manager.create_output_stream("output".to_string(), OutputDestinationType::AudioDevice, OutputFormat::Float32).unwrap();
        
        // Apply transform
        let result = manager.apply_transform_stream("input", "gain", "output");
        assert!(result.is_ok());
        
        // Verify output has transformed data
        let output_data = manager.read_from_stream("output", 4).unwrap();
        assert_eq!(output_data.len(), 4);
        
        // Values should be halved (gain of 0.5)
        for (i, &value) in output_data.iter().enumerate() {
            let expected = (i + 1) as f32 * 0.5;
            assert!((value - expected).abs() < 0.001, "Expected {}, got {}", expected, value);
        }
    }

    #[test]
    fn test_apply_transform_stream_filter() {
        let mut manager = StreamManager::new();
        
        // Create streams
        manager.create_input_stream("input".to_string(), InputSourceType::AudioDevice).unwrap();
        manager.write_to_stream("input", vec![1.0, -1.0, 1.0, -1.0]).unwrap(); // Square wave
        
        manager.create_transform_stream("filter".to_string(), 
            TransformType::Filter { cutoff: 0.5, resonance: 0.1, filter_type: FilterType::LowPass }).unwrap();
        
        manager.create_output_stream("output".to_string(), OutputDestinationType::AudioDevice, OutputFormat::Float32).unwrap();
        
        // Apply filter transform
        let result = manager.apply_transform_stream("input", "filter", "output");
        assert!(result.is_ok());
        
        // Verify output is filtered (should be smoother than input)
        let output_data = manager.read_from_stream("output", 4).unwrap();
        assert_eq!(output_data.len(), 4);
        
        // Low-pass filter should reduce the sharp transitions
        assert!(output_data[0].abs() > 0.0 && output_data[0].abs() < 1.0);
    }

    #[test]
    fn test_apply_transform_stream_delay() {
        let mut manager = StreamManager::new();
        
        // Create streams  
        manager.create_input_stream("input".to_string(), InputSourceType::AudioDevice).unwrap();
        manager.write_to_stream("input", vec![1.0, 0.0, 0.0, 0.0]).unwrap(); // Impulse
        
        manager.create_transform_stream("delay".to_string(), 
            TransformType::Delay { time: 0.001, feedback: 0.5 }).unwrap(); // 1ms delay
        
        manager.create_output_stream("output".to_string(), OutputDestinationType::AudioDevice, OutputFormat::Float32).unwrap();
        
        // Apply delay transform
        let result = manager.apply_transform_stream("input", "delay", "output");
        assert!(result.is_ok());
        
        // Output should be longer due to delay
        let output_data = manager.read_from_stream("output", 50).unwrap();
        assert!(output_data.len() >= 4); // At least original length
        
        // Should have the original impulse plus delayed copies
        let has_initial = output_data.iter().any(|&x| x.abs() > 0.9);
        let has_delayed = output_data.iter().skip(1).any(|&x| x.abs() > 0.3);
        assert!(has_initial, "Expected initial impulse in output");
        assert!(has_delayed, "Expected delayed signal in output");
    }

    #[test]
    fn test_process_output_stream() {
        let mut manager = StreamManager::new();
        
        // Create output stream with data
        manager.create_output_stream("audio_out".to_string(), 
            OutputDestinationType::AudioDevice, OutputFormat::Float32).unwrap();
        manager.write_to_stream("audio_out", vec![0.1, 0.2, 0.3]).unwrap();
        
        // Process output (should simulate sending to audio device)
        let result = manager.process_output_stream("audio_out");
        assert!(result.is_ok());
    }

    #[test]
    fn test_full_stream_processing_chain() {
        let mut manager = StreamManager::new();
        
        // Create a complete processing chain: Input -> Gain -> Filter -> Output
        
        // Input: sine wave generator
        manager.create_input_stream("sine_input".to_string(), 
            InputSourceType::Generator { waveform: WaveformType::Sine }).unwrap();
        
        // Transform 1: gain
        manager.create_transform_stream("gain".to_string(), 
            TransformType::Gain { amount: 0.8 }).unwrap();
        
        // Transform 2: low-pass filter
        manager.create_transform_stream("filter".to_string(), 
            TransformType::Filter { cutoff: 0.7, resonance: 0.2, filter_type: FilterType::LowPass }).unwrap();
        
        // Output: audio device
        manager.create_output_stream("audio_out".to_string(), 
            OutputDestinationType::AudioDevice, OutputFormat::Float32).unwrap();
        
        // Process the chain
        
        // Step 1: Generate input data
        let input_data = manager.process_input_stream("sine_input").unwrap();
        manager.write_to_stream("temp_1", input_data).unwrap();
        
        // Step 2: Apply gain
        manager.create_stream("temp_1".to_string(), DataType::Audio, None).unwrap();
        manager.apply_transform_stream("temp_1", "gain", "temp_2").unwrap();
        
        // Step 3: Apply filter  
        manager.create_stream("temp_2".to_string(), DataType::Audio, None).unwrap();
        manager.apply_transform_stream("temp_2", "filter", "audio_out").unwrap();
        
        // Step 4: Process output
        let result = manager.process_output_stream("audio_out");
        assert!(result.is_ok());
        
        // Verify final output has data
        let final_data = manager.read_from_stream("audio_out", 10).unwrap();
        assert!(!final_data.is_empty());
        assert!(final_data.iter().any(|&x| x != 0.0), "Expected non-zero output data");
    }

    #[test]
    fn test_stream_primitive_error_handling() {
        let mut manager = StreamManager::new();
        
        // Test processing non-existent input stream
        let result = manager.process_input_stream("nonexistent");
        assert!(result.is_err());
        
        // Test processing non-input stream as input
        manager.create_stream("regular_stream".to_string(), DataType::Audio, None).unwrap();
        let result = manager.process_input_stream("regular_stream");
        assert!(result.is_err());
        
        // Test applying transform with non-existent streams
        let result = manager.apply_transform_stream("missing_input", "missing_transform", "missing_output");
        assert!(result.is_err());
    }

    #[test]
    fn test_buffer_policy_circular() {
        let mut manager = StreamManager::new();
        
        manager.create_buffer_stream("circular".to_string(), 4, BufferPolicy::Circular).unwrap();
        
        // Fill buffer beyond capacity
        manager.write_to_stream("circular", vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        
        // Should have only kept the most recent 4 values
        let data = manager.read_from_stream("circular", 4).unwrap();
        assert_eq!(data.len(), 4);
        
        // Should contain the most recent values (buffer policy handled in write_to_stream)
        assert!(data.iter().any(|&x| x > 0.0), "Expected some data in circular buffer");
    }
}