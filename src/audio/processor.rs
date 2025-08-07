use crate::audio::{effects::*, analysis::*};
use crate::runtime::realtime_buffer::{RealtimeCircularBuffer, BufferError};

pub struct AudioProcessor {
    _sample_rate: f32,
    buffer_size: usize,
    input_buffer: RealtimeCircularBuffer,
    output_buffer: RealtimeCircularBuffer,
    effects_chain: Vec<Box<dyn AudioEffect + Send>>,
    fft_analyzer: FFTAnalyzer,
    beat_detector: BeatDetector,
    pitch_detector: PitchDetector,
}

pub trait AudioEffect: Send {
    fn process(&mut self, input: f32) -> f32;
    fn set_parameter(&mut self, name: &str, value: f32);
}

impl AudioProcessor {
    pub fn new(sample_rate: f32, buffer_size: usize) -> Self {
        let buffer_capacity = (buffer_size * 4).next_power_of_two();
        
        Self {
            _sample_rate: sample_rate,
            buffer_size,
            input_buffer: RealtimeCircularBuffer::new(buffer_capacity).unwrap(),
            output_buffer: RealtimeCircularBuffer::new(buffer_capacity).unwrap(),
            effects_chain: Vec::new(),
            fft_analyzer: FFTAnalyzer::new(1024),
            beat_detector: BeatDetector::new(44),
            pitch_detector: PitchDetector::new(sample_rate),
        }
    }
    
    pub fn add_effect(&mut self, effect: Box<dyn AudioEffect + Send>) {
        self.effects_chain.push(effect);
    }
    
    pub fn process_buffer(&mut self, input: &[f32]) -> Vec<f32> {
        // Pre-allocate output vector to exact size (no dynamic growth)
        let mut output = Vec::with_capacity(input.len());
        output.resize(input.len(), 0.0);
        
        // Write input to circular buffer (real-time safe)
        for &sample in input {
            // Silently drop samples if buffer full (prevents blocking)
            let _ = self.input_buffer.write_single(sample);
        }
        
        // Process available samples (bounded by input length)
        for i in 0..input.len() {
            match self.input_buffer.read_single() {
                Ok(mut sample) => {
                    // Apply effects chain
                    for effect in &mut self.effects_chain {
                        sample = effect.process(sample);
                    }
                    
                    output[i] = sample;
                    // Store processed sample (silently drop if buffer full)
                    let _ = self.output_buffer.write_single(sample);
                }
                Err(BufferError::BufferEmpty) => {
                    // Output silence if no input available
                    output[i] = 0.0;
                }
                Err(_) => {
                    output[i] = 0.0;
                }
            }
        }
        
        output
    }
    
    pub fn analyze_audio(&mut self, samples: &[f32]) -> AudioAnalysis {
        let fft_data = self.fft_analyzer.analyze(samples, 16);
        let beat = self.beat_detector.detect_beat(samples);
        let pitch = self.pitch_detector.detect_pitch(samples);
        let onset = self.detect_onset(samples);
        
        AudioAnalysis {
            fft_bands: fft_data,
            beat_detected: beat,
            pitch_hz: pitch,
            onset_detected: onset,
            rms_level: self.calculate_rms(samples),
            peak_level: self.calculate_peak(samples),
        }
    }
    
    fn detect_onset(&self, samples: &[f32]) -> bool {
        // Simple onset detection using energy difference
        if samples.len() < 128 {
            return false;
        }
        
        let mid = samples.len() / 2;
        let energy_1 = samples[..mid].iter().map(|x| x * x).sum::<f32>() / mid as f32;
        let energy_2 = samples[mid..].iter().map(|x| x * x).sum::<f32>() / (samples.len() - mid) as f32;
        
        energy_2 > energy_1 * 1.5 && energy_2 > 0.01
    }
    
    fn calculate_rms(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_squares = samples.iter().map(|x| x * x).sum::<f32>();
        (sum_squares / samples.len() as f32).sqrt()
    }
    
    fn calculate_peak(&self, samples: &[f32]) -> f32 {
        samples.iter().map(|x| x.abs()).fold(0.0, f32::max)
    }
    
    pub fn get_recent_samples(&self, count: usize) -> Vec<f32> {
        let mut result = Vec::with_capacity(count);
        
        // Read most recent samples (may be fewer than requested)
        for _ in 0..count {
            match self.output_buffer.read_single() {
                Ok(sample) => result.push(sample),
                Err(BufferError::BufferEmpty) => result.push(0.0),
                Err(_) => result.push(0.0),
            }
        }
        
        result
    }
    
    pub fn clear_buffers(&mut self) {
        // Circular buffers clear by resetting read/write pointers
        self.input_buffer.clear();
        self.output_buffer.clear();
    }
}

