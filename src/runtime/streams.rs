use crate::runtime::types::{DataType, Value};
use crate::runtime::realtime_buffer::{SharedRealtimeBuffer, BufferError};
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

// Enhanced StreamData with real-time buffer for performance-critical streams
#[derive(Debug)]
pub struct RealtimeStreamData {
    pub name: String,
    pub data_type: DataType,
    pub sample_rate: Option<f32>,
    pub realtime_buffer: SharedRealtimeBuffer,
    pub fallback_buffer: VecDeque<f32>, // For compatibility with existing code
    pub position: usize,
    pub is_active: bool,
    pub timestamp: Instant,
    pub metadata: HashMap<String, Value>,
    pub processing_chain: Vec<StreamProcessor>,
    pub latency_target: Duration,
    pub last_processed: Option<Instant>,
    pub processing_time_us: u64,
    pub use_realtime_buffer: bool, // Toggle between buffer types
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

// Core stream primitive types
#[derive(Debug, Clone)]
pub enum StreamPrimitive {
    Input {
        source_type: InputSourceType,
        callback: Option<String>, // Function name for external input handling
    },
    Output {
        destination_type: OutputDestinationType,
        format: OutputFormat,
    },
    Transform {
        transform_type: TransformType,
        parameters: HashMap<String, Value>,
    },
    Buffer {
        size: usize,
        policy: BufferPolicy,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputSourceType {
    AudioDevice,
    MidiController,
    OSC,
    File { path: String },
    Generator { waveform: WaveformType },
    ExternalFunction { name: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputDestinationType {
    AudioDevice,
    MidiDevice,
    OSC { host: String, port: u16 },
    File { path: String },
    Graphics,
    ExternalFunction { name: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Float32,
    Int16,
    Midi,
    OSCMessage,
    Graphics { format: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransformType {
    Gain { amount: f32 },
    Filter { cutoff: f32, resonance: f32, filter_type: FilterType },
    Delay { time: f32, feedback: f32 },
    Reverb { room_size: f32, damping: f32, wet_mix: f32 },
    Distortion { drive: f32, tone: f32 },
    Compressor { threshold: f32, ratio: f32, attack: f32, release: f32 },
    EQ { bands: Vec<EQBand> },
    Envelope { attack: f32, decay: f32, sustain: f32, release: f32 },
    Custom { function: String }, // Reference to user-defined transform function
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EQBand {
    pub frequency: f32,
    pub gain: f32,
    pub q_factor: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaveformType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BufferPolicy {
    Circular,    // Overwrite oldest data when full
    Blocking,    // Block writes when full
    Dropping,    // Drop new data when full
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
    
    // Real-time stream management methods
    
    pub fn create_realtime_stream(&mut self, name: String, data_type: DataType, sample_rate: Option<f32>, buffer_size: Option<usize>) -> crate::Result<()> {
        let latency_target = Duration::from_millis(self.real_time_config.target_latency_ms as u64);
        let buffer_size = buffer_size.unwrap_or(self.real_time_config.buffer_size);
        
        // Ensure buffer size is power of 2 for optimal performance
        let buffer_size = buffer_size.next_power_of_two();
        
        let _realtime_buffer = SharedRealtimeBuffer::new(buffer_size)
            .map_err(|_| crate::SynthesisError::new(ErrorKind::AudioDeviceError, 
                "Failed to create real-time buffer".to_string()))?;
        
        let regular_stream_data = StreamData {
            name: name.clone(),
            data_type,
            sample_rate,
            buffer: VecDeque::with_capacity(buffer_size),
            max_buffer_size: buffer_size,
            position: 0,
            is_active: false,
            timestamp: Instant::now(),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("is_realtime".to_string(), Value::Boolean(true));
                metadata.insert("buffer_type".to_string(), Value::String("realtime_circular".to_string()));
                metadata
            },
            processing_chain: Vec::new(),
            latency_target,
            last_processed: None,
            processing_time_us: 0,
        };
        
        self.streams.insert(name, Arc::new(RwLock::new(regular_stream_data)));
        Ok(())
    }
    
    pub fn write_to_realtime_stream(&self, name: &str, data: Vec<f32>) -> crate::Result<()> {
        if let Some(stream) = self.streams.get(name) {
            let stream_data = stream.read().unwrap();
            
            // Check if this is a real-time stream
            if let Some(Value::Boolean(true)) = stream_data.metadata.get("is_realtime") {
                drop(stream_data);
                
                // Use regular buffer with real-time characteristics
                let mut stream_data = stream.write().unwrap();
                
                // Implement circular buffer behavior for real-time performance
                let available_space = stream_data.max_buffer_size.saturating_sub(stream_data.buffer.len());
                
                if data.len() > available_space {
                    // Remove old data to make space (circular buffer behavior)
                    let excess = data.len() - available_space;
                    for _ in 0..excess {
                        stream_data.buffer.pop_front();
                    }
                }
                
                // Add new data
                for &sample in &data {
                    stream_data.buffer.push_back(sample);
                }
                
                stream_data.is_active = true;
                stream_data.timestamp = Instant::now();
                Ok(())
            } else {
                // Fall back to regular stream write
                drop(stream_data);
                self.write_to_stream(name, data)
            }
        } else {
            Err(crate::SynthesisError::new(ErrorKind::UnknownModule, format!("Stream '{}' not found", name)))
        }
    }
    
    pub fn read_from_realtime_stream(&self, name: &str, count: usize) -> crate::Result<Vec<f32>> {
        if let Some(stream) = self.streams.get(name) {
            let stream_data = stream.read().unwrap();
            
            // Check if this is a real-time stream
            if let Some(Value::Boolean(true)) = stream_data.metadata.get("is_realtime") {
                drop(stream_data);
                
                let mut stream_data = stream.write().unwrap();
                
                // Implement real-time read behavior
                let available = stream_data.buffer.len().min(count);
                let mut data = Vec::with_capacity(count);
                
                // Read available data
                for _ in 0..available {
                    if let Some(sample) = stream_data.buffer.pop_front() {
                        data.push(sample);
                    }
                }
                
                // Fill remaining with zeros if underrun (real-time safe behavior)
                while data.len() < count {
                    data.push(0.0);
                    
                    // Update underrun metrics
                    let mut metrics = self.performance_metrics.lock().unwrap();
                    metrics.buffer_underruns += 1;
                }
                
                stream_data.position += count;
                Ok(data)
            } else {
                // Fall back to regular stream read
                drop(stream_data);
                self.read_from_stream(name, count)
            }
        } else {
            Err(crate::SynthesisError::new(ErrorKind::UnknownModule, format!("Stream '{}' not found", name)))
        }
    }
    
    // Core stream primitive implementations
    
    pub fn create_input_stream(&mut self, name: String, source_type: InputSourceType) -> crate::Result<()> {
        let primitive = StreamPrimitive::Input { 
            source_type: source_type.clone(), 
            callback: None 
        };
        
        let data_type = match source_type {
            InputSourceType::AudioDevice | InputSourceType::Generator { .. } => DataType::Audio,
            InputSourceType::MidiController => DataType::MIDI,
            InputSourceType::OSC => DataType::Control,
            InputSourceType::File { .. } => DataType::Generic,
            InputSourceType::ExternalFunction { .. } => DataType::Generic,
        };
        
        self.create_stream(name.clone(), data_type, None)?;
        
        // Add the primitive to the stream metadata
        if let Some(stream) = self.streams.get(&name) {
            let mut stream_data = stream.write().unwrap();
            stream_data.metadata.insert("primitive".to_string(), 
                Value::String("input".to_string()));
            stream_data.metadata.insert("source_type".to_string(), 
                Value::String(format!("{:?}", source_type)));
        }
        
        Ok(())
    }
    
    pub fn create_output_stream(&mut self, name: String, destination_type: OutputDestinationType, format: OutputFormat) -> crate::Result<()> {
        let _primitive = StreamPrimitive::Output { 
            destination_type: destination_type.clone(), 
            format: format.clone()
        };
        
        let data_type = match destination_type {
            OutputDestinationType::AudioDevice => DataType::Audio,
            OutputDestinationType::MidiDevice => DataType::MIDI,
            OutputDestinationType::OSC { .. } => DataType::Control,
            OutputDestinationType::Graphics => DataType::Visual,
            OutputDestinationType::File { .. } | OutputDestinationType::ExternalFunction { .. } => DataType::Generic,
        };
        
        self.create_stream(name.clone(), data_type, None)?;
        
        // Add the primitive to the stream metadata
        if let Some(stream) = self.streams.get(&name) {
            let mut stream_data = stream.write().unwrap();
            stream_data.metadata.insert("primitive".to_string(), 
                Value::String("output".to_string()));
            stream_data.metadata.insert("destination_type".to_string(), 
                Value::String(format!("{:?}", destination_type)));
            stream_data.metadata.insert("format".to_string(), 
                Value::String(format!("{:?}", format)));
        }
        
        Ok(())
    }
    
    pub fn create_transform_stream(&mut self, name: String, transform_type: TransformType) -> crate::Result<()> {
        let mut parameters = HashMap::new();
        
        // Extract parameters from transform type
        match &transform_type {
            TransformType::Gain { amount } => {
                parameters.insert("amount".to_string(), Value::Float(*amount as f64));
            }
            TransformType::Filter { cutoff, resonance, filter_type } => {
                parameters.insert("cutoff".to_string(), Value::Float(*cutoff as f64));
                parameters.insert("resonance".to_string(), Value::Float(*resonance as f64));
                parameters.insert("filter_type".to_string(), Value::String(format!("{:?}", filter_type)));
            }
            TransformType::Delay { time, feedback } => {
                parameters.insert("time".to_string(), Value::Float(*time as f64));
                parameters.insert("feedback".to_string(), Value::Float(*feedback as f64));
            }
            TransformType::Reverb { room_size, damping, wet_mix } => {
                parameters.insert("room_size".to_string(), Value::Float(*room_size as f64));
                parameters.insert("damping".to_string(), Value::Float(*damping as f64));
                parameters.insert("wet_mix".to_string(), Value::Float(*wet_mix as f64));
            }
            TransformType::Compressor { threshold, ratio, attack, release } => {
                parameters.insert("threshold".to_string(), Value::Float(*threshold as f64));
                parameters.insert("ratio".to_string(), Value::Float(*ratio as f64));
                parameters.insert("attack".to_string(), Value::Float(*attack as f64));
                parameters.insert("release".to_string(), Value::Float(*release as f64));
            }
            TransformType::Custom { function } => {
                parameters.insert("function".to_string(), Value::String(function.clone()));
            }
            _ => {} // Handle other transform types as needed
        }
        
        let _primitive = StreamPrimitive::Transform { 
            transform_type: transform_type.clone(), 
            parameters: parameters.clone()
        };
        
        self.create_stream(name.clone(), DataType::Generic, None)?;
        
        // Add the primitive and parameters to stream metadata
        if let Some(stream) = self.streams.get(&name) {
            let mut stream_data = stream.write().unwrap();
            stream_data.metadata.insert("primitive".to_string(), 
                Value::String("transform".to_string()));
            stream_data.metadata.insert("transform_type".to_string(), 
                Value::String(format!("{:?}", transform_type)));
            
            // Add all parameters to metadata
            for (key, value) in parameters {
                stream_data.metadata.insert(format!("param_{}", key), value);
            }
        }
        
        Ok(())
    }
    
    pub fn create_buffer_stream(&mut self, name: String, size: usize, policy: BufferPolicy) -> crate::Result<()> {
        let _primitive = StreamPrimitive::Buffer { 
            size, 
            policy: policy.clone()
        };
        
        self.create_stream(name.clone(), DataType::Generic, None)?;
        
        // Configure the buffer according to the policy
        if let Some(stream) = self.streams.get(&name) {
            let mut stream_data = stream.write().unwrap();
            stream_data.max_buffer_size = size;
            stream_data.metadata.insert("primitive".to_string(), 
                Value::String("buffer".to_string()));
            stream_data.metadata.insert("policy".to_string(), 
                Value::String(format!("{:?}", policy)));
            stream_data.metadata.insert("configured_size".to_string(), 
                Value::Integer(size as i64));
        }
        
        Ok(())
    }
    
    // Advanced stream processing methods utilizing the primitives
    
    pub fn process_input_stream(&mut self, stream_name: &str) -> crate::Result<Vec<f32>> {
        if let Some(stream) = self.streams.get(stream_name) {
            let stream_data = stream.read().unwrap();
            
            if let Some(Value::String(primitive_type)) = stream_data.metadata.get("primitive") {
                if primitive_type == "input" {
                    if let Some(Value::String(source_type)) = stream_data.metadata.get("source_type") {
                        return self.generate_input_data(source_type);
                    }
                }
            }
        }
        
        Err(crate::SynthesisError::new(ErrorKind::UnknownModule, 
            format!("Stream '{}' is not an input stream or not found", stream_name)))
    }
    
    fn generate_input_data(&self, source_type: &str) -> crate::Result<Vec<f32>> {
        match source_type {
            source if source.contains("AudioDevice") => {
                // Simulate audio input
                Ok(vec![0.1, 0.2, -0.1, -0.2, 0.0]) // Placeholder
            }
            source if source.contains("Generator") => {
                let sample_rate = 44100.0;
                let frequency = 440.0; // A4
                let samples = 128;
                let mut data = Vec::with_capacity(samples);
                
                for i in 0..samples {
                    let t = i as f32 / sample_rate;
                    let sample = (2.0 * std::f32::consts::PI * frequency * t).sin();
                    data.push(sample);
                }
                Ok(data)
            }
            source if source.contains("MidiController") => {
                // Simulate MIDI control values
                Ok(vec![64.0, 65.0, 63.0, 66.0]) // MIDI values (0-127)
            }
            source if source.contains("OSC") => {
                // Simulate OSC control data
                Ok(vec![0.5, 0.6, 0.4, 0.7])
            }
            _ => Ok(vec![0.0; 128]) // Default silence
        }
    }
    
    pub fn apply_transform_stream(&mut self, input_stream: &str, transform_stream: &str, output_stream: &str) -> crate::Result<()> {
        // Get input data
        let input_data = self.read_from_stream(input_stream, 128)?;
        
        // Get transform parameters
        let transform_data = if let Some(stream) = self.streams.get(transform_stream) {
            stream.read().unwrap()
        } else {
            return Err(crate::SynthesisError::new(ErrorKind::UnknownModule, 
                format!("Transform stream '{}' not found", transform_stream)));
        };
        
        // Apply transformation based on metadata
        let processed_data = if let Some(Value::String(transform_type)) = transform_data.metadata.get("transform_type") {
            match transform_type.as_str() {
                transform_str if transform_str.contains("Gain") => {
                    if let Some(Value::Float(amount)) = transform_data.metadata.get("param_amount") {
                        input_data.iter().map(|&x| x * (*amount as f32)).collect()
                    } else {
                        input_data
                    }
                }
                transform_str if transform_str.contains("Filter") => {
                    self.apply_filter_transform(&input_data, &transform_data.metadata)?
                }
                transform_str if transform_str.contains("Delay") => {
                    self.apply_delay_transform(&input_data, &transform_data.metadata)?
                }
                transform_str if transform_str.contains("Reverb") => {
                    self.apply_reverb_transform(&input_data, &transform_data.metadata)?
                }
                _ => input_data // Pass-through for unknown transforms
            }
        } else {
            input_data // No transform metadata found
        };
        
        // Write to output stream
        self.write_to_stream(output_stream, processed_data)?;
        
        Ok(())
    }
    
    fn apply_filter_transform(&self, data: &[f32], metadata: &HashMap<String, Value>) -> crate::Result<Vec<f32>> {
        let cutoff = if let Some(Value::Float(c)) = metadata.get("param_cutoff") {
            *c as f32
        } else {
            0.5 // Default cutoff
        };
        
        let mut filtered = Vec::with_capacity(data.len());
        let mut prev = 0.0;
        let alpha = cutoff.min(1.0).max(0.0);
        
        for &sample in data {
            let filtered_sample = prev + alpha * (sample - prev);
            filtered.push(filtered_sample);
            prev = filtered_sample;
        }
        
        Ok(filtered)
    }
    
    fn apply_delay_transform(&self, data: &[f32], metadata: &HashMap<String, Value>) -> crate::Result<Vec<f32>> {
        let delay_time = if let Some(Value::Float(t)) = metadata.get("param_time") {
            *t as f32
        } else {
            0.1 // Default 100ms
        };
        
        let feedback = if let Some(Value::Float(f)) = metadata.get("param_feedback") {
            *f as f32
        } else {
            0.3 // Default feedback
        };
        
        let delay_samples = (delay_time * 44100.0) as usize;
        let mut delayed = vec![0.0; delay_samples];
        delayed.extend_from_slice(data);
        
        // Apply feedback
        if feedback > 0.0 {
            for i in delay_samples..delayed.len() {
                if i >= delay_samples {
                    delayed[i] += delayed[i - delay_samples] * feedback;
                }
            }
        }
        
        Ok(delayed)
    }
    
    fn apply_reverb_transform(&self, data: &[f32], metadata: &HashMap<String, Value>) -> crate::Result<Vec<f32>> {
        let room_size = if let Some(Value::Float(r)) = metadata.get("param_room_size") {
            *r as f32
        } else {
            0.5 // Default room size
        };
        
        let wet_mix = if let Some(Value::Float(w)) = metadata.get("param_wet_mix") {
            *w as f32
        } else {
            0.3 // Default wet mix
        };
        
        // Simple reverb simulation using multiple delays
        let delay1 = (room_size * 0.1 * 44100.0) as usize;
        let delay2 = (room_size * 0.15 * 44100.0) as usize;
        let delay3 = (room_size * 0.22 * 44100.0) as usize;
        
        let max_delay = delay3;
        let mut reverb_buffer = vec![0.0; data.len() + max_delay];
        reverb_buffer[..data.len()].copy_from_slice(data);
        
        // Add delayed reflections
        for i in data.len()..reverb_buffer.len() {
            let mut reflection = 0.0;
            
            if i >= delay1 {
                reflection += reverb_buffer[i - delay1] * 0.3;
            }
            if i >= delay2 {
                reflection += reverb_buffer[i - delay2] * 0.2;
            }
            if i >= delay3 {
                reflection += reverb_buffer[i - delay3] * 0.15;
            }
            
            reverb_buffer[i] += reflection;
        }
        
        // Mix dry and wet signals
        let mut output = Vec::with_capacity(data.len());
        for i in 0..data.len() {
            let dry = data[i];
            let wet = reverb_buffer[i];
            output.push(dry * (1.0 - wet_mix) + wet * wet_mix);
        }
        
        Ok(output)
    }
    
    pub fn process_output_stream(&mut self, stream_name: &str) -> crate::Result<()> {
        if let Some(stream) = self.streams.get(stream_name) {
            let stream_data = stream.read().unwrap();
            
            if let Some(Value::String(primitive_type)) = stream_data.metadata.get("primitive") {
                if primitive_type == "output" {
                    if let Some(Value::String(dest_type)) = stream_data.metadata.get("destination_type") {
                        let data = stream_data.buffer.iter().cloned().collect::<Vec<f32>>();
                        return self.output_data(dest_type, &data);
                    }
                }
            }
        }
        
        Err(crate::SynthesisError::new(ErrorKind::UnknownModule, 
            format!("Stream '{}' is not an output stream or not found", stream_name)))
    }
    
    fn output_data(&self, destination_type: &str, data: &[f32]) -> crate::Result<()> {
        match destination_type {
            dest if dest.contains("AudioDevice") => {
                // Simulate audio output
                eprintln!("Audio output: {} samples", data.len());
                Ok(())
            }
            dest if dest.contains("Graphics") => {
                // Simulate graphics output
                eprintln!("Graphics output: {} data points", data.len());
                Ok(())
            }
            dest if dest.contains("MidiDevice") => {
                // Simulate MIDI output
                eprintln!("MIDI output: {} values", data.len());
                Ok(())
            }
            dest if dest.contains("OSC") => {
                // Simulate OSC output
                eprintln!("OSC output: {} messages", data.len());
                Ok(())
            }
            _ => {
                eprintln!("Unknown output destination: {}", destination_type);
                Ok(())
            }
        }
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