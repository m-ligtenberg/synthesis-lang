pub struct Reverb {
    delay_lines: Vec<DelayLine>,
    feedback: f32,
    wet_mix: f32,
}

pub struct DelayLine {
    buffer: Vec<f32>,
    read_pos: usize,
    write_pos: usize,
}

impl DelayLine {
    pub fn new(delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            read_pos: 0,
            write_pos: 0,
        }
    }

    pub fn process(&mut self, input: f32, feedback: f32) -> f32 {
        let delayed = self.buffer[self.read_pos];
        self.buffer[self.write_pos] = input + delayed * feedback;

        self.read_pos = (self.read_pos + 1) % self.buffer.len();
        self.write_pos = (self.write_pos + 1) % self.buffer.len();

        delayed
    }
}

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        let delays = vec![
            (0.03 * sample_rate) as usize,
            (0.05 * sample_rate) as usize,
            (0.07 * sample_rate) as usize,
            (0.11 * sample_rate) as usize,
        ];

        let delay_lines = delays
            .into_iter()
            .map(DelayLine::new)
            .collect();

        Self {
            delay_lines,
            feedback: 0.6,
            wet_mix: 0.3,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let mut output = 0.0;

        for delay_line in &mut self.delay_lines {
            output += delay_line.process(input, self.feedback);
        }

        output /= self.delay_lines.len() as f32;
        input * (1.0 - self.wet_mix) + output * self.wet_mix
    }
}

pub struct Filter {
    filter_type: FilterType,
    cutoff: f32,
    resonance: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    sample_rate: f32,
}

#[derive(Debug, Clone)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
}

impl Filter {
    pub fn new(filter_type: FilterType, cutoff: f32, resonance: f32, sample_rate: f32) -> Self {
        Self {
            filter_type,
            cutoff,
            resonance,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            sample_rate,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let frequency = self.cutoff / (self.sample_rate * 0.5);
        let frequency = frequency.max(0.001).min(0.999);
        
        let c = 1.0 / (2.0 * std::f32::consts::PI * frequency).tan();
        let a1 = 1.0 / (1.0 + self.resonance * c + c * c);
        let a2 = 2.0 * a1;
        let a3 = a1;
        let b1 = 2.0 * (1.0 - c * c) * a1;
        let b2 = (1.0 - self.resonance * c + c * c) * a1;

        let output = match self.filter_type {
            FilterType::LowPass => {
                a1 * input + a2 * self.x1 + a3 * self.x2 - b1 * self.y1 - b2 * self.y2
            }
            FilterType::HighPass => {
                a1 * c * c * (input - 2.0 * self.x1 + self.x2) - b1 * self.y1 - b2 * self.y2
            }
            FilterType::BandPass => {
                a1 * self.resonance * c * (input - self.x2) - b1 * self.y1 - b2 * self.y2
            }
        };

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
    }
}

// Advanced Compressor with professional controls
pub struct Compressor {
    threshold: f32,
    ratio: f32,
    attack_time: f32,
    release_time: f32,
    makeup_gain: f32,
    envelope: f32,
    sample_rate: f32,
    attack_coeff: f32,
    release_coeff: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        let mut compressor = Self {
            threshold: -20.0, // dB
            ratio: 4.0,
            attack_time: 0.003, // 3ms
            release_time: 0.1, // 100ms
            makeup_gain: 0.0,
            envelope: 0.0,
            sample_rate,
            attack_coeff: 0.0,
            release_coeff: 0.0,
        };
        compressor.update_coefficients();
        compressor
    }
    
    fn update_coefficients(&mut self) {
        self.attack_coeff = (-1.0 / (self.attack_time * self.sample_rate)).exp();
        self.release_coeff = (-1.0 / (self.release_time * self.sample_rate)).exp();
    }
    
    pub fn process(&mut self, input: f32) -> f32 {
        let input_level = 20.0 * input.abs().log10().max(-60.0); // Convert to dB
        
        let target_envelope = if input_level > self.threshold {
            self.threshold + (input_level - self.threshold) / self.ratio
        } else {
            input_level
        };
        
        let gain_reduction = target_envelope - input_level;
        
        // Smooth the envelope
        let coeff = if gain_reduction < self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        
        self.envelope = gain_reduction + (self.envelope - gain_reduction) * coeff;
        
        let gain = (self.envelope / 20.0).exp() * (self.makeup_gain / 20.0).exp();
        input * gain
    }
    
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold = threshold_db;
    }
    
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.max(1.0);
    }
    
    pub fn set_attack(&mut self, attack_ms: f32) {
        self.attack_time = attack_ms / 1000.0;
        self.update_coefficients();
    }
    
    pub fn set_release(&mut self, release_ms: f32) {
        self.release_time = release_ms / 1000.0;
        self.update_coefficients();
    }
}

