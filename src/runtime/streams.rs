use crate::runtime::types::{DataType, Value};
use crate::errors::ErrorKind;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};

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
    streams: HashMap<String, Arc<RwLock<StreamData>>>,
    connections: HashMap<String, Vec<String>>,
    processing_scheduler: Option<ProcessingScheduler>,
    real_time_config: RealTimeConfig,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
}

#[derive(Debug, Clone)]
pub struct RealTimeConfig {
    pub target_latency_ms: f32,
    pub buffer_size: usize,
    pub sample_rate: f32,
    pub max_processing_time_us: u64,
    pub enable_parallel_processing: bool,
    pub gc_threshold: usize,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub processing_time_avg_us: f64,
    pub processing_time_max_us: u64,
    pub buffer_underruns: u64,
    pub buffer_overruns: u64,
    pub streams_processed: u64,
    pub last_reset: Instant,
}

#[derive(Debug)]
pub struct ProcessingScheduler {
    task_queue: Arc<Mutex<VecDeque<StreamTask>>>,
    worker_handles: Vec<thread::JoinHandle<()>>,
    shutdown_flag: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct StreamTask {
    pub stream_name: String,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    Process,
    Connect,
    Transform,
    Merge,
    Fork,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Critical,  // Audio processing, <1ms
    High,      // Graphics, <16ms
    Medium,    // Control signals
    Low,       // Cleanup, metadata
}

#[derive(Debug, Clone)]
pub struct StreamData {
    pub name: String,
    pub data_type: DataType,
    pub sample_rate: Option<f32>,
    pub buffer: VecDeque<f32>,
    pub max_buffer_size: usize,
    pub position: usize,
    pub is_active: bool,
    pub timestamp: Instant,
    pub metadata: HashMap<String, Value>,
    pub processing_chain: Vec<StreamProcessor>,
    pub latency_target: Duration,
    pub last_processed: Option<Instant>,
    pub processing_time_us: u64,
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
        Self::with_config(RealTimeConfig::default())
    }
    
    pub fn with_config(config: RealTimeConfig) -> Self {
        let performance_metrics = Arc::new(Mutex::new(PerformanceMetrics {
            processing_time_avg_us: 0.0,
            processing_time_max_us: 0,
            buffer_underruns: 0,
            buffer_overruns: 0,
            streams_processed: 0,
            last_reset: Instant::now(),
        }));
        
        Self {
            streams: HashMap::new(),
            connections: HashMap::new(),
            processing_scheduler: None,
            real_time_config: config,
            performance_metrics,
        }
    }
    
    pub fn start_real_time_processing(&mut self) -> crate::Result<()> {
        if self.processing_scheduler.is_some() {
            return Ok(()); // Already started
        }
        
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let task_queue = Arc::new(Mutex::new(VecDeque::new()));
        let mut worker_handles = Vec::new();
        
        // Create worker threads based on CPU count and config
        let worker_count = if self.real_time_config.enable_parallel_processing {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
                .min(8) // Cap at 8 threads
        } else {
            1
        };
        
        for i in 0..worker_count {
            let queue_clone = Arc::clone(&task_queue);
            let metrics_clone = Arc::clone(&self.performance_metrics);
            let config = self.real_time_config.clone();
            let shutdown_flag_clone = Arc::clone(&shutdown_flag);
            
            let handle = thread::Builder::new()
                .name(format!("synthesis-stream-worker-{}", i))
                .spawn(move || {
                    Self::worker_thread(queue_clone, metrics_clone, config, shutdown_flag_clone);
                })
                .map_err(|e| crate::SynthesisError::new(ErrorKind::AudioDeviceError, format!("Failed to spawn worker thread: {}", e)))?;
            
            worker_handles.push(handle);
        }
        
        self.processing_scheduler = Some(ProcessingScheduler {
            task_queue,
            worker_handles,
            shutdown_flag,
        });
        
        Ok(())
    }
    
