use crate::runtime::Value;
use std::time::{SystemTime, UNIX_EPOCH, Instant, Duration};
use std::collections::HashMap;

pub fn now(_args: &[Value]) -> crate::Result<Value> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs_f64();
    Ok(Value::Float(timestamp))
}

pub fn delta_time(_args: &[Value]) -> crate::Result<Value> {
    // This would typically be calculated by the main loop
    Ok(Value::Float(0.016667)) // 60 FPS default
}

pub fn fps(_args: &[Value]) -> crate::Result<Value> {
    Ok(Value::Float(60.0))
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub start_time: Instant,
    pub current_time: f64,
    pub speed: f64,
    pub is_playing: bool,
    pub is_looping: bool,
    pub loop_start: f64,
    pub loop_end: f64,
    pub markers: Vec<TimelineMarker>,
    pub events: Vec<TimelineEvent>,
}

#[derive(Debug, Clone)]
pub struct TimelineMarker {
    pub name: String,
    pub time: f64,
    pub color: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct TimelineEvent {
    pub name: String,
    pub start_time: f64,
    pub duration: Option<f64>,
    pub data: HashMap<String, Value>,
    pub event_type: EventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    Audio,
    Visual,
    MIDI,
    Control,
    Custom(String),
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            current_time: 0.0,
            speed: 1.0,
            is_playing: false,
            is_looping: false,
            loop_start: 0.0,
            loop_end: 60.0, // Default 60 second loop
            markers: Vec::new(),
            events: Vec::new(),
        }
    }
    
    pub fn play(&mut self) {
        self.is_playing = true;
        self.start_time = Instant::now();
    }
    
    pub fn pause(&mut self) {
        self.is_playing = false;
    }
    
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_time = 0.0;
        self.start_time = Instant::now();
    }
    
    pub fn seek(&mut self, time: f64) {
        self.current_time = time;
        self.start_time = Instant::now();
    }
    
    pub fn update(&mut self) {
        if !self.is_playing {
            return;
        }
        
        let elapsed = self.start_time.elapsed().as_secs_f64() * self.speed;
        self.current_time = elapsed;
        
        // Handle looping
        if self.is_looping && self.current_time >= self.loop_end {
            let overshoot = self.current_time - self.loop_end;
            self.current_time = self.loop_start + overshoot;
            self.start_time = Instant::now() - Duration::from_secs_f64(overshoot / self.speed);
        }
    }
    
    pub fn add_marker(&mut self, name: String, time: f64, color: [f32; 3]) {
        self.markers.push(TimelineMarker { name, time, color });
        self.markers.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }
    
    pub fn add_event(&mut self, event: TimelineEvent) {
        self.events.push(event);
        self.events.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
    }
    
    pub fn get_active_events(&self) -> Vec<&TimelineEvent> {
        self.events
            .iter()
            .filter(|event| {
                event.start_time <= self.current_time && 
                (event.duration.is_none() || 
                 event.start_time + event.duration.unwrap() >= self.current_time)
            })
            .collect()
    }
    
    pub fn get_events_in_range(&self, start: f64, end: f64) -> Vec<&TimelineEvent> {
        self.events
            .iter()
            .filter(|event| {
                event.start_time < end && 
                (event.duration.is_none() || event.start_time + event.duration.unwrap() > start)
            })
            .collect()
    }
    
    pub fn set_loop(&mut self, start: f64, end: f64) {
        self.loop_start = start;
        self.loop_end = end;
        self.is_looping = true;
    }
    
    pub fn clear_loop(&mut self) {
        self.is_looping = false;
    }
}

#[derive(Debug, Clone)]
pub struct Sequencer {
    pub timeline: Timeline,
    pub tracks: Vec<SequencerTrack>,
    pub patterns: HashMap<String, Pattern>,
    pub current_pattern: Option<String>,
    pub bpm: f32,
    pub time_signature: (u8, u8), // (beats per measure, beat unit)
    pub swing: f32, // 0.0 = straight, 0.5 = maximum swing
}

#[derive(Debug, Clone)]
pub struct SequencerTrack {
    pub name: String,
    pub muted: bool,
    pub solo: bool,
    pub volume: f32,
    pub pan: f32,
    pub steps: Vec<SequencerStep>,
    pub length: usize,
    pub track_type: TrackType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrackType {
    Drum,
    Instrument,
    Audio,
    Control,
}

#[derive(Debug, Clone)]
pub struct SequencerStep {
    pub active: bool,
    pub velocity: f32,
    pub probability: f32,
    pub micro_timing: f32, // -0.5 to 0.5 step offset
    pub note: Option<u8>,
    pub parameters: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: String,
    pub length: usize, // in steps
    pub tracks: Vec<SequencerTrack>,
}

impl Sequencer {
    pub fn new(bpm: f32) -> Self {
        Self {
            timeline: Timeline::new(),
            tracks: Vec::new(),
            patterns: HashMap::new(),
            current_pattern: None,
            bpm,
            time_signature: (4, 4),
            swing: 0.0,
        }
    }
    
