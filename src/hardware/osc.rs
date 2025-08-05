use rosc::{OscMessage, OscPacket, OscType, decoder, encoder};
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct OscParameter {
    pub address: String,
    pub value: OscValue,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub enum OscValue {
    Float(f32),
    Int(i32),
    String(String),
    Bool(bool),
    Array(Vec<OscValue>),
}

impl From<OscType> for OscValue {
    fn from(osc_type: OscType) -> Self {
        match osc_type {
            OscType::Float(f) => OscValue::Float(f),
            OscType::Int(i) => OscValue::Int(i),
            OscType::String(s) => OscValue::String(s),
            OscType::Bool(b) => OscValue::Bool(b),
            _ => OscValue::Float(0.0), // Default fallback
        }
    }
}

impl Into<OscType> for OscValue {
    fn into(self) -> OscType {
        match self {
            OscValue::Float(f) => OscType::Float(f),
            OscValue::Int(i) => OscType::Int(i),
            OscValue::String(s) => OscType::String(s),
            OscValue::Bool(b) => OscType::Bool(b),
            OscValue::Array(_) => OscType::Float(0.0), // Arrays need special handling
        }
    }
}

pub struct OscServer {
    socket: Option<UdpSocket>,
    parameters: Arc<Mutex<HashMap<String, OscParameter>>>,
    address_patterns: HashMap<String, Box<dyn Fn(&OscMessage) + Send>>,
    is_running: bool,
}

impl OscServer {
    pub fn new() -> Self {
        Self {
            socket: None,
            parameters: Arc::new(Mutex::new(HashMap::new())),
            address_patterns: HashMap::new(),
            is_running: false,
        }
    }
    
    pub fn bind<A: ToSocketAddrs>(&mut self, addr: A) -> crate::Result<()> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        self.socket = Some(socket);
        Ok(())
    }
    
    pub fn start_listening(&mut self) -> crate::Result<()> {
        if let Some(socket) = &self.socket {
            self.is_running = true;
            let socket_clone = socket.try_clone()?;
            let parameters = Arc::clone(&self.parameters);
            
            thread::spawn(move || {
                let mut buffer = [0u8; rosc::decoder::MTU];
                
                while let Ok((size, _addr)) = socket_clone.recv_from(&mut buffer) {
                    if let Ok((_, packet)) = decoder::decode_udp(&buffer[..size]) {
                        match packet {
                            OscPacket::Message(msg) => {
                                // Store parameter value
                                if let Some(arg) = msg.args.first() {
                                    let param = OscParameter {
                                        address: msg.addr.clone(),
                                        value: OscValue::from(arg.clone()),
                                        timestamp: Instant::now(),
                                    };
                                    
                                    let mut params = parameters.lock().unwrap();
                                    params.insert(msg.addr.clone(), param);
                                }
                            }
                            OscPacket::Bundle(bundle) => {
                                // Handle bundles (multiple messages with timestamps)
                                for packet in bundle.content {
                                    if let OscPacket::Message(msg) = packet {
                                        if let Some(arg) = msg.args.first() {
                                            let param = OscParameter {
                                                address: msg.addr.clone(),
                                                value: OscValue::from(arg.clone()),
                                                timestamp: Instant::now(),
                                            };
                                            
                                            let mut params = parameters.lock().unwrap();
                                            params.insert(msg.addr.clone(), param);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
        
        Ok(())
    }
    
    pub fn stop_listening(&mut self) {
        self.is_running = false;
    }
    
    pub fn get_parameter(&self, address: &str) -> Option<OscParameter> {
        let params = self.parameters.lock().unwrap();
        params.get(address).cloned()
    }
    
    pub fn get_float(&self, address: &str) -> Option<f32> {
        self.get_parameter(address).and_then(|param| {
            match param.value {
                OscValue::Float(f) => Some(f),
                OscValue::Int(i) => Some(i as f32),
                _ => None,
            }
        })
    }
    
    pub fn get_int(&self, address: &str) -> Option<i32> {
        self.get_parameter(address).and_then(|param| {
            match param.value {
                OscValue::Int(i) => Some(i),
                OscValue::Float(f) => Some(f as i32),
                _ => None,
            }
        })
    }
    
    pub fn get_string(&self, address: &str) -> Option<String> {
        self.get_parameter(address).and_then(|param| {
            match param.value {
                OscValue::String(s) => Some(s),
                _ => None,
            }
        })
    }
    
    pub fn get_bool(&self, address: &str) -> Option<bool> {
        self.get_parameter(address).and_then(|param| {
            match param.value {
                OscValue::Bool(b) => Some(b),
                OscValue::Float(f) => Some(f > 0.5),
                OscValue::Int(i) => Some(i != 0),
                _ => None,
            }
        })
    }
    
    pub fn get_all_parameters(&self) -> HashMap<String, OscParameter> {
        let params = self.parameters.lock().unwrap();
        params.clone()
    }
    
    pub fn clear_parameters(&self) {
        let mut params = self.parameters.lock().unwrap();
        params.clear();
    }
    
    pub fn register_pattern_handler<F>(&mut self, pattern: String, handler: F)
    where
        F: Fn(&OscMessage) + Send + 'static,
    {
        self.address_patterns.insert(pattern, Box::new(handler));
    }
}

pub struct OscClient {
    socket: Option<UdpSocket>,
    target_addr: Option<SocketAddr>,
}

impl OscClient {
    pub fn new() -> Self {
        Self {
            socket: None,
            target_addr: None,
        }
    }
    
    pub fn connect<A: ToSocketAddrs>(&mut self, target: A) -> crate::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let target_addr = target.to_socket_addrs()?.next()
            .ok_or_else(|| anyhow::anyhow!("Invalid target address"))?;
        
        self.socket = Some(socket);
        self.target_addr = Some(target_addr);
        Ok(())
    }
    
    pub fn send_message(&self, addr: &str, args: Vec<OscValue>) -> crate::Result<()> {
        if let (Some(socket), Some(target_addr)) = (&self.socket, &self.target_addr) {
            let osc_args: Vec<OscType> = args.into_iter().map(|v| v.into()).collect();
            
            let msg = OscMessage {
                addr: addr.to_string(),
                args: osc_args,
            };
            
            let packet = OscPacket::Message(msg);
            let encoded = encoder::encode(&packet)?;
            
            socket.send_to(&encoded, target_addr)?;
        } else {
            return Err(anyhow::anyhow!("OSC client not connected").into());
        }
        
        Ok(())
    }
    
    pub fn send_float(&self, addr: &str, value: f32) -> crate::Result<()> {
        self.send_message(addr, vec![OscValue::Float(value)])
    }
    
    pub fn send_int(&self, addr: &str, value: i32) -> crate::Result<()> {
        self.send_message(addr, vec![OscValue::Int(value)])
    }
    
    pub fn send_string(&self, addr: &str, value: &str) -> crate::Result<()> {
        self.send_message(addr, vec![OscValue::String(value.to_string())])
    }
    
    pub fn send_bool(&self, addr: &str, value: bool) -> crate::Result<()> {
        self.send_message(addr, vec![OscValue::Bool(value)])
    }
    
    pub fn send_multiple(&self, addr: &str, values: Vec<OscValue>) -> crate::Result<()> {
        self.send_message(addr, values)
    }
}

// Pre-configured OSC setups for common applications
pub struct OscPresets;

impl OscPresets {
    // TouchOSC layout mapping
    pub fn setup_touchosc_layout(server: &mut OscServer) {
        // Common TouchOSC control mappings
        server.register_pattern_handler("/1/fader1".to_string(), |msg| {
            println!("Master volume: {:?}", msg.args);
        });
        
        server.register_pattern_handler("/1/xy1".to_string(), |msg| {
            if msg.args.len() >= 2 {
                println!("XY Pad: X={:?}, Y={:?}", msg.args[0], msg.args[1]);
            }
        });
        
        server.register_pattern_handler("/1/toggle1".to_string(), |msg| {
            println!("Toggle button: {:?}", msg.args);
        });
    }
    
    // Ableton Live integration
    pub fn setup_ableton_live(client: &OscClient) -> crate::Result<()> {
        // Example: Send tempo change to Ableton
        client.send_message("/live/song/set/tempo", vec![OscValue::Float(120.0)])?;
        
        // Start/stop playback
        client.send_message("/live/song/start_playing", vec![])?;
        
        Ok(())
    }
    
    // Max/MSP integration
    pub fn setup_max_msp(server: &mut OscServer, _client: &OscClient) {
        // Register handlers for Max/MSP messages
        server.register_pattern_handler("/synth/freq".to_string(), |msg| {
            if let Some(OscType::Float(freq)) = msg.args.first() {
                println!("Synth frequency: {} Hz", freq);
            }
        });
        
        server.register_pattern_handler("/effect/reverb".to_string(), |msg| {
            if let Some(OscType::Float(amount)) = msg.args.first() {
                println!("Reverb amount: {}", amount);
            }
        });
        
        // Send control data to Max/MSP
        // client.send_float("/control/volume", 0.8).ok();
    }
    
    // Generic controller mapping for creative applications
    pub fn setup_creative_controls(server: &mut OscServer) -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        
        // Map common creative control addresses
        mappings.insert("/creative/brush/size".to_string(), "brush_size".to_string());
        mappings.insert("/creative/brush/opacity".to_string(), "brush_opacity".to_string());
        mappings.insert("/creative/color/hue".to_string(), "color_hue".to_string());
        mappings.insert("/creative/color/saturation".to_string(), "color_saturation".to_string());
        mappings.insert("/creative/color/brightness".to_string(), "color_brightness".to_string());
        mappings.insert("/creative/audio/volume".to_string(), "audio_volume".to_string());
        mappings.insert("/creative/audio/frequency".to_string(), "audio_frequency".to_string());
        mappings.insert("/creative/effect/amount".to_string(), "effect_amount".to_string());
        mappings.insert("/creative/timeline/position".to_string(), "timeline_position".to_string());
        mappings.insert("/creative/timeline/speed".to_string(), "timeline_speed".to_string());
        
        // Register handlers for all mappings
        for address in mappings.keys() {
            let addr_clone = address.clone();
            server.register_pattern_handler(address.clone(), move |msg| {
                if let Some(arg) = msg.args.first() {
                    println!("Creative control {}: {:?}", addr_clone, arg);
                }
            });
        }
        
        mappings
    }
}

// Utility functions for OSC-based creative control
pub fn osc_to_frequency(osc_value: f32, min_freq: f32, max_freq: f32) -> f32 {
    min_freq + osc_value * (max_freq - min_freq)
}

pub fn osc_to_color_hue(osc_value: f32) -> f32 {
    osc_value * 360.0 // 0-1 to 0-360 degrees
}

pub fn osc_xy_to_coordinates(x_val: f32, y_val: f32, width: f32, height: f32) -> (f32, f32) {
    (x_val * width, y_val * height)
}

pub fn multi_osc_blend(values: &[(String, f32)], blend_mode: OscBlendMode) -> f32 {
    match blend_mode {
        OscBlendMode::Average => {
            values.iter().map(|(_, v)| v).sum::<f32>() / values.len() as f32
        }
        OscBlendMode::Maximum => {
            values.iter().map(|(_, v)| *v).fold(0.0, f32::max)
        }
        OscBlendMode::Minimum => {
            values.iter().map(|(_, v)| *v).fold(1.0, f32::min)
        }
        OscBlendMode::Multiply => {
            values.iter().map(|(_, v)| v).product()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OscBlendMode {
    Average,
    Maximum,
    Minimum,
    Multiply,
}

impl Default for OscServer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OscClient {
    fn default() -> Self {
        Self::new()
    }
}