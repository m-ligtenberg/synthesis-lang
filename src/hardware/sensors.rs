use serialport::{SerialPort, SerialPortInfo};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct SensorData {
    pub sensor_id: String,
    pub value: f32,
    pub timestamp: Instant,
    pub data_type: SensorDataType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SensorDataType {
    Analog,      // 0.0 to 1.0
    Digital,     // 0.0 or 1.0
    Temperature, // Celsius
    Humidity,    // Percentage
    Pressure,    // hPa
    Acceleration, // g-force
    Gyroscope,   // degrees/second
    Distance,    // centimeters
    Light,       // lux
    Sound,       // dB
    Custom(String),
}

pub struct ArduinoManager {
    connections: HashMap<String, Box<dyn SerialPort>>,
    sensor_data: Arc<Mutex<HashMap<String, SensorData>>>,
    data_parsers: HashMap<String, Box<dyn Fn(&str) -> Option<Vec<SensorData>> + Send>>,
}

impl ArduinoManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            sensor_data: Arc::new(Mutex::new(HashMap::new())),
            data_parsers: HashMap::new(),
        }
    }
    
    pub fn list_ports() -> crate::Result<Vec<SerialPortInfo>> {
        Ok(serialport::available_ports()?)
    }
    
    pub fn connect(&mut self, port_name: String, baud_rate: u32) -> crate::Result<()> {
        let port = serialport::new(&port_name, baud_rate)
            .timeout(Duration::from_millis(100))
            .open()?;
        
        self.connections.insert(port_name, port);
        Ok(())
    }
    
    pub fn disconnect(&mut self, port_name: &str) {
        self.connections.remove(port_name);
    }
    
    pub fn send_command(&mut self, port_name: &str, command: &str) -> crate::Result<()> {
        if let Some(port) = self.connections.get_mut(port_name) {
            port.write_all(command.as_bytes())?;
            port.write_all(b"\n")?;
        } else {
            return Err(crate::errors::synthesis_error(crate::errors::ErrorKind::AudioDeviceError, format!("Serial port '{}' not connected", port_name)));
        }
        Ok(())
    }
    
    pub fn read_data(&mut self, port_name: &str) -> crate::Result<Vec<SensorData>> {
        if let Some(port) = self.connections.get_mut(port_name) {
            let mut buffer = [0u8; 1024];
            match port.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        let data_str = String::from_utf8_lossy(&buffer[..bytes_read]);
                        
                        // Try custom parser first
                        if let Some(parser) = self.data_parsers.get(port_name) {
                            if let Some(sensor_data) = parser(&data_str) {
                                // Update stored sensor data
                                let mut data_map = self.sensor_data.lock().unwrap();
                                for sensor in &sensor_data {
                                    data_map.insert(sensor.sensor_id.clone(), sensor.clone());
                                }
                                return Ok(sensor_data);
                            }
                        }
                        
                        // Default parser (expects CSV format: sensor_id,value,type)
                        Ok(self.parse_default_format(&data_str))
                    } else {
                        Ok(Vec::new())
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(Vec::new()),
                Err(e) => Err(crate::errors::synthesis_error(crate::errors::ErrorKind::AudioDeviceError, format!("Serial port read error: {}", e))),
            }
        } else {
            Err(crate::errors::synthesis_error(crate::errors::ErrorKind::AudioDeviceError, format!("Serial port '{}' not connected", port_name)))
        }
    }
    
    pub fn set_data_parser<F>(&mut self, port_name: String, parser: F)
    where
        F: Fn(&str) -> Option<Vec<SensorData>> + Send + 'static,
    {
        self.data_parsers.insert(port_name, Box::new(parser));
    }
    
    fn parse_default_format(&self, data_str: &str) -> Vec<SensorData> {
        let mut results = Vec::new();
        
        for line in data_str.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                if let Ok(value) = parts[1].parse::<f32>() {
                    let data_type = match parts[2] {
                        "analog" => SensorDataType::Analog,
                        "digital" => SensorDataType::Digital,
                        "temperature" => SensorDataType::Temperature,
                        "humidity" => SensorDataType::Humidity,
                        "pressure" => SensorDataType::Pressure,
                        "acceleration" => SensorDataType::Acceleration,
                        "gyroscope" => SensorDataType::Gyroscope,
                        "distance" => SensorDataType::Distance,
                        "light" => SensorDataType::Light,
                        "sound" => SensorDataType::Sound,
                        custom => SensorDataType::Custom(custom.to_string()),
                    };
                    
                    results.push(SensorData {
                        sensor_id: parts[0].to_string(),
                        value,
                        timestamp: Instant::now(),
                        data_type,
                    });
                }
            }
        }
        
        results
    }
    
    pub fn get_sensor_value(&self, sensor_id: &str) -> Option<f32> {
        let data_map = self.sensor_data.lock().unwrap();
        data_map.get(sensor_id).map(|data| data.value)
    }
    
    pub fn get_sensor_data(&self, sensor_id: &str) -> Option<SensorData> {
        let data_map = self.sensor_data.lock().unwrap();
        data_map.get(sensor_id).cloned()
    }
    
    pub fn get_all_sensor_data(&self) -> HashMap<String, SensorData> {
        let data_map = self.sensor_data.lock().unwrap();
        data_map.clone()
    }
    
    // Convenience methods for common sensor types
    pub fn get_analog_value(&self, sensor_id: &str, min_range: f32, max_range: f32) -> Option<f32> {
        self.get_sensor_value(sensor_id)
            .map(|raw_value| min_range + raw_value * (max_range - min_range))
    }
    
    pub fn is_digital_active(&self, sensor_id: &str) -> bool {
        self.get_sensor_value(sensor_id).unwrap_or(0.0) > 0.5
    }
    
    pub fn get_scaled_value(&self, sensor_id: &str, scale: f32, offset: f32) -> Option<f32> {
        self.get_sensor_value(sensor_id)
            .map(|value| value * scale + offset)
    }
}