    fn worker_thread(
        task_queue: Arc<Mutex<VecDeque<StreamTask>>>,
        metrics: Arc<Mutex<PerformanceMetrics>>,
        config: RealTimeConfig,
        shutdown_flag: Arc<AtomicBool>,
    ) {
        loop {
            // Check for shutdown signal
            if shutdown_flag.load(Ordering::Relaxed) {
                break;
            }
            
            let task = {
                let mut queue = task_queue.lock().unwrap();
                queue.pop_front()
            };
            
            if let Some(task) = task {
                let start_time = Instant::now();
                
                // Process task based on priority and type
                Self::process_task(&task, &config);
                
                let processing_time = start_time.elapsed().as_micros() as u64;
                
                // Update metrics
                {
                    let mut metrics = metrics.lock().unwrap();
                    metrics.streams_processed += 1;
                    metrics.processing_time_max_us = metrics.processing_time_max_us.max(processing_time);
                    
                    // Update rolling average
                    let alpha = 0.1; // Exponential moving average factor
                    metrics.processing_time_avg_us = 
                        alpha * processing_time as f64 + (1.0 - alpha) * metrics.processing_time_avg_us;
                }
                
                // Check if we exceeded our time budget
                if processing_time > config.max_processing_time_us {
                    eprintln!("Warning: Stream processing exceeded time budget: {}μs > {}μs", 
                             processing_time, config.max_processing_time_us);
                }
            } else {
                // No tasks available - yield CPU briefly
                thread::sleep(Duration::from_micros(10));
            }
        }
    }
    
    fn process_task(task: &StreamTask, _config: &RealTimeConfig) {
        // Task processing implementation would go here
        // For now, just a placeholder that simulates work
        match task.task_type {
            TaskType::Process => {
                // Simulate audio processing
                thread::sleep(Duration::from_micros(50));
            }
            TaskType::Connect | TaskType::Transform | TaskType::Merge | TaskType::Fork => {
                // Simulate lighter processing
                thread::sleep(Duration::from_micros(10));
            }
        }
    }
    
    pub fn create_stream(&mut self, name: String, data_type: DataType, sample_rate: Option<f32>) -> crate::Result<()> {
        let latency_target = Duration::from_millis(self.real_time_config.target_latency_ms as u64);
        let max_buffer_size = self.real_time_config.buffer_size;
        
        let stream_data = StreamData {
            name: name.clone(),
            data_type,
            sample_rate,
            buffer: VecDeque::with_capacity(max_buffer_size),
            max_buffer_size,
            position: 0,
            is_active: false,
            timestamp: Instant::now(),
            metadata: HashMap::new(),
            processing_chain: Vec::new(),
            latency_target,
            last_processed: None,
            processing_time_us: 0,
        };
        
        self.streams.insert(name, Arc::new(RwLock::new(stream_data)));
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
    
    pub fn get_stream(&self, name: &str) -> Option<Arc<RwLock<StreamData>>> {
        self.streams.get(name).cloned()
    }
    
    pub fn write_to_stream(&self, name: &str, data: Vec<f32>) -> crate::Result<()> {
        if let Some(stream) = self.streams.get(name) {
            let mut stream_data = stream.write().unwrap();
            
            // Check for buffer overflow and handle gracefully
            let available_space = stream_data.max_buffer_size.saturating_sub(stream_data.buffer.len());
            if data.len() > available_space {
                // Buffer overflow - update metrics and truncate data
                {
                    let mut metrics = self.performance_metrics.lock().unwrap();
                    metrics.buffer_overruns += 1;
                }
                
                // Keep the most recent data
                let keep_count = available_space;
                if keep_count > 0 {
                    let start_idx = data.len() - keep_count;
                    for &sample in &data[start_idx..] {
                        stream_data.buffer.push_back(sample);
                    }
                }
            } else {
                // Normal case - add all data
                for &sample in &data {
                    stream_data.buffer.push_back(sample);
                }
            }
            
            stream_data.is_active = true;
            stream_data.timestamp = Instant::now();
            Ok(())
        } else {
            Err(crate::SynthesisError::new(crate::ErrorKind::UnknownModule, format!("Stream '{}' not found", name)))
        }
    }
    
    pub fn read_from_stream(&self, name: &str, count: usize) -> crate::Result<Vec<f32>> {
        if let Some(stream) = self.streams.get(name) {
            let mut stream_data = stream.write().unwrap();
            
            if stream_data.buffer.len() < count {
                // Buffer underrun - update metrics and return zeros
                {
                    let mut metrics = self.performance_metrics.lock().unwrap();
                    metrics.buffer_underruns += 1;
                }
                return Ok(vec![0.0; count]);
            }
            
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                if let Some(sample) = stream_data.buffer.pop_front() {
                    data.push(sample);
                } else {
                    data.push(0.0);
                }
            }
            
            stream_data.position += count;
            Ok(data)
        } else {
            Err(crate::SynthesisError::new(crate::ErrorKind::UnknownModule, format!("Stream '{}' not found", name)))
        }
    }
    
