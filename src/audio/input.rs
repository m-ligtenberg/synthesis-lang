use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct AudioInput {
    stream: Option<cpal::Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    config: cpal::StreamConfig,
}

impl AudioInput {
    pub fn new() -> crate::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

        let config = device.default_input_config()?;
        let config = config.into();
        
        let buffer = Arc::new(Mutex::new(Vec::new()));

        Ok(Self {
            stream: None,
            buffer,
            config,
        })
    }

    pub fn start_capture(&mut self) -> crate::Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

        let buffer = Arc::clone(&self.buffer);
        
        let stream = device.build_input_stream(
            &self.config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = buffer.lock().unwrap();
                buffer.extend_from_slice(data);
                
                if buffer.len() > 44100 {
                    let excess = buffer.len() - 44100;
                    buffer.drain(0..excess);
                }
            },
            |err| {
                eprintln!("Audio input error: {}", err);
            },
            None,
        )?;

        stream.play()?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn get_samples(&self, count: usize) -> Vec<f32> {
        let buffer = self.buffer.lock().unwrap();
        if buffer.len() >= count {
            buffer[buffer.len() - count..].to_vec()
        } else {
            let mut result = vec![0.0; count];
            let available = buffer.len();
            if available > 0 {
                result[count - available..].copy_from_slice(&buffer);
            }
            result
        }
    }

    pub fn stop_capture(&mut self) {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }
}

impl Drop for AudioInput {
    fn drop(&mut self) {
        self.stop_capture();
    }
}