// Predefined sensor configurations for common Arduino setups
pub struct SensorPresets;

impl SensorPresets {
    // Basic analog sensors (potentiometers, light sensors, etc.)
    pub fn setup_basic_analog(arduino: &mut ArduinoManager, port_name: String) {
        arduino.set_data_parser(port_name, |data| {
            // Expects format: "A0:512,A1:723,A2:234"
            let mut results = Vec::new();
            
            for pair in data.split(',') {
                let parts: Vec<&str> = pair.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(raw_value) = parts[1].parse::<i32>() {
                        // Convert 0-1023 to 0.0-1.0
                        let normalized = (raw_value as f32) / 1023.0;
                        results.push(SensorData {
                            sensor_id: parts[0].to_string(),
                            value: normalized,
                            timestamp: Instant::now(),
                            data_type: SensorDataType::Analog,
                        });
                    }
                }
            }
            
            if results.is_empty() { None } else { Some(results) }
        });
    }
    
    // Accelerometer/gyroscope (MPU6050 format)
    pub fn setup_mpu6050(arduino: &mut ArduinoManager, port_name: String) {
        arduino.set_data_parser(port_name, |data| {
            // Expects format: "ACC:1.23,-0.45,0.67,GYRO:12.3,-45.6,67.8"
            let mut results = Vec::new();
            
            for section in data.split(',') {
                if section.starts_with("ACC:") {
                    let coords = &section[4..];
                    let values: Vec<&str> = coords.split(',').collect();
                    if values.len() >= 3 {
                        for (i, axis) in ["acc_x", "acc_y", "acc_z"].iter().enumerate() {
                            if let Ok(value) = values[i].parse::<f32>() {
                                results.push(SensorData {
                                    sensor_id: axis.to_string(),
                                    value,
                                    timestamp: Instant::now(),
                                    data_type: SensorDataType::Acceleration,
                                });
                            }
                        }
                    }
                } else if section.starts_with("GYRO:") {
                    let coords = &section[5..];
                    let values: Vec<&str> = coords.split(',').collect();
                    if values.len() >= 3 {
                        for (i, axis) in ["gyro_x", "gyro_y", "gyro_z"].iter().enumerate() {
                            if let Ok(value) = values[i].parse::<f32>() {
                                results.push(SensorData {
                                    sensor_id: axis.to_string(),
                                    value,
                                    timestamp: Instant::now(),
                                    data_type: SensorDataType::Gyroscope,
                                });
                            }
                        }
                    }
                }
            }
            
            if results.is_empty() { None } else { Some(results) }
        });
    }
    
    // Environmental sensors (DHT22 temperature/humidity)
    pub fn setup_environmental(arduino: &mut ArduinoManager, port_name: String) {
        arduino.set_data_parser(port_name, |data| {
            // Expects format: "TEMP:23.5,HUMID:65.2,PRESS:1013.25"
            let mut results = Vec::new();
            
            for pair in data.split(',') {
                let parts: Vec<&str> = pair.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(value) = parts[1].parse::<f32>() {
                        let (sensor_id, data_type) = match parts[0] {
                            "TEMP" => ("temperature", SensorDataType::Temperature),
                            "HUMID" => ("humidity", SensorDataType::Humidity),
                            "PRESS" => ("pressure", SensorDataType::Pressure),
                            "LIGHT" => ("light", SensorDataType::Light),
                            _ => (parts[0], SensorDataType::Custom(parts[0].to_string())),
                        };
                        
                        results.push(SensorData {
                            sensor_id: sensor_id.to_string(),
                            value,
                            timestamp: Instant::now(),
                            data_type,
                        });
                    }
                }
            }
            
            if results.is_empty() { None } else { Some(results) }
        });
    }
}

// Utility functions for creative sensor mappings
pub fn sensor_to_frequency(sensor_value: f32, min_freq: f32, max_freq: f32, curve: f32) -> f32 {
    let curved_value = if curve == 1.0 {
        sensor_value
    } else {
        sensor_value.powf(curve)
    };
    
    min_freq + curved_value * (max_freq - min_freq)
}

pub fn sensor_to_midi_note(sensor_value: f32, min_note: u8, max_note: u8) -> u8 {
    let note_range = max_note - min_note;
    let scaled = sensor_value * note_range as f32;
    (min_note as f32 + scaled).round().clamp(0.0, 127.0) as u8
}

pub fn multi_sensor_blend(sensors: &[(String, f32)], weights: &[f32]) -> f32 {
    let mut total_value = 0.0;
    let mut total_weight = 0.0;
    
    for (i, (_, value)) in sensors.iter().enumerate() {
        let weight = weights.get(i).copied().unwrap_or(1.0);
        total_value += value * weight;
        total_weight += weight;
    }
    
    if total_weight > 0.0 {
        total_value / total_weight
    } else {
        0.0
    }
}

impl Default for ArduinoManager {
    fn default() -> Self {
        Self::new()
    }
}