#[derive(Debug, Clone)]
pub struct AudioAnalysis {
    pub fft_bands: Vec<f32>,
    pub beat_detected: bool,
    pub pitch_hz: Option<f32>,
    pub onset_detected: bool,
    pub rms_level: f32,
    pub peak_level: f32,
}

pub struct PitchDetector {
    _sample_rate: f32,
    _autocorr_buffer: Vec<f32>,
    min_freq: f32,
    max_freq: f32,
}

impl PitchDetector {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            _sample_rate: sample_rate,
            _autocorr_buffer: vec![0.0; 2048],
            min_freq: 80.0,   // Minimum frequency to detect
            max_freq: 2000.0, // Maximum frequency to detect
        }
    }
    
    pub fn detect_pitch(&mut self, samples: &[f32]) -> Option<f32> {
        if samples.len() < 512 {
            return None;
        }
        
        // Use autocorrelation for pitch detection
        let min_period = (self._sample_rate / self.max_freq) as usize;
        let max_period = (self._sample_rate / self.min_freq) as usize;
        
        if max_period >= samples.len() {
            return None;
        }
        
        let mut best_correlation = 0.0;
        let mut best_period = 0;
        
        for period in min_period..max_period.min(samples.len() - 1) {
            let mut correlation = 0.0;
            let mut norm1 = 0.0;
            let mut norm2 = 0.0;
            
            for i in 0..(samples.len() - period) {
                correlation += samples[i] * samples[i + period];
                norm1 += samples[i] * samples[i];
                norm2 += samples[i + period] * samples[i + period];
            }
            
            if norm1 > 0.0 && norm2 > 0.0 {
                correlation /= (norm1 * norm2).sqrt();
                
                if correlation > best_correlation && correlation > 0.3 {
                    best_correlation = correlation;
                    best_period = period;
                }
            }
        }
        
        if best_correlation > 0.5 && best_period > 0 {
            Some(self._sample_rate / best_period as f32)
        } else {
            None
        }
    }
}

// Real-time safe reverb implementation
pub struct RealtimeReverb {
    delay_lines: Vec<DelayLine>,
    feedback: f32,
    wet_mix: f32,
    diffusion: f32,
}

impl RealtimeReverb {
    pub fn new(sample_rate: f32, room_size: f32) -> Self {
        let base_delay = (room_size * sample_rate * 0.001) as usize;
        
        let delays = vec![
            base_delay + 347,
            base_delay + 563,
            base_delay + 773,
            base_delay + 997,
            base_delay + 1117,
            base_delay + 1237,
        ];
        
        let delay_lines = delays.into_iter().map(DelayLine::new).collect();
        
        Self {
            delay_lines,
            feedback: 0.7,
            wet_mix: 0.3,
            diffusion: 0.625,
        }
    }
}

impl AudioEffect for RealtimeReverb {
    fn process(&mut self, input: f32) -> f32 {
        let mut output = 0.0;
        let diffused_input = input * self.diffusion;
        
        for delay_line in &mut self.delay_lines {
            let delayed = delay_line.process(diffused_input, self.feedback);
            output += delayed;
        }
        
        output /= self.delay_lines.len() as f32;
        input * (1.0 - self.wet_mix) + output * self.wet_mix
    }
    
    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "feedback" => self.feedback = value.clamp(0.0, 0.95),
            "wet_mix" => self.wet_mix = value.clamp(0.0, 1.0),
            "diffusion" => self.diffusion = value.clamp(0.0, 1.0),
            _ => {}
        }
    }
}

// Real-time safe distortion
pub struct Distortion {
    drive: f32,
    output_gain: f32,
    tone: f32,
    filter: Filter,
}

impl Distortion {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            drive: 2.0,
            output_gain: 0.5,
            tone: 0.5,
            filter: Filter::new(
                crate::audio::effects::FilterType::LowPass,
                8000.0,
                0.7,
                sample_rate,
            ),
        }
    }
    
    fn waveshape(&self, input: f32) -> f32 {
        let driven = input * self.drive;
        
        // Soft clipping with smooth transitions
        if driven > 1.0 {
            1.0 - (driven - 1.0).exp() * 0.1
        } else if driven < -1.0 {
            -1.0 + (driven + 1.0).exp() * 0.1
        } else {
            driven - (driven.powi(3) / 3.0)
        }
    }
}

impl AudioEffect for Distortion {
    fn process(&mut self, input: f32) -> f32 {
        let distorted = self.waveshape(input);
        let filtered = self.filter.process(distorted);
        filtered * self.output_gain
    }
    
    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "drive" => self.drive = value.clamp(0.1, 10.0),
            "output_gain" => self.output_gain = value.clamp(0.0, 1.0),
            "tone" => {
                self.tone = value.clamp(0.0, 1.0);
                let cutoff = 1000.0 + value * 7000.0;
                self.filter.set_cutoff(cutoff);
            }
            _ => {}
        }
    }
}