    pub fn add_track(&mut self, name: String, track_type: TrackType, length: usize) {
        let steps = (0..length).map(|_| SequencerStep {
            active: false,
            velocity: 0.8,
            probability: 1.0,
            micro_timing: 0.0,
            note: None,
            parameters: HashMap::new(),
        }).collect();
        
        let track = SequencerTrack {
            name,
            muted: false,
            solo: false,
            volume: 0.8,
            pan: 0.0,
            steps,
            length,
            track_type,
        };
        
        self.tracks.push(track);
    }
    
    pub fn set_step(&mut self, track_index: usize, step_index: usize, active: bool) {
        if let Some(track) = self.tracks.get_mut(track_index) {
            if let Some(step) = track.steps.get_mut(step_index) {
                step.active = active;
            }
        }
    }
    
    pub fn set_step_velocity(&mut self, track_index: usize, step_index: usize, velocity: f32) {
        if let Some(track) = self.tracks.get_mut(track_index) {
            if let Some(step) = track.steps.get_mut(step_index) {
                step.velocity = velocity.clamp(0.0, 1.0);
            }
        }
    }
    
    pub fn set_step_probability(&mut self, track_index: usize, step_index: usize, probability: f32) {
        if let Some(track) = self.tracks.get_mut(track_index) {
            if let Some(step) = track.steps.get_mut(step_index) {
                step.probability = probability.clamp(0.0, 1.0);
            }
        }
    }
    
    pub fn get_current_step(&self) -> usize {
        let beat_duration = 60.0 / self.bpm as f64;
        let step_duration = beat_duration / 4.0; // 16th note steps
        (self.timeline.current_time / step_duration) as usize
    }
    