// Multi-tap Delay with stereo width and modulation
pub struct MultiTapDelay {
    buffer: Vec<f32>,
    write_pos: usize,
    taps: Vec<DelayTap>,
    feedback: f32,
    wet_mix: f32,
    sample_rate: f32,
}

#[derive(Clone)]
pub struct DelayTap {
    delay_samples: usize,
    gain: f32,
    pan: f32, // -1.0 to 1.0
}

impl MultiTapDelay {
    pub fn new(max_delay_ms: f32, sample_rate: f32) -> Self {
        let max_samples = (max_delay_ms * sample_rate / 1000.0) as usize;
        
        Self {
            buffer: vec![0.0; max_samples],
            write_pos: 0,
            taps: Vec::new(),
            feedback: 0.3,
            wet_mix: 0.5,
            sample_rate,
        }
    }
    
    pub fn add_tap(&mut self, delay_ms: f32, gain: f32, pan: f32) {
        let delay_samples = (delay_ms * self.sample_rate / 1000.0) as usize;
        if delay_samples < self.buffer.len() {
            self.taps.push(DelayTap {
                delay_samples,
                gain,
                pan: pan.clamp(-1.0, 1.0),
            });
        }
    }
    
    pub fn process_stereo(&mut self, input: f32) -> (f32, f32) {
        let mut left_output = 0.0;
        let mut right_output = 0.0;
        
        // Process each tap
        for tap in &self.taps {
            let read_pos = (self.write_pos + self.buffer.len() - tap.delay_samples) % self.buffer.len();
            let delayed_sample = self.buffer[read_pos] * tap.gain;
            
            // Apply stereo panning
            let left_gain = ((1.0 - tap.pan) * 0.5).sqrt();
            let right_gain = ((1.0 + tap.pan) * 0.5).sqrt();
            
            left_output += delayed_sample * left_gain;
            right_output += delayed_sample * right_gain;
        }
        
        // Write to buffer with feedback
        self.buffer[self.write_pos] = input + (left_output + right_output) * 0.5 * self.feedback;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        
        // Mix dry and wet signals
        let dry_gain = 1.0 - self.wet_mix;
        (
            input * dry_gain + left_output * self.wet_mix,
            input * dry_gain + right_output * self.wet_mix,
        )
    }
}

// Chorus/Flanger with LFO modulation
pub struct Modulation {
    delay_line: Vec<f32>,
    write_pos: usize,
    lfo_phase: f32,
    lfo_frequency: f32,
    depth: f32,
    base_delay: f32,
    feedback: f32,
    wet_mix: f32,
    sample_rate: f32,
    effect_type: ModulationType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModulationType {
    Chorus,
    Flanger,
    Vibrato,
}

impl Modulation {
    pub fn new(effect_type: ModulationType, sample_rate: f32) -> Self {
        let max_delay = match effect_type {
            ModulationType::Chorus => 0.05,  // 50ms max delay
            ModulationType::Flanger => 0.01, // 10ms max delay
            ModulationType::Vibrato => 0.005, // 5ms max delay
        };
        
        let buffer_size = (max_delay * sample_rate) as usize;
        
        Self {
            delay_line: vec![0.0; buffer_size],
            write_pos: 0,
            lfo_phase: 0.0,
            lfo_frequency: 0.5, // Hz
            depth: 0.5,
            base_delay: max_delay * 0.5,
            feedback: if effect_type == ModulationType::Flanger { 0.7 } else { 0.0 },
            wet_mix: 0.5,
            sample_rate,
            effect_type,
        }
    }
    
    pub fn process(&mut self, input: f32) -> f32 {
        // Calculate LFO modulation
        let lfo_value = (2.0 * std::f32::consts::PI * self.lfo_phase).sin();
        let modulated_delay = self.base_delay + lfo_value * self.depth * self.base_delay;
        let delay_samples = (modulated_delay * self.sample_rate).max(1.0);
        
        // Fractional delay interpolation
        let delay_int = delay_samples as usize;
        let delay_frac = delay_samples - delay_int as f32;
        
        let read_pos1 = (self.write_pos + self.delay_line.len() - delay_int) % self.delay_line.len();
        let read_pos2 = (read_pos1 + 1) % self.delay_line.len();
        
        let sample1 = self.delay_line[read_pos1];
        let sample2 = self.delay_line[read_pos2];
        let delayed_sample = sample1 * (1.0 - delay_frac) + sample2 * delay_frac;
        
        // Write input with feedback
        self.delay_line[self.write_pos] = input + delayed_sample * self.feedback;
        self.write_pos = (self.write_pos + 1) % self.delay_line.len();
        
        // Update LFO phase
        self.lfo_phase += self.lfo_frequency / self.sample_rate;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }
        
