use crate::runtime::types::{Value, DataType};
use crate::runtime::units::UnitValue;
use crate::errors::ErrorKind;
use std::collections::HashMap;
use std::fmt;

/// Creative type system with automatic coercion for artistic workflows
/// This system prioritizes creative flow over strict type safety
#[derive(Debug, Clone)]
pub struct CreativeTypeSystem {
    pub coercion_rules: Vec<CoercionRule>,
    pub creative_contexts: HashMap<String, CreativeContext>,
    pub type_inference_enabled: bool,
}

/// Rules for automatic type coercion in creative contexts
#[derive(Debug, Clone)]
pub struct CoercionRule {
    pub name: String,
    pub from_type: CreativeType,
    pub to_type: CreativeType,
    pub coercion_fn: CoercionFunction,
    pub context: CoercionContext,
    pub priority: u32,
}

/// Enhanced creative types that understand artistic concepts
#[derive(Debug, Clone, PartialEq)]
pub enum CreativeType {
    // Basic types with creative semantics
    Number(NumberType),
    Text(TextType),
    Boolean(BooleanType),
    
    // Audio-specific types
    Audio(AudioType),
    Frequency(FrequencyType),
    Amplitude(AmplitudeType),
    Duration(DurationType),
    
    // Visual-specific types
    Visual(VisualType),
    Color(ColorType),
    Position(PositionType),
    
    // Creative composite types
    Chord(ChordType),
    Scale(ScaleType),
    Rhythm(RhythmType),
    Palette(PaletteType),
    
    // Stream types
    Stream(StreamType),
    