    pub fn process_connections(&mut self) -> crate::Result<()> {
        for (source_name, destinations) in &self.connections {
            if let Some(source_stream) = self.streams.get(source_name).cloned() {
                let source_data = {
                    let stream = source_stream.read().unwrap();
                    stream.buffer.iter().cloned().collect::<Vec<f32>>()
                };
                
                for dest_name in destinations {
                    if let Some(dest_stream) = self.streams.get(dest_name) {
                        let mut dest_stream_data = dest_stream.write().unwrap();
                        for &sample in &source_data {
                            dest_stream_data.buffer.push_back(sample);
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    pub fn get_stream_value(&self, name: &str) -> Value {
        if let Some(stream) = self.streams.get(name) {
            let stream_data = stream.read().unwrap();
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
            let mut stream_data = stream.write().unwrap();
            stream_data.processing_chain.push(processor);
            Ok(())
        } else {
            Err(crate::SynthesisError::new(ErrorKind::UnknownModule, format!("Stream '{}' not found", stream_name)))
        }
    }
    
    pub fn set_metadata(&mut self, stream_name: &str, key: String, value: Value) -> crate::Result<()> {
        if let Some(stream) = self.streams.get(stream_name) {
            let mut stream_data = stream.write().unwrap();
            stream_data.metadata.insert(key, value);
            Ok(())
        } else {
            Err(crate::SynthesisError::new(ErrorKind::UnknownModule, format!("Stream '{}' not found", stream_name)))
        }
    }
    
    pub fn get_metadata(&self, stream_name: &str, key: &str) -> Option<Value> {
        if let Some(stream) = self.streams.get(stream_name) {
            let stream_data = stream.read().unwrap();
            stream_data.metadata.get(key).cloned()
        } else {
            None
        }
    }
    
    pub fn process_stream_data(&self, stream_name: &str) -> crate::Result<Vec<f32>> {
        if let Some(stream) = self.streams.get(stream_name) {
            let mut stream_data = stream.write().unwrap();
            let mut data = stream_data.buffer.iter().cloned().collect::<Vec<f32>>();
            
            // Apply processing chain
            for processor in &stream_data.processing_chain.clone() {
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
            let source_data = source_stream.read().unwrap();
            let mut new_stream_data = source_data.clone();
            new_stream_data.name = new_name.clone();
            new_stream_data.timestamp = Instant::now();
            
            self.streams.insert(new_name, Arc::new(RwLock::new(new_stream_data)));
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
                let stream_data = stream.read().unwrap();
                let buffer_data: Vec<f32> = stream_data.buffer.iter().cloned().collect();
                
                // Mix audio data
                if merged_buffer.is_empty() {
                    merged_buffer = buffer_data;
                } else {
                    let min_len = merged_buffer.len().min(buffer_data.len());
                    for i in 0..min_len {
                        merged_buffer[i] += buffer_data[i];
                    }
                    // Extend if one stream is longer
                    if buffer_data.len() > merged_buffer.len() {
                        merged_buffer.extend_from_slice(&buffer_data[merged_buffer.len()..]);
                    }
                }
                
                // Merge metadata
                for (key, value) in &stream_data.metadata {
                    merged_metadata.insert(format!("{}_{}", stream_name, key), value.clone());
                }
                
                // Use first non-None sample rate
                if sample_rate.is_none() {
                    sample_rate = stream_data.sample_rate;
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
        
        let merged_stream_data = StreamData {
            name: output_name.clone(),
            data_type: DataType::Audio, // Default to audio for mixed streams
            sample_rate,
            buffer: merged_buffer.into_iter().collect(),
            max_buffer_size: self.real_time_config.buffer_size,
            position: 0,
            is_active: true,
            timestamp: Instant::now(),
            metadata: merged_metadata,
            processing_chain: Vec::new(),
            latency_target: Duration::from_millis(self.real_time_config.target_latency_ms as u64),
            last_processed: None,
            processing_time_us: 0,
        };
        
        self.streams.insert(output_name, Arc::new(RwLock::new(merged_stream_data)));
        Ok(())
    }
    
    pub fn get_stream_info(&self, name: &str) -> Option<StreamInfo> {
        if let Some(stream) = self.streams.get(name) {
            let stream_data = stream.read().unwrap();
            Some(StreamInfo {
                name: stream_data.name.clone(),
                data_type: stream_data.data_type.clone(),
                sample_rate: stream_data.sample_rate,
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

impl Default for RealTimeConfig {
    fn default() -> Self {
        Self {
            target_latency_ms: 1.0,  // 1ms target latency
            buffer_size: 4096,       // 4KB buffer
            sample_rate: 44100.0,    // CD quality
            max_processing_time_us: 500, // 0.5ms max processing time
            enable_parallel_processing: true,
            gc_threshold: 1024,      // Trigger GC when buffers exceed this size
        }
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamManager {
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.lock().unwrap().clone()
    }
    
    pub fn reset_performance_metrics(&mut self) {
        let mut metrics = self.performance_metrics.lock().unwrap();
        *metrics = PerformanceMetrics {
            processing_time_avg_us: 0.0,
            processing_time_max_us: 0,
            buffer_underruns: 0,
            buffer_overruns: 0,
            streams_processed: 0,
            last_reset: Instant::now(),
        };
    }
    
    pub fn schedule_task(&mut self, task: StreamTask) -> crate::Result<()> {
        if let Some(ref scheduler) = self.processing_scheduler {
            let mut queue = scheduler.task_queue.lock().unwrap();
            
            // Insert based on priority (higher priority first)
            let insert_pos = queue.iter()
                .position(|existing_task| existing_task.priority > task.priority)
                .unwrap_or(queue.len());
                
            queue.insert(insert_pos, task);
            Ok(())
        } else {
            Err(crate::SynthesisError::new(ErrorKind::AudioDeviceError, 
                "Real-time processing not started. Call start_real_time_processing() first".to_string()))
        }
    }
    
    pub fn optimize_stream_buffer(&mut self, stream_name: &str) -> crate::Result<()> {
        if let Some(stream) = self.streams.get(stream_name) {
            let mut stream_data = stream.write().unwrap();
            
            // Remove old data beyond the GC threshold
            if stream_data.buffer.len() > self.real_time_config.gc_threshold {
                let excess = stream_data.buffer.len() - self.real_time_config.gc_threshold;
                for _ in 0..excess {
                    stream_data.buffer.pop_front();
                }
                stream_data.position = stream_data.position.saturating_sub(excess);
            }
            
            Ok(())
        } else {
            Err(crate::SynthesisError::new(ErrorKind::UnknownModule, 
                format!("Stream '{}' not found", stream_name)))
        }
    }
    
    pub fn get_stream_latency(&self, stream_name: &str) -> Option<Duration> {
        if let Some(stream) = self.streams.get(stream_name) {
            let stream_data = stream.read().unwrap();
            stream_data.last_processed.map(|last| last.elapsed())
        } else {
            None
        }
    }
    
    pub fn create_audio_stream(&mut self, name: String, sample_rate: f32) -> crate::Result<()> {
        self.create_stream(name, DataType::Audio, Some(sample_rate))
    }
    
    pub fn create_control_stream(&mut self, name: String) -> crate::Result<()> {
        self.create_stream(name, DataType::Control, None)
    }
    
    pub fn create_midi_stream(&mut self, name: String) -> crate::Result<()> {
        self.create_stream(name, DataType::MIDI, None)
    }
    
    pub fn shutdown(&mut self) -> crate::Result<()> {
        if let Some(scheduler) = self.processing_scheduler.take() {
            // Signal shutdown to all worker threads
            scheduler.shutdown_flag.store(true, Ordering::Relaxed);
            
            // Wait for all threads to finish
            for handle in scheduler.worker_handles {
                handle.join().map_err(|_| crate::SynthesisError::new(
                    ErrorKind::AudioDeviceError,
                    "Failed to join worker thread".to_string()
                ))?;
            }
        }
        
        // Clear all streams
        self.streams.clear();
        self.connections.clear();
        
        Ok(())
    }
}