        // Mix output based on effect type
        match self.effect_type {
            ModulationType::Vibrato => delayed_sample,
            _ => input * (1.0 - self.wet_mix) + delayed_sample * self.wet_mix,
        }
    }
    
    pub fn set_rate(&mut self, frequency_hz: f32) {
        self.lfo_frequency = frequency_hz.clamp(0.01, 20.0);
    }
    
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }
}

// Distortion/Saturation effects
pub struct Distortion {
    drive: f32,
    output_gain: f32,
    distortion_type: DistortionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DistortionType {
    SoftClip,
    HardClip,
    Tube,
    Bitcrush,
}

impl Distortion {
    pub fn new(distortion_type: DistortionType) -> Self {
        Self {
            drive: 2.0,
            output_gain: 0.5,
            distortion_type,
        }
    }
    
    pub fn process(&self, input: f32) -> f32 {
        let driven = input * self.drive;
        
        let processed = match self.distortion_type {
            DistortionType::SoftClip => {
                // Soft clipping using tanh
                driven.tanh()
            }
            DistortionType::HardClip => {
                // Hard clipping
                driven.clamp(-1.0, 1.0)
            }
            DistortionType::Tube => {
                // Tube-style asymmetric saturation
                if driven >= 0.0 {
                    1.0 - (-driven).exp()
                } else {
                    -1.0 + driven.exp()
                }
            }
            DistortionType::Bitcrush => {
                // Bit reduction
                let bits = 8.0;
                let step = 2.0 / (2.0_f32.powf(bits) - 1.0);
                (driven / step).round() * step
            }
        };
        
        processed * self.output_gain
    }
    
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.max(0.1);
    }
    
    pub fn set_output_gain(&mut self, gain: f32) {
        self.output_gain = gain.clamp(0.0, 2.0);
    }
}

// Parametric EQ with multiple bands
pub struct ParametricEQ {
    bands: Vec<EQBand>,
    sample_rate: f32,
}

pub struct EQBand {
    filter: BiquadFilter,
    frequency: f32,
    gain_db: f32,
    q_factor: f32,
    band_type: EQBandType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EQBandType {
    HighPass,
    LowShelf,
    Bell,
    HighShelf,
    LowPass,
}

pub struct BiquadFilter {
    a0: f32, a1: f32, a2: f32,
    b1: f32, b2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl BiquadFilter {
    pub fn new() -> Self {
        Self {
            a0: 1.0, a1: 0.0, a2: 0.0,
            b1: 0.0, b2: 0.0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }
    
    pub fn set_coefficients(&mut self, a0: f32, a1: f32, a2: f32, b1: f32, b2: f32) {
        self.a0 = a0; self.a1 = a1; self.a2 = a2;
        self.b1 = b1; self.b2 = b2;
    }
    
    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.a0 * input + self.a1 * self.x1 + self.a2 * self.x2
                   - self.b1 * self.y1 - self.b2 * self.y2;
        
        self.x2 = self.x1; self.x1 = input;
        self.y2 = self.y1; self.y1 = output;
        
        output
    }
    
    pub fn reset(&mut self) {
        self.x1 = 0.0; self.x2 = 0.0;
        self.y1 = 0.0; self.y2 = 0.0;
    }
}

impl ParametricEQ {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            bands: Vec::new(),
            sample_rate,
        }
    }
    
    pub fn add_band(&mut self, frequency: f32, gain_db: f32, q_factor: f32, band_type: EQBandType) {
        let mut band = EQBand {
            filter: BiquadFilter::new(),
            frequency,
            gain_db,
            q_factor,
            band_type,
        };
        self.calculate_coefficients(&mut band);
        self.bands.push(band);
    }
    
    fn calculate_coefficients(&self, band: &mut EQBand) {
        let omega = 2.0 * std::f32::consts::PI * band.frequency / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * band.q_factor);
        let a = (band.gain_db / 40.0 * std::f32::consts::LN_10).exp();
        
        let (a0, a1, a2, b0, b1, b2) = match band.band_type {
            EQBandType::Bell => {
                let b0 = 1.0 + alpha * a;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0 - alpha * a;
                let a0 = 1.0 + alpha / a;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha / a;
                (a0, a1, a2, b0, b1, b2)
            }
            EQBandType::LowShelf => {
                let s = 1.0;
                let beta = (a / band.q_factor).sqrt();
                let b0 = a * ((a + 1.0) - (a - 1.0) * cos_omega + beta * sin_omega);
                let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega);
                let b2 = a * ((a + 1.0) - (a - 1.0) * cos_omega - beta * sin_omega);
                let a0 = (a + 1.0) + (a - 1.0) * cos_omega + beta * sin_omega;
                let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega);
                let a2 = (a + 1.0) + (a - 1.0) * cos_omega - beta * sin_omega;
                (a0, a1, a2, b0, b1, b2)
            }
            _ => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0), // Default passthrough
        };
        
        band.filter.set_coefficients(b0/a0, b1/a0, b2/a0, a1/a0, a2/a0);
    }
    
    pub fn process(&mut self, input: f32) -> f32 {
        let mut output = input;
        for band in &mut self.bands {
            output = band.filter.process(output);
        }
        output
    }
}

