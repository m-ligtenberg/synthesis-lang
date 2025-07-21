use midir::{MidiInput, MidiOutput, MidiInputConnection, MidiOutputConnection};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MidiNote {
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidiCC {
    pub channel: u8,
    pub controller: u8,
    pub value: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MidiMessage {
    NoteOn(MidiNote),
    NoteOff(MidiNote),
    ControlChange(MidiCC),
    PitchBend { channel: u8, value: u16 },
    ProgramChange { channel: u8, program: u8 },
    Aftertouch { channel: u8, pressure: u8 },
    SystemExclusive(Vec<u8>),
    Clock,
    Start,
    Continue,
    Stop,
}

impl MidiMessage {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        
        let status = bytes[0];
        
        match status {
            0xF8 => Some(MidiMessage::Clock),
            0xFA => Some(MidiMessage::Start),
            0xFB => Some(MidiMessage::Continue),
            0xFC => Some(MidiMessage::Stop),
            _ => {
                let channel = status & 0x0F;
                let command = status & 0xF0;
                
                match command {
                    0x80 => { // Note Off
                        if bytes.len() >= 3 {
                            Some(MidiMessage::NoteOff(MidiNote {
                                channel,
                                note: bytes[1],
                                velocity: bytes[2],
                            }))
                        } else { None }
                    }
                    0x90 => { // Note On
                        if bytes.len() >= 3 {
                            let note = MidiNote {
                                channel,
                                note: bytes[1],
                                velocity: bytes[2],
                            };
                            // Velocity 0 is actually Note Off
                            if note.velocity == 0 {
                                Some(MidiMessage::NoteOff(note))
                            } else {
                                Some(MidiMessage::NoteOn(note))
                            }
                        } else { None }
                    }
                    0xB0 => { // Control Change
                        if bytes.len() >= 3 {
                            Some(MidiMessage::ControlChange(MidiCC {
                                channel,
                                controller: bytes[1],
                                value: bytes[2],
                            }))
                        } else { None }
                    }
                    0xC0 => { // Program Change
                        if bytes.len() >= 2 {
                            Some(MidiMessage::ProgramChange {
                                channel,
                                program: bytes[1],
                            })
                        } else { None }
                    }
                    0xD0 => { // Channel Aftertouch
                        if bytes.len() >= 2 {
                            Some(MidiMessage::Aftertouch {
                                channel,
                                pressure: bytes[1],
                            })
                        } else { None }
                    }
                    0xE0 => { // Pitch Bend
                        if bytes.len() >= 3 {
                            let value = ((bytes[2] as u16) << 7) | (bytes[1] as u16);
                            Some(MidiMessage::PitchBend { channel, value })
                        } else { None }
                    }
                    0xF0 => { // System Exclusive
                        if status == 0xF0 {
                            Some(MidiMessage::SystemExclusive(bytes.to_vec()))
                        } else { None }
                    }
                    _ => None,
                }
            }
        }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            MidiMessage::NoteOn(note) => vec![0x90 | note.channel, note.note, note.velocity],
            MidiMessage::NoteOff(note) => vec![0x80 | note.channel, note.note, note.velocity],
            MidiMessage::ControlChange(cc) => vec![0xB0 | cc.channel, cc.controller, cc.value],
            MidiMessage::ProgramChange { channel, program } => vec![0xC0 | channel, *program],
            MidiMessage::Aftertouch { channel, pressure } => vec![0xD0 | channel, *pressure],
            MidiMessage::PitchBend { channel, value } => {
                vec![0xE0 | channel, (*value & 0x7F) as u8, ((*value >> 7) & 0x7F) as u8]
            }
            MidiMessage::SystemExclusive(data) => data.clone(),
            MidiMessage::Clock => vec![0xF8],
            MidiMessage::Start => vec![0xFA],
            MidiMessage::Continue => vec![0xFB],
            MidiMessage::Stop => vec![0xFC],
        }
    }
}

pub struct MidiInputManager {
    input: MidiInput,
    connections: HashMap<String, MidiInputConnection<()>>,
    message_buffer: Arc<Mutex<Vec<(Instant, MidiMessage)>>>,
    note_states: Arc<Mutex<HashMap<(u8, u8), bool>>>, // (channel, note) -> is_pressed
    cc_values: Arc<Mutex<HashMap<(u8, u8), u8>>>, // (channel, controller) -> value
}

