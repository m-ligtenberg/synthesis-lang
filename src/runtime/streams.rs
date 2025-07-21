use crate::runtime::types::{DataType, Stream, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct StreamManager {
    streams: HashMap<String, Arc<Mutex<StreamData>>>,
    connections: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct StreamData {
    pub stream: Stream,
    pub buffer: Vec<f32>,
    pub position: usize,
    pub is_active: bool,
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
            connections: HashMap::new(),
        }
    }
    
    pub fn create_stream(&mut self, name: String, data_type: DataType, sample_rate: Option<f32>) -> crate::Result<()> {
        let stream = Stream {
            name: name.clone(),
            data_type,
            sample_rate,
        };
        
        let stream_data = StreamData {
            stream,
            buffer: Vec::new(),
            position: 0,
            is_active: false,
        };
        
        self.streams.insert(name, Arc::new(Mutex::new(stream_data)));
        Ok(())
    }
    
    pub fn connect(&mut self, source: String, destination: String) -> crate::Result<()> {
        if !self.streams.contains_key(&source) {
            return Err(anyhow::anyhow!("Source stream '{}' does not exist", source));
        }
        
        if !self.streams.contains_key(&destination) {
            return Err(anyhow::anyhow!("Destination stream '{}' does not exist", destination));
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
            Err(anyhow::anyhow!("Stream '{}' not found", name))
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
            Err(anyhow::anyhow!("Stream '{}' not found", name))
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
            Value::Stream(stream_data.stream.clone())
        } else {
            Value::Null
        }
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}