// Effects chain manager
pub struct EffectsChain {
    effects: Vec<Box<dyn AudioEffect>>,
    bypass: bool,
}

pub trait AudioEffect {
    fn process(&mut self, input: f32) -> f32;
    fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        (self.process(left), self.process(right))
    }
    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
}

impl AudioEffect for Reverb {
    fn process(&mut self, input: f32) -> f32 {
        self.process(input)
    }
}

impl AudioEffect for Filter {
    fn process(&mut self, input: f32) -> f32 {
        self.process(input)
    }
}

impl AudioEffect for Compressor {
    fn process(&mut self, input: f32) -> f32 {
        self.process(input)
    }
}

impl AudioEffect for Distortion {
    fn process(&mut self, input: f32) -> f32 {
        self.process(input)
    }
}

impl AudioEffect for Modulation {
    fn process(&mut self, input: f32) -> f32 {
        self.process(input)
    }
}

impl EffectsChain {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            bypass: false,
        }
    }
    
    pub fn add_effect(&mut self, effect: Box<dyn AudioEffect>) {
        self.effects.push(effect);
    }
    
    pub fn process(&mut self, input: f32) -> f32 {
        if self.bypass {
            return input;
        }
        
        let mut output = input;
        for effect in &mut self.effects {
            output = effect.process(output);
        }
        output
    }
    
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if self.bypass {
            return (left, right);
        }
        
        let mut left_out = left;
        let mut right_out = right;
        
        for effect in &mut self.effects {
            let (l, r) = effect.process_stereo(left_out, right_out);
            left_out = l;
            right_out = r;
        }
        
        (left_out, right_out)
    }
    
    pub fn set_bypass(&mut self, bypass: bool) {
        self.bypass = bypass;
    }
    
    pub fn clear(&mut self) {
        self.effects.clear();
    }
}

// Preset configurations for common effect combinations
pub struct EffectPresets;

impl EffectPresets {
    pub fn create_vocal_chain(sample_rate: f32) -> EffectsChain {
        let mut chain = EffectsChain::new();
        
        // High-pass filter to remove low-end rumble
        let mut hp_filter = Filter::new(FilterType::HighPass, 80.0, 0.7, sample_rate);
        chain.add_effect(Box::new(hp_filter));
        
        // Compressor for dynamics control
        let mut compressor = Compressor::new(sample_rate);
        compressor.set_threshold(-18.0);
        compressor.set_ratio(3.0);
        compressor.set_attack(5.0);
        compressor.set_release(50.0);
        chain.add_effect(Box::new(compressor));
        
        // EQ for presence
        let mut eq = ParametricEQ::new(sample_rate);
        eq.add_band(3000.0, 2.0, 2.0, EQBandType::Bell);
        chain.add_effect(Box::new(eq));
        
        // Reverb for space
        let reverb = Reverb::new(sample_rate);
        chain.add_effect(Box::new(reverb));
        
        chain
    }
    
    pub fn create_guitar_chain(sample_rate: f32) -> EffectsChain {
        let mut chain = EffectsChain::new();
        
        // Tube-style distortion
        let mut distortion = Distortion::new(DistortionType::Tube);
        distortion.set_drive(1.5);
        chain.add_effect(Box::new(distortion));
        
        // Chorus for width
        let mut chorus = Modulation::new(ModulationType::Chorus, sample_rate);
        chorus.set_rate(0.4);
        chorus.set_depth(0.3);
        chain.add_effect(Box::new(chorus));
        
        // Delay for ambience
        let mut delay = MultiTapDelay::new(500.0, sample_rate);
        delay.add_tap(250.0, 0.4, -0.3);
        delay.add_tap(375.0, 0.3, 0.4);
        
        chain
    }
    
    pub fn create_drum_bus(sample_rate: f32) -> EffectsChain {
        let mut chain = EffectsChain::new();
        
        // Transient compressor
        let mut compressor = Compressor::new(sample_rate);
        compressor.set_threshold(-15.0);
        compressor.set_ratio(4.0);
        compressor.set_attack(1.0);
        compressor.set_release(30.0);
        chain.add_effect(Box::new(compressor));
        
        // EQ for punch
        let mut eq = ParametricEQ::new(sample_rate);
        eq.add_band(100.0, 1.5, 1.0, EQBandType::LowShelf);
        eq.add_band(5000.0, 1.0, 2.0, EQBandType::Bell);
        chain.add_effect(Box::new(eq));
        
        chain
    }
}