impl MidiInputManager {
    pub fn new() -> crate::Result<Self> {
        let input = MidiInput::new("Synthesis MIDI Input")?;
        
        Ok(Self {
            input,
            connections: HashMap::new(),
            message_buffer: Arc::new(Mutex::new(Vec::new())),
            note_states: Arc::new(Mutex::new(HashMap::new())),
            cc_values: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    pub fn list_ports(&self) -> Vec<(usize, String)> {
        self.input
            .ports()
            .iter()
            .enumerate()
            .filter_map(|(i, port)| {
                self.input.port_name(port).ok().map(|name| (i, name))
            })
            .collect()
    }
    
    pub fn connect_port(&mut self, port_index: usize, port_name: String) -> crate::Result<()> {
        let ports = self.input.ports();
        let port = ports.get(port_index)
            .ok_or_else(|| anyhow::anyhow!("Invalid MIDI port index: {}", port_index))?;
        
        let message_buffer = Arc::clone(&self.message_buffer);
        let note_states = Arc::clone(&self.note_states);
        let cc_values = Arc::clone(&self.cc_values);
        
        // Create a new MidiInput for this connection
        let input = MidiInput::new("Synthesis MIDI Input")?;
        let connection = input.connect(port, "synthesis-input", move |_timestamp, bytes, _| {
            if let Some(message) = MidiMessage::from_bytes(bytes) {
                // Store in message buffer
                {
                    let mut buffer = message_buffer.lock().unwrap();
                    buffer.push((Instant::now(), message));
                    
                    // Keep buffer size manageable
                    if buffer.len() > 1000 {
                        buffer.drain(0..500);
                    }
                }
                
                // Update state tracking
                match message {
                    MidiMessage::NoteOn(note) => {
                        let mut states = note_states.lock().unwrap();
                        states.insert((note.channel, note.note), true);
                    }
                    MidiMessage::NoteOff(note) => {
                        let mut states = note_states.lock().unwrap();
                        states.insert((note.channel, note.note), false);
                    }
                    MidiMessage::ControlChange(cc) => {
                        let mut values = cc_values.lock().unwrap();
                        values.insert((cc.channel, cc.controller), cc.value);
                    }
                    _ => {}
                }
            }
        }, ())?;
        
        self.connections.insert(port_name, connection);
        Ok(())
    }
    
    pub fn disconnect_port(&mut self, port_name: &str) {
        self.connections.remove(port_name);
    }
    
    pub fn get_messages_since(&self, since: Instant) -> Vec<MidiMessage> {
        let buffer = self.message_buffer.lock().unwrap();
        buffer
            .iter()
            .filter(|(timestamp, _)| *timestamp >= since)
            .map(|(_, message)| *message)
            .collect()
    }
    
    pub fn get_recent_messages(&self, max_count: usize) -> Vec<MidiMessage> {
        let buffer = self.message_buffer.lock().unwrap();
        buffer
            .iter()
            .rev()
            .take(max_count)
            .map(|(_, message)| *message)
            .collect()
    }
    
    pub fn is_note_pressed(&self, channel: u8, note: u8) -> bool {
        let states = self.note_states.lock().unwrap();
        states.get(&(channel, note)).copied().unwrap_or(false)
    }
    
    pub fn get_cc_value(&self, channel: u8, controller: u8) -> Option<u8> {
        let values = self.cc_values.lock().unwrap();
        values.get(&(channel, controller)).copied()
    }
    
    pub fn get_pressed_notes(&self) -> Vec<(u8, u8)> {
        let states = self.note_states.lock().unwrap();
        states
            .iter()
            .filter(|(_, &pressed)| pressed)
            .map(|((channel, note), _)| (*channel, *note))
            .collect()
    }
    
    pub fn clear_message_buffer(&self) {
        let mut buffer = self.message_buffer.lock().unwrap();
        buffer.clear();
    }
}

pub struct MidiOutputManager {
    output: MidiOutput,
    connections: HashMap<String, MidiOutputConnection>,
}

impl MidiOutputManager {
    pub fn new() -> crate::Result<Self> {
        let output = MidiOutput::new("Synthesis MIDI Output")?;
        
        Ok(Self {
            output,
            connections: HashMap::new(),
        })
    }
    
    pub fn list_ports(&self) -> Vec<(usize, String)> {
        self.output
            .ports()
            .iter()
            .enumerate()
            .filter_map(|(i, port)| {
                self.output.port_name(port).ok().map(|name| (i, name))
            })
            .collect()
    }
    
    pub fn connect_port(&mut self, port_index: usize, port_name: String) -> crate::Result<()> {
        let ports = self.output.ports();
        let port = ports.get(port_index)
            .ok_or_else(|| anyhow::anyhow!("Invalid MIDI port index: {}", port_index))?;
        
        // Create a new MidiOutput for this connection
        let output = MidiOutput::new("Synthesis MIDI Output")?;
        let connection = output.connect(port, "synthesis-output")?;
        
        self.connections.insert(port_name, connection);
        Ok(())
    }
    
    pub fn disconnect_port(&mut self, port_name: &str) {
        self.connections.remove(port_name);
    }
    
    pub fn send_message(&mut self, port_name: &str, message: MidiMessage) -> crate::Result<()> {
        if let Some(connection) = self.connections.get_mut(port_name) {
            let bytes = message.to_bytes();
            connection.send(&bytes)?;
        } else {
            return Err(anyhow::anyhow!("MIDI output port '{}' not connected", port_name));
        }
        Ok(())
    }
    
    pub fn send_note_on(&mut self, port_name: &str, channel: u8, note: u8, velocity: u8) -> crate::Result<()> {
        let message = MidiMessage::NoteOn(MidiNote { channel, note, velocity });
        self.send_message(port_name, message)
    }
    
    pub fn send_note_off(&mut self, port_name: &str, channel: u8, note: u8) -> crate::Result<()> {
        let message = MidiMessage::NoteOff(MidiNote { channel, note, velocity: 0 });
        self.send_message(port_name, message)
    }
    
    pub fn send_cc(&mut self, port_name: &str, channel: u8, controller: u8, value: u8) -> crate::Result<()> {
        let message = MidiMessage::ControlChange(MidiCC { channel, controller, value });
        self.send_message(port_name, message)
    }
    
    pub fn send_program_change(&mut self, port_name: &str, channel: u8, program: u8) -> crate::Result<()> {
        let message = MidiMessage::ProgramChange { channel, program };
        self.send_message(port_name, message)
    }
}

pub struct MidiClock {
    bpm: f32,
    is_running: bool,
    tick_count: u64,
    last_tick: Instant,
    tick_interval: Duration,
}

impl MidiClock {
    pub fn new(bpm: f32) -> Self {
        Self {
            bpm,
            is_running: false,
            tick_count: 0,
            last_tick: Instant::now(),
            tick_interval: Duration::from_nanos((60_000_000_000.0 / (bpm * 24.0)) as u64),
        }
    }
    
    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        self.tick_interval = Duration::from_nanos((60_000_000_000.0 / (bpm * 24.0)) as u64);
    }
    
    pub fn start(&mut self) {
        self.is_running = true;
        self.last_tick = Instant::now();
    }
    
    pub fn stop(&mut self) {
        self.is_running = false;
    }
    
    pub fn reset(&mut self) {
        self.tick_count = 0;
        self.last_tick = Instant::now();
    }
    
    pub fn update(&mut self) -> Vec<MidiMessage> {
        let mut messages = Vec::new();
        
        if !self.is_running {
            return messages;
        }
        
        let now = Instant::now();
        while now.duration_since(self.last_tick) >= self.tick_interval {
            messages.push(MidiMessage::Clock);
            self.tick_count += 1;
            self.last_tick += self.tick_interval;
        }
        
        messages
    }
    
    pub fn get_position(&self) -> f64 {
        (self.tick_count as f64) / 24.0 // 24 ticks per quarter note
    }
    
    pub fn get_bpm(&self) -> f32 {
        self.bpm
    }
    
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

// Utility functions for MIDI note/frequency conversion
pub fn note_to_frequency(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

pub fn frequency_to_note(freq: f32) -> u8 {
    (69.0 + 12.0 * (freq / 440.0).log2()).round().max(0.0).min(127.0) as u8
}

pub fn note_name(note: u8) -> String {
    let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i32 - 1;
    format!("{}{}", names[(note % 12) as usize], octave)
}