    pub fn get_active_steps(&self, track_index: usize) -> Vec<(usize, &SequencerStep)> {
        if let Some(track) = self.tracks.get(track_index) {
            let current_step = self.get_current_step();
            
            track.steps
                .iter()
                .enumerate()
                .filter(|(step_idx, step)| {
                    // Check if this step should trigger
                    *step_idx == current_step % track.length &&
                    step.active &&
                    rand::random::<f32>() < step.probability
                })
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn save_pattern(&mut self, name: String) {
        let pattern = Pattern {
            name: name.clone(),
            length: self.tracks.first().map(|t| t.length).unwrap_or(16),
            tracks: self.tracks.clone(),
        };
        
        self.patterns.insert(name.clone(), pattern);
        self.current_pattern = Some(name);
    }
    
    pub fn load_pattern(&mut self, name: &str) -> crate::Result<()> {
        if let Some(pattern) = self.patterns.get(name) {
            self.tracks = pattern.tracks.clone();
            self.current_pattern = Some(name.to_string());
            Ok(())
        } else {
            Err(crate::errors::synthesis_error(crate::errors::ErrorKind::UnknownFunction, format!("Pattern '{}' not found", name)))
        }
    }
    
    pub fn clear_pattern(&mut self) {
        for track in &mut self.tracks {
            for step in &mut track.steps {
                step.active = false;
            }
        }
    }
    
    pub fn randomize_pattern(&mut self, density: f32) {
        for track in &mut self.tracks {
            for step in &mut track.steps {
                step.active = rand::random::<f32>() < density;
                if step.active {
                    step.velocity = 0.5 + rand::random::<f32>() * 0.5;
                    step.micro_timing = (rand::random::<f32>() - 0.5) * 0.2;
                }
            }
        }
    }
    
    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.clamp(60.0, 200.0);
    }
    
    pub fn set_swing(&mut self, swing: f32) {
        self.swing = swing.clamp(0.0, 0.5);
    }
    
    pub fn apply_swing_timing(&self, step_index: usize, base_time: f64) -> f64 {
        if step_index % 2 == 1 && self.swing > 0.0 {
            // Apply swing to off-beats
            let beat_duration = 60.0 / self.bpm as f64 / 4.0; // 16th note duration
            base_time + beat_duration * self.swing as f64
        } else {
            base_time
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnimationCurve {
    pub keyframes: Vec<Keyframe>,
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f64,
    pub value: f32,
    pub easing: EasingType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterpolationType {
    Linear,
    Bezier,
    Step,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    Back,
}

impl AnimationCurve {
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
            interpolation: InterpolationType::Linear,
        }
    }
    
    pub fn add_keyframe(&mut self, time: f64, value: f32, easing: EasingType) {
        self.keyframes.push(Keyframe { time, value, easing });
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }
    
    pub fn evaluate(&self, time: f64) -> f32 {
        if self.keyframes.is_empty() {
            return 0.0;
        }
        
        if self.keyframes.len() == 1 {
            return self.keyframes[0].value;
        }
        
        // Find surrounding keyframes
        let mut before_idx = 0;
        let mut after_idx = self.keyframes.len() - 1;
        
        for (i, keyframe) in self.keyframes.iter().enumerate() {
            if keyframe.time <= time {
                before_idx = i;
            }
            if keyframe.time >= time {
                after_idx = i;
                break;
            }
        }
        
        if before_idx == after_idx {
            return self.keyframes[before_idx].value;
        }
        
        let before = &self.keyframes[before_idx];
        let after = &self.keyframes[after_idx];
        
        // Calculate interpolation parameter
        let t = ((time - before.time) / (after.time - before.time)) as f32;
        let t = t.clamp(0.0, 1.0);
        
        // Apply easing
        let eased_t = match before.easing {
            EasingType::Linear => t,
            EasingType::EaseIn => t * t,
            EasingType::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingType::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
            EasingType::Bounce => {
                let t = 1.0 - t;
                1.0 - if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
            EasingType::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.3;
                    let s = p / 4.0;
                    -(2.0_f32.powf(10.0 * (t - 1.0))) * ((t - 1.0 - s) * (2.0 * std::f32::consts::PI) / p).sin()
                }
            }
            EasingType::Back => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
        };
        
        // Interpolate between values
        match self.interpolation {
            InterpolationType::Linear => before.value + (after.value - before.value) * eased_t,
            InterpolationType::Step => before.value,
            InterpolationType::Bezier => {
                // Simple bezier interpolation (could be enhanced with control points)
                before.value + (after.value - before.value) * eased_t
            }
        }
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AnimationCurve {
    fn default() -> Self {
        Self::new()
    }
}

// Module functions for the runtime
pub fn timeline_create(_args: &[Value]) -> crate::Result<Value> {
    let timeline = Timeline::new();
    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::String("timeline".to_string()));
    result.insert("current_time".to_string(), Value::Float(timeline.current_time));
    result.insert("is_playing".to_string(), Value::Boolean(timeline.is_playing));
    Ok(Value::Object(result))
}

pub fn sequencer_create(args: &[Value]) -> crate::Result<Value> {
    let bpm = args.get(0)
        .and_then(|v| v.as_number())
        .unwrap_or(120.0) as f32;
    
    let sequencer = Sequencer::new(bpm);
    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::String("sequencer".to_string()));
    result.insert("bpm".to_string(), Value::Float(sequencer.bpm as f64));
    result.insert("tracks".to_string(), Value::Array(Vec::new()));
    Ok(Value::Object(result))
}

pub fn animation_curve_create(_args: &[Value]) -> crate::Result<Value> {
    let curve = AnimationCurve::new();
    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::String("animation_curve".to_string()));
    result.insert("keyframes".to_string(), Value::Array(Vec::new()));
    Ok(Value::Object(result))
}

pub fn every(args: &[Value]) -> crate::Result<Value> {
    if args.is_empty() {
        return Err(crate::errors::synthesis_error(crate::errors::ErrorKind::InvalidExpression, "every requires a duration argument"));
    }
    
    let duration = args[0].as_number().ok_or_else(|| crate::errors::synthesis_error(crate::errors::ErrorKind::TypeMismatch, "Duration must be a number"))?;
    
    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::String("temporal_every".to_string()));
    result.insert("duration".to_string(), Value::Float(duration));
    Ok(Value::Object(result))
}

pub fn after(args: &[Value]) -> crate::Result<Value> {
    if args.is_empty() {
        return Err(crate::errors::synthesis_error(crate::errors::ErrorKind::InvalidExpression, "after requires a duration argument"));
    }
    
    let duration = args[0].as_number().ok_or_else(|| crate::errors::synthesis_error(crate::errors::ErrorKind::TypeMismatch, "Duration must be a number"))?;
    
    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::String("temporal_after".to_string()));
    result.insert("duration".to_string(), Value::Float(duration));
    Ok(Value::Object(result))
}

pub fn sequence(args: &[Value]) -> crate::Result<Value> {
    let steps = args.iter().cloned().collect();
    
    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::String("sequence".to_string()));
    result.insert("steps".to_string(), Value::Array(steps));
    Ok(Value::Object(result))
}