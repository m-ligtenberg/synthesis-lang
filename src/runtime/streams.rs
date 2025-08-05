use crate::runtime::types::{DataType, Value, Stream};
use crate::errors::ErrorKind;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub name: String,
    pub data_type: DataType,
    pub sample_rate: Option<f32>,
    pub buffer_size: usize,
    pub is_active: bool,
    pub age: Duration,
    pub processor_count: usize,
}

#[derive(Debug)]
pub struct StreamManager {
    streams: HashMap<String, Arc<Mutex<StreamData>>>,
    connections: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct StreamData {
    pub name: String,
    pub data_type: DataType,
    pub sample_rate: Option<f32>,
    pub buffer: Vec<f32>,
    pub position: usize,
    pub is_active: bool,
    pub timestamp: Instant,
    pub metadata: HashMap<String, Value>,
    pub processing_chain: Vec<StreamProcessor>,
}

#[derive(Debug, Clone)]
pub enum StreamProcessor {
    Filter { cutoff: f32, resonance: f32 },
    Gain { amount: f32 },
    Delay { time: f32, feedback: f32 },
    Compressor { threshold: f32, ratio: f32 },
    Transform { function: StreamTransformFunction },
}

#[derive(Debug, Clone)]
pub enum StreamTransformFunction {
    Map,      // Transform each value
    Filter,   // Filter based on condition
    Reduce,   // Accumulate values
    Window,   // Sliding window operations
    FFT,      // Frequency domain transform
    Envelope, // ADSR envelope
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
            connections: HashMap::new(),
        }
    }
    
    pub fn create_stream(&mut self, name: String, data_type: DataType, sample_rate: Option<f32>) -> crate::Result<()> {
        let stream_data = StreamData {
            name: name.clone(),
            data_type,
            sample_rate,
            buffer: Vec::new(),
            position: 0,
            is_active: false,
            timestamp: Instant::now(),
            metadata: HashMap::new(),
            processing_chain: Vec::new(),
        };
        
        self.streams.insert(name, Arc::new(Mutex::new(stream_data)));
        Ok(())
    }
    
    pub fn connect(&mut self, source: String, destination: String) -> crate::Result<()> {
        if !self.streams.contains_key(&source) {
            return Err(crate::SynthesisError::new(crate::ErrorKind::UnknownModule, format!("Source stream '{}' does not exist", source)));
        }
        
        if !self.streams.contains_key(&destination) {
            return Err(crate::SynthesisError::new(crate::ErrorKind::UnknownModule, format!("Destination stream '{}' does not exist", destination)));
        }
        
        self.connections
            .entry(source)
            .or_insert_with(Vec::new)
            .push(destination);
        
        Ok(())
    }
    
    pub fn get_stream(&self, name: &str) -> Option<Arc<Mutex<StreamData>>> {
        self.streams.get(name).cloned()
    }
    
    pub fn write_to_stream(&self, name: &str, data: Vec<f32>) -> crate::Result<()> {
        if let Some(stream) = self.streams.get(name) {
            let mut stream_data = stream.lock().unwrap();
            stream_data.buffer.extend(data);
            stream_data.is_active = true;
            Ok(())
        } else {
            Err(crate::SynthesisError::new(crate::ErrorKind::UnknownModule, format!("Stream '{}' not found", name)))
        }
    }
    
    pub fn read_from_stream(&self, name: &str, count: usize) -> crate::Result<Vec<f32>> {
        if let Some(stream) = self.streams.get(name) {
            let mut stream_data = stream.lock().unwrap();
            
            if stream_data.buffer.len() < count {
                return Ok(vec![0.0; count]);
            }
            
            let data = stream_data.buffer[..count].to_vec();
            stream_data.buffer.drain(..count);
            Ok(data)
        } else {
            Err(crate::SynthesisError::new(crate::ErrorKind::UnknownModule, format!("Stream '{}' not found", name)))
        }
    }
    
    pub fn process_connections(&mut self) -> crate::Result<()> {
        for (source_name, destinations) in &self.connections {
            if let Some(source_stream) = self.streams.get(source_name).cloned() {
                let source_data = {
                    let stream = source_stream.lock().unwrap();
                    stream.buffer.clone()
                };
                
                for dest_name in destinations {
                    if let Some(dest_stream) = self.streams.get(dest_name) {
                        let mut dest_stream_data = dest_stream.lock().unwrap();
                        dest_stream_data.buffer.extend(&source_data);
                    }
                }
            }
        }
        Ok(())
    }
    
    pub fn get_stream_value(&self, name: &str) -> Value {
        if let Some(stream) = self.streams.get(name) {
            let stream_data = stream.lock().unwrap();
            Value::Stream(crate::runtime::types::Stream {
                name: stream_data.name.clone(),
                data_type: stream_data.data_type.clone(),
                sample_rate: stream_data.sample_rate,
            })
        } else {
            Value::Null
        }
    }
    
    // Enhanced stream processing methods
    
    pub fn add_processor(&mut self, stream_name: &str, processor: StreamProcessor) -> crate::Result<()> {
        if let Some(stream) = self.streams.get(stream_name) {
            let mut stream_data = stream.lock().unwrap();
            stream_data.processing_chain.push(processor);
            Ok(())
        } else {
            Err(crate::SynthesisError::new(ErrorKind::UnknownModule, format!("Stream '{}' not found", stream_name)))
        }
    }
    
    pub fn set_metadata(&mut self, stream_name: &str, key: String, value: Value) -> crate::Result<()> {
        if let Some(stream) = self.streams.get(stream_name) {
            let mut stream_data = stream.lock().unwrap();
            stream_data.metadata.insert(key, value);
            Ok(())
        } else {
            Err(crate::SynthesisError::new(ErrorKind::UnknownModule, format!("Stream '{}' not found", stream_name)))
        }
    }
    
    pub fn get_metadata(&self, stream_name: &str, key: &str) -> Option<Value> {
        if let Some(stream) = self.streams.get(stream_name) {
            let stream_data = stream.lock().unwrap();
            stream_data.metadata.get(key).cloned()
        } else {
            None
        }
    }
    
    pub fn process_stream_data(&self, stream_name: &str) -> crate::Result<Vec<f32>> {
        if let Some(stream) = self.streams.get(stream_name) {
            let mut stream_data = stream.lock().unwrap();
            let mut data = stream_data.buffer.clone();
            
            // Apply processing chain
            for processor in &stream_data.processing_chain {
                data = self.apply_processor(processor, data)?;
            }
            
            Ok(data)
        } else {
            Err(crate::SynthesisError::new(ErrorKind::UnknownModule, format!("Stream '{}' not found", stream_name)))
        }
    }
    
    fn apply_processor(&self, processor: &StreamProcessor, mut data: Vec<f32>) -> crate::Result<Vec<f32>> {
        match processor {
            StreamProcessor::Gain { amount } => {
                for sample in &mut data {
                    *sample *= amount;
                }
                Ok(data)
            }
            StreamProcessor::Filter { cutoff, resonance: _ } => {
                // Simple low-pass filter implementation
                let mut filtered = Vec::with_capacity(data.len());
                let mut prev = 0.0;
                let alpha = (*cutoff).min(1.0).max(0.0);
                
                for sample in data {
                    let filtered_sample = prev + alpha * (sample - prev);
                    filtered.push(filtered_sample);
                    prev = filtered_sample;
                }
                Ok(filtered)
            }
            StreamProcessor::Delay { time, feedback } => {
                let delay_samples = (*time * 44100.0) as usize; // Assume 44.1kHz sample rate
                let mut delayed = vec![0.0; delay_samples];
                delayed.extend(data);
                
                // Apply feedback
                if *feedback > 0.0 {
                    for i in delay_samples..delayed.len() {
                        if i >= delay_samples {
                            delayed[i] += delayed[i - delay_samples] * feedback;
                        }
                    }
                }
                
                Ok(delayed)
            }
            StreamProcessor::Compressor { threshold, ratio } => {
                for sample in &mut data {
                    let abs_sample = sample.abs();
                    if abs_sample > *threshold {
                        let excess = abs_sample - threshold;
                        let compressed_excess = excess / ratio;
                        let sign = if *sample >= 0.0 { 1.0 } else { -1.0 };
                        *sample = sign * (threshold + compressed_excess);
                    }
                }
                Ok(data)
            }
            StreamProcessor::Transform { function } => {
                match function {
                    StreamTransformFunction::Map => Ok(data), // Identity for now
                    StreamTransformFunction::Filter => {
                        // Remove values below threshold
                        Ok(data.into_iter().filter(|&x| x.abs() > 0.1).collect())
                    }
                    StreamTransformFunction::Window => {
                        // Apply window function (Hanning window)
                        let len = data.len();
                        for (i, sample) in data.iter_mut().enumerate() {
                            let window_val = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / len as f32).cos();
                            *sample *= window_val;
                        }
                        Ok(data)
                    }
                    StreamTransformFunction::Envelope => {
                        // Simple ADSR envelope (attack only for now)
                        let attack_time = 0.1; // 100ms attack
                        let attack_samples = (attack_time * 44100.0) as usize;
                        
                        for (i, sample) in data.iter_mut().enumerate() {
                            if i < attack_samples {
                                let envelope = i as f32 / attack_samples as f32;
                                *sample *= envelope;
                            }
                        }
                        Ok(data)
                    }
                    _ => Ok(data), // FFT and Reduce not implemented yet
                }
            }
        }
    }
    
    pub fn fork_stream(&mut self, source_name: &str, new_name: String) -> crate::Result<()> {
        if let Some(source_stream) = self.streams.get(source_name).cloned() {
            let source_data = source_stream.lock().unwrap();
            let mut new_stream_data = source_data.clone();
            new_stream_data.name = new_name.clone();
            new_stream_data.timestamp = Instant::now();
            
            self.streams.insert(new_name, Arc::new(Mutex::new(new_stream_data)));
            Ok(())
        } else {
            Err(crate::SynthesisError::new(ErrorKind::UnknownModule, format!("Source stream '{}' not found", source_name)))
        }
    }
    
    pub fn merge_streams(&mut self, stream_names: Vec<String>, output_name: String) -> crate::Result<()> {
        let mut merged_buffer = Vec::new();
        let mut merged_metadata = HashMap::new();
        let mut sample_rate = None;
        
        for stream_name in &stream_names {
            if let Some(stream) = self.streams.get(stream_name) {
                let stream_data = stream.lock().unwrap();
                
                // Mix audio data
                if merged_buffer.is_empty() {
                    merged_buffer = stream_data.buffer.clone();
                } else {
                    let min_len = merged_buffer.len().min(stream_data.buffer.len());
                    for i in 0..min_len {
                        merged_buffer[i] += stream_data.buffer[i];
                    }
                    // Extend if one stream is longer
                    if stream_data.buffer.len() > merged_buffer.len() {
                        merged_buffer.extend_from_slice(&stream_data.buffer[merged_buffer.len()..]);
                    }
                }
                
                // Merge metadata
                for (key, value) in &stream_data.metadata {
                    merged_metadata.insert(format!("{}_{}", stream_name, key), value.clone());
                }
                
                // Use first non-None sample rate
                if sample_rate.is_none() {
                    sample_rate = stream_data.stream.sample_rate;
                }
            }
        }
        
        // Normalize merged audio
        if !merged_buffer.is_empty() {
            let max_val = merged_buffer.iter().map(|x| x.abs()).fold(0.0, f32::max);
            if max_val > 1.0 {
                for sample in &mut merged_buffer {
                    *sample /= max_val;
                }
            }
        }
        
        // Create merged stream
        let merged_stream = Stream {
            name: output_name.clone(),
            data_type: DataType::Audio, // Default to audio for mixed streams
            sample_rate,
        };
        
        let merged_stream_data = StreamData {
            stream: merged_stream,
            buffer: merged_buffer,
            position: 0,
            is_active: true,
            timestamp: Instant::now(),
            metadata: merged_metadata,
            processing_chain: Vec::new(),
        };
        
        self.streams.insert(output_name, Arc::new(Mutex::new(merged_stream_data)));
        Ok(())
    }
    
    pub fn get_stream_info(&self, name: &str) -> Option<StreamInfo> {
        if let Some(stream) = self.streams.get(name) {
            let stream_data = stream.lock().unwrap();
            Some(StreamInfo {
                name: stream_data.stream.name.clone(),
                data_type: stream_data.stream.data_type.clone(),
                sample_rate: stream_data.stream.sample_rate,
                buffer_size: stream_data.buffer.len(),
                is_active: stream_data.is_active,
                age: stream_data.timestamp.elapsed(),
                processor_count: stream_data.processing_chain.len(),
            })
        } else {
            None
        }
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}