    // Generic creative value
    Creative(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberType {
    Integer,
    Float,
    Percentage,  // 0-100% mapped to 0.0-1.0
    Normalized,  // Always 0.0-1.0
    MIDI,        // 0-127 integer range
    Decibel,     // Audio level in dB
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextType {
    PlainText,
    ChordName,   // "Cmaj7", "Am", etc.
    NoteName,    // "C4", "F#3", etc.
    ColorName,   // "red", "warm_blue", etc.
}

#[derive(Debug, Clone, PartialEq)]
pub enum BooleanType {
    Switch,      // Simple on/off
    Gate,        // Audio gate (with timing)
    Trigger,     // One-shot activation
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioType {
    Mono,
    Stereo,
    Multichannel(u8),
    Spatial,     // 3D positioned audio
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrequencyType {
    Hertz,       // Raw frequency value
    Note,        // Musical note (C4 = 261.63 Hz)
    MIDINote,    // MIDI note number (C4 = 60)
    Pitch,       // Perceptual pitch
}

#[derive(Debug, Clone, PartialEq)]
pub enum AmplitudeType {
    Linear,      // 0.0-1.0 linear scale
    Decibel,     // dB scale
    MIDI,        // MIDI velocity 0-127
    Perceived,   // Perceptually linear
}

#[derive(Debug, Clone, PartialEq)]
pub enum DurationType {
    Seconds,
    Milliseconds,
    Beats,       // Musical beats
    Measures,    // Musical measures
    Samples,     // Audio samples
}

#[derive(Debug, Clone, PartialEq)]
pub enum VisualType {
    RGB,
    HSV,
    LAB,
    Texture,
    Geometry,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorType {
    RGB(f32, f32, f32),
    HSV(f32, f32, f32),
    Named(String),
    Palette(Vec<(f32, f32, f32)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionType {
    Absolute,    // Pixel coordinates
    Relative,    // 0.0-1.0 relative to container
    Percentage,  // 0-100% of container
    Centered,    // Center-relative positioning
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChordType {
    Triad(String, ChordQuality),
    Seventh(String, ChordQuality),
    Extended(String, Vec<u8>),
    Custom(Vec<f32>), // Frequency ratios
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Sus2,
    Sus4,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScaleType {
    Major(String),      // Root note
    Minor(String),
    Pentatonic(String),
    Blues(String),
    Custom(Vec<f32>),   // Interval ratios
}

#[derive(Debug, Clone, PartialEq)]
pub enum RhythmType {
    Simple(f32),        // Beats per measure
    Compound(Vec<f32>), // Complex rhythm pattern
    Groove(String),     // Named groove pattern
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaletteType {
    Monochromatic(ColorType),
    Complementary(ColorType, ColorType),
    Triadic(ColorType, ColorType, ColorType),
    Custom(Vec<ColorType>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Audio(AudioType),
    Visual(VisualType),
    Control,
    MIDI,
    Generic,
}

/// Context information for type coercion
#[derive(Debug, Clone, PartialEq)]
pub enum CoercionContext {
    Musical { key: String, tempo: f32, time_signature: (u8, u8) },
    Visual { palette: String, resolution: (u32, u32) },
    Audio { sample_rate: f32, bit_depth: u8 },
    Generic,
}

/// Creative context for interpreting values
#[derive(Debug, Clone)]
pub struct CreativeContext {
    pub name: String,
    pub musical_key: Option<String>,
    pub tempo_bpm: Option<f32>,
    pub color_palette: Option<PaletteType>,
    pub coordinate_system: PositionType,
    pub preferred_units: HashMap<String, String>,
}

/// Type coercion functions
#[derive(Debug, Clone)]
pub enum CoercionFunction {
    Direct,                    // Simple casting
    Scale(f32, f32),          // Scale from one range to another
    Musical(MusicalConversion),
    Visual(VisualConversion),
    Custom(String),           // Named custom function
}

#[derive(Debug, Clone, PartialEq)]
pub enum MusicalConversion {
    NoteToFrequency,
    FrequencyToNote,
    MIDIToFrequency,
    ChordToFrequencies,
    ScaleToFrequencies,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VisualConversion {
    RGBToHSV,
    HSVToRGB,
    ColorNameToRGB,
    PaletteToColors,
    PercentageToPixel,
}

impl CreativeTypeSystem {
    pub fn new() -> Self {
        let mut system = Self {
            coercion_rules: Vec::new(),
            creative_contexts: HashMap::new(),
            type_inference_enabled: true,
        };
        
        system.register_default_coercion_rules();
        system.register_default_contexts();
        
        system
    }
    
    /// Automatically coerce a value to the target type in a creative-friendly way
    pub fn coerce_value(&self, value: &Value, target_type: &CreativeType, context: Option<&str>) -> Result<Value, String> {
        let source_type = self.infer_creative_type(value, context);
        
        if self.types_compatible(&source_type, target_type) {
            return Ok(value.clone());
        }
        
        // Find applicable coercion rule
        for rule in &self.coercion_rules {
            if self.rule_applies(&rule, &source_type, target_type, context) {
                return self.apply_coercion_rule(value, rule, context);
            }
        }
        
        // If no rule found, try intelligent defaults
        self.apply_default_coercion(value, target_type, context)
    }
    
    /// Infer the creative type from a Value
    pub fn infer_creative_type(&self, value: &Value, context: Option<&str>) -> CreativeType {
        match value {
            Value::Integer(n) => {
                if *n >= 0 && *n <= 127 {
                    CreativeType::Number(NumberType::MIDI)
                } else {
                    CreativeType::Number(NumberType::Integer)
                }
            }
            Value::Float(f) => {
                if *f >= 0.0 && *f <= 1.0 {
                    CreativeType::Number(NumberType::Normalized)
                } else if *f > 20.0 && *f < 20000.0 {
                    // Likely a frequency
                    CreativeType::Frequency(FrequencyType::Hertz)
                } else {
                    CreativeType::Number(NumberType::Float)
                }
            }
            Value::String(s) => {
                if self.is_chord_name(s) {
                    CreativeType::Text(TextType::ChordName)
                } else if self.is_note_name(s) {
                    CreativeType::Text(TextType::NoteName)
                } else if self.is_color_name(s) {
                    CreativeType::Text(TextType::ColorName)
                } else {
                    CreativeType::Text(TextType::PlainText)
                }
            }
            Value::Boolean(_) => CreativeType::Boolean(BooleanType::Switch),
            Value::UnitValue(unit_val) => {
                match unit_val.unit.to_string().as_str() {
                    "hz" => CreativeType::Frequency(FrequencyType::Hertz),
                    "%" => CreativeType::Number(NumberType::Percentage),
                    "s" => CreativeType::Duration(DurationType::Seconds),
                    "ms" => CreativeType::Duration(DurationType::Milliseconds),
                    "db" => CreativeType::Number(NumberType::Decibel),
                    _ => CreativeType::Number(NumberType::Float),
                }
            }
            Value::Stream(stream) => {
                match stream.data_type {
                    DataType::Audio => CreativeType::Stream(StreamType::Audio(AudioType::Mono)),
                    DataType::Visual => CreativeType::Stream(StreamType::Visual(VisualType::RGB)),
                    DataType::MIDI => CreativeType::Stream(StreamType::MIDI),
                    DataType::Control => CreativeType::Stream(StreamType::Control),
                    _ => CreativeType::Stream(StreamType::Generic),
                }
            }
            _ => CreativeType::Creative(value.clone()),
        }
    }
    
    /// Check if two types are compatible without coercion
    fn types_compatible(&self, source: &CreativeType, target: &CreativeType) -> bool {
        match (source, target) {
            // Exact matches
            (a, b) if a == b => true,
            
            // Number type compatibility
            (CreativeType::Number(_), CreativeType::Number(_)) => true,
            
            // Frequency types are interconvertible
            (CreativeType::Frequency(_), CreativeType::Frequency(_)) => true,
            
            // Duration types are interconvertible
            (CreativeType::Duration(_), CreativeType::Duration(_)) => true,
            
            // Stream types with same underlying type
            (CreativeType::Stream(a), CreativeType::Stream(b)) => a == b,
            
            _ => false,
        }
    }
    
    /// Check if a coercion rule applies to the current situation
    fn rule_applies(&self, rule: &CoercionRule, source: &CreativeType, target: &CreativeType, context: Option<&str>) -> bool {
        // Check type compatibility
        if !self.types_compatible(&rule.from_type, source) || !self.types_compatible(&rule.to_type, target) {
            return false;
        }
        
        // Check context compatibility
        if let Some(ctx) = context {
            if let Some(creative_ctx) = self.creative_contexts.get(ctx) {
                // Context-specific logic would go here
                return true;
            }
        }
        
        true
    }
    
    /// Apply a coercion rule to transform a value
    fn apply_coercion_rule(&self, value: &Value, rule: &CoercionRule, context: Option<&str>) -> Result<Value, String> {
        match &rule.coercion_fn {
            CoercionFunction::Direct => Ok(value.clone()),
            
            CoercionFunction::Scale(from_max, to_max) => {
                if let Some(num) = value.as_number() {
                    let normalized = num / from_max;
                    let scaled = normalized * to_max;
                    Ok(Value::Float(scaled))
                } else {
                    Err("🎨 Can't scale non-numeric value".to_string())
                }
            }
            
            CoercionFunction::Musical(conversion) => {
                self.apply_musical_conversion(value, conversion, context)
            }
            
            CoercionFunction::Visual(conversion) => {
                self.apply_visual_conversion(value, conversion, context)
            }
            
            CoercionFunction::Custom(func_name) => {
                self.apply_custom_conversion(value, func_name, context)
            }
        }
    }
    
    /// Apply default coercion when no specific rule exists
    fn apply_default_coercion(&self, value: &Value, target_type: &CreativeType, _context: Option<&str>) -> Result<Value, String> {
        match target_type {
            CreativeType::Number(NumberType::Percentage) => {
                if let Some(num) = value.as_number() {
                    // Assume input is 0-1, convert to percentage
                    Ok(Value::Float((num * 100.0).min(100.0).max(0.0)))
                } else {
                    Err("🎨 Can't convert to percentage - need a number".to_string())
                }
            }
            
            CreativeType::Number(NumberType::Normalized) => {
                if let Some(num) = value.as_number() {
                    // Normalize to 0-1 range
                    if num > 1.0 {
                        Ok(Value::Float(num / 100.0)) // Assume percentage
                    } else {
                        Ok(Value::Float(num.min(1.0).max(0.0)))
                    }
                } else {
                    Err("🎨 Can't normalize non-numeric value".to_string())
                }
            }
            
            CreativeType::Frequency(FrequencyType::Hertz) => {
                match value {
                    Value::String(note) => {
                        if let Some(freq) = self.note_to_frequency(note) {
                            Ok(Value::Float(freq))
                        } else {
                            Err(format!("🎵 Don't recognize the note '{}'", note))
                        }
                    }
                    Value::Integer(midi_note) => {
                        let freq = self.midi_to_frequency(*midi_note as u8);
                        Ok(Value::Float(freq))
                    }
                    _ => Err("🎵 Need a note name or MIDI number to get frequency".to_string())
                }
            }
            
            _ => Err(format!("🎨 Don't know how to convert to {:?}", target_type))
        }
    }
    
    /// Apply musical conversions
    fn apply_musical_conversion(&self, value: &Value, conversion: &MusicalConversion, _context: Option<&str>) -> Result<Value, String> {
        match conversion {
            MusicalConversion::NoteToFrequency => {
                if let Value::String(note) = value {
                    if let Some(freq) = self.note_to_frequency(note) {
                        Ok(Value::Float(freq))
                    } else {
                        Err(format!("🎵 '{}' isn't a note I recognize", note))
                    }
                } else {
                    Err("🎵 Need a note name like 'C4' or 'F#3'".to_string())
                }
            }
            
            MusicalConversion::MIDIToFrequency => {
                if let Some(midi) = value.as_number() {
                    let freq = self.midi_to_frequency(midi as u8);
                    Ok(Value::Float(freq))
                } else {
                    Err("🎵 Need a MIDI note number (0-127)".to_string())
                }
            }
            
            MusicalConversion::ChordToFrequencies => {
                if let Value::String(chord) = value {
                    let frequencies = self.chord_to_frequencies(chord)?;
                    Ok(Value::Array(frequencies.into_iter().map(Value::Float).collect()))
                } else {
                    Err("🎵 Need a chord name like 'Cmaj7' or 'Am'".to_string())
                }
            }
            
            _ => Err("🎵 Musical conversion not implemented yet".to_string())
        }
    }
    
    /// Apply visual conversions
    fn apply_visual_conversion(&self, value: &Value, conversion: &VisualConversion, _context: Option<&str>) -> Result<Value, String> {
        match conversion {
            VisualConversion::ColorNameToRGB => {
                if let Value::String(color_name) = value {
                    if let Some((r, g, b)) = self.color_name_to_rgb(color_name) {
                        let mut rgb_object = HashMap::new();
                        rgb_object.insert("r".to_string(), Value::Float(r as f64));
                        rgb_object.insert("g".to_string(), Value::Float(g as f64));
                        rgb_object.insert("b".to_string(), Value::Float(b as f64));
                        Ok(Value::Object(rgb_object))
                    } else {
                        Err(format!("🎨 Don't know the color '{}'", color_name))
                    }
                } else {
                    Err("🎨 Need a color name like 'red' or 'warm_blue'".to_string())
                }
            }
            
            VisualConversion::PercentageToPixel => {
                if let Some(percentage) = value.as_number() {
                    // Would need screen/container dimensions from context
                    let pixels = (percentage / 100.0) * 1920.0; // Assume 1920 width
                    Ok(Value::Float(pixels))
                } else {
                    Err("🎨 Need a percentage value".to_string())
                }
            }
            
            _ => Err("🎨 Visual conversion not implemented yet".to_string())
        }
    }
    
    /// Apply custom conversions
    fn apply_custom_conversion(&self, value: &Value, func_name: &str, _context: Option<&str>) -> Result<Value, String> {
        match func_name {
            "creative_boost" => {
                // Creative enhancement function
                if let Some(num) = value.as_number() {
                    let boosted = num * 1.2; // 20% creative boost!
                    Ok(Value::Float(boosted))
                } else {
                    Ok(value.clone()) // Pass through non-numeric
                }
            }
            
            _ => Err(format!("🎨 Don't know the conversion '{}'", func_name))
        }
    }
    
    /// Register default coercion rules
    fn register_default_coercion_rules(&mut self) {
        // Number to percentage
        self.coercion_rules.push(CoercionRule {
            name: "float_to_percentage".to_string(),
            from_type: CreativeType::Number(NumberType::Float),
            to_type: CreativeType::Number(NumberType::Percentage),
            coercion_fn: CoercionFunction::Scale(1.0, 100.0),
            context: CoercionContext::Generic,
            priority: 10,
        });
        
        // Note name to frequency
        self.coercion_rules.push(CoercionRule {
            name: "note_to_frequency".to_string(),
            from_type: CreativeType::Text(TextType::NoteName),
            to_type: CreativeType::Frequency(FrequencyType::Hertz),
            coercion_fn: CoercionFunction::Musical(MusicalConversion::NoteToFrequency),
            context: CoercionContext::Musical { key: "C".to_string(), tempo: 120.0, time_signature: (4, 4) },
            priority: 20,
        });
        
        // Color name to RGB
        self.coercion_rules.push(CoercionRule {
            name: "color_name_to_rgb".to_string(),
            from_type: CreativeType::Text(TextType::ColorName),
            to_type: CreativeType::Color(ColorType::RGB(0.0, 0.0, 0.0)),
            coercion_fn: CoercionFunction::Visual(VisualConversion::ColorNameToRGB),
            context: CoercionContext::Visual { palette: "default".to_string(), resolution: (1920, 1080) },
            priority: 20,
        });
    }
    
    /// Register default creative contexts
    fn register_default_contexts(&mut self) {
        self.creative_contexts.insert("musical".to_string(), CreativeContext {
            name: "musical".to_string(),
            musical_key: Some("C".to_string()),
            tempo_bpm: Some(120.0),
            color_palette: None,
            coordinate_system: PositionType::Relative,
            preferred_units: {
                let mut units = HashMap::new();
                units.insert("frequency".to_string(), "hz".to_string());
                units.insert("time".to_string(), "beats".to_string());
                units.insert("amplitude".to_string(), "db".to_string());
                units
            },
        });
        
        self.creative_contexts.insert("visual".to_string(), CreativeContext {
            name: "visual".to_string(),
            musical_key: None,
            tempo_bpm: None,
            color_palette: Some(PaletteType::Triadic(
                ColorType::RGB(1.0, 0.0, 0.0),
                ColorType::RGB(0.0, 1.0, 0.0),
                ColorType::RGB(0.0, 0.0, 1.0),
            )),
            coordinate_system: PositionType::Percentage,
            preferred_units: {
                let mut units = HashMap::new();
                units.insert("position".to_string(), "%".to_string());
                units.insert("size".to_string(), "px".to_string());
                units.insert("opacity".to_string(), "normalized".to_string());
                units
            },
        });
    }
    
    // Helper functions for musical conversions
    
    fn is_chord_name(&self, s: &str) -> bool {
        s.contains("maj") || s.contains("min") || s.contains("dim") || s.contains("aug") || 
        s.ends_with('7') || s.ends_with("sus2") || s.ends_with("sus4")
    }
    
    fn is_note_name(&self, s: &str) -> bool {
        let notes = ["C", "D", "E", "F", "G", "A", "B"];
        notes.iter().any(|&note| s.starts_with(note)) && 
        (s.len() >= 2 && s.chars().last().unwrap().is_ascii_digit())
    }
    
    fn is_color_name(&self, s: &str) -> bool {
        let colors = ["red", "green", "blue", "yellow", "orange", "purple", "pink", "brown", "black", "white", "gray"];
        colors.iter().any(|&color| s.to_lowercase().contains(color))
    }
    
    fn note_to_frequency(&self, note: &str) -> Option<f32> {
        // Simple note to frequency conversion
        let note_frequencies = [
            ("C", 16.35), ("C#", 17.32), ("Db", 17.32),
            ("D", 18.35), ("D#", 19.45), ("Eb", 19.45),
            ("E", 20.60), ("F", 21.83), ("F#", 23.12), ("Gb", 23.12),
            ("G", 24.50), ("G#", 25.96), ("Ab", 25.96),
            ("A", 27.50), ("A#", 29.14), ("Bb", 29.14),
            ("B", 30.87),
        ];
        
        if note.len() < 2 {
            return None;
        }
        
        let note_name = &note[..note.len()-1];
        let octave = note.chars().last().unwrap().to_digit(10)? as i32;
        
        for (name, base_freq) in &note_frequencies {
            if *name == note_name {
                let freq = base_freq * 2.0_f32.powi(octave);
                return Some(freq);
            }
        }
        
        None
    }
    
    fn midi_to_frequency(&self, midi_note: u8) -> f32 {
        440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0)
    }
    
    fn chord_to_frequencies(&self, chord: &str) -> Result<Vec<f32>, String> {
        // Simple chord parsing - in a full implementation this would be more comprehensive
        let root = &chord[..1];
        let root_freq = self.note_to_frequency(&format!("{}4", root))
            .ok_or_else(|| format!("🎵 Can't find root note '{}'", root))?;
        
        // Major triad intervals (simple implementation)
        let intervals = if chord.contains("min") {
            vec![0.0, 3.0, 7.0] // Minor triad semitones
        } else {
            vec![0.0, 4.0, 7.0] // Major triad semitones
        };
        
        let frequencies = intervals.iter()
            .map(|&interval| root_freq * 2.0_f32.powf(interval / 12.0))
            .collect();
            
        Ok(frequencies)
    }
    
    fn color_name_to_rgb(&self, color: &str) -> Option<(f32, f32, f32)> {
        match color.to_lowercase().as_str() {
            "red" => Some((1.0, 0.0, 0.0)),
            "green" => Some((0.0, 1.0, 0.0)),
            "blue" => Some((0.0, 0.0, 1.0)),
            "yellow" => Some((1.0, 1.0, 0.0)),
            "orange" => Some((1.0, 0.5, 0.0)),
            "purple" => Some((0.5, 0.0, 0.5)),
            "pink" => Some((1.0, 0.0, 0.5)),
            "white" => Some((1.0, 1.0, 1.0)),
            "black" => Some((0.0, 0.0, 0.0)),
            "warm_blue" => Some((0.3, 0.6, 1.0)),
            "cool_red" => Some((0.8, 0.1, 0.3)),
            _ => None,
        }
    }
    
    /// Get type information for debugging and user feedback
    pub fn describe_type(&self, creative_type: &CreativeType) -> String {
        match creative_type {
            CreativeType::Number(num_type) => format!("Number ({})", match num_type {
                NumberType::Integer => "whole number",
                NumberType::Float => "decimal number",
                NumberType::Percentage => "percentage 0-100%",
                NumberType::Normalized => "value 0.0-1.0",
                NumberType::MIDI => "MIDI value 0-127",
                NumberType::Decibel => "decibel level",
            }),
            CreativeType::Frequency(freq_type) => format!("Frequency ({})", match freq_type {
                FrequencyType::Hertz => "Hz",
                FrequencyType::Note => "musical note",
                FrequencyType::MIDINote => "MIDI note",
                FrequencyType::Pitch => "pitch",
            }),
            CreativeType::Text(text_type) => format!("Text ({})", match text_type {
                TextType::PlainText => "plain text",
                TextType::ChordName => "chord name",
                TextType::NoteName => "note name",
                TextType::ColorName => "color name",
            }),
            _ => format!("{:?}", creative_type),
        }
    }
}

impl Default for CreativeTypeSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CreativeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CreativeType::Number(nt) => write!(f, "Number({:?})", nt),
            CreativeType::Frequency(ft) => write!(f, "Frequency({:?})", ft),
            CreativeType::Text(tt) => write!(f, "Text({:?})", tt),
            _ => write!(f, "{:?}", self),
        }
    }
}