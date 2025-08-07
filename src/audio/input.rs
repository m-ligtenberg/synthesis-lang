use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use crate::runtime::realtime_buffer::{RealtimeCircularBuffer, BufferError};

pub struct AudioInput {
    stream: Option<cpal::Stream>,
    buffer: Arc<RealtimeCircularBuffer>,
    config: cpal::StreamConfig,
}

impl AudioInput {
    pub fn new() -> crate::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| crate::errors::synthesis_error(crate::errors::ErrorKind::AudioDeviceError, "No input device available"))?;

        let config = device.default_input_config()?;
        let config = config.into();
        
        let buffer = Arc::new(RealtimeCircularBuffer::new(8192)
            .map_err(|_| crate::errors::synthesis_error(crate::errors::ErrorKind::AudioDeviceError, "Failed to create audio buffer"))?);

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
            .ok_or_else(|| crate::errors::synthesis_error(crate::errors::ErrorKind::AudioDeviceError, "No input device available"))?;

        let buffer = Arc::clone(&self.buffer);
        
        let stream = device.build_input_stream(
            &self.config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Real-time safe: no locks, no allocations, bounded time
                for &sample in data {
                    // Silently drop samples if buffer is full (prevents blocking)
                    let _ = buffer.write_single(sample);
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
        let mut result = Vec::with_capacity(count);
        
        // Read available samples (non-blocking)
        for _ in 0..count {
            match self.buffer.read_single() {
                Ok(sample) => result.push(sample),
                Err(BufferError::BufferEmpty) => result.push(0.0), // Silence for missing samples
                Err(_) => result.push(0.0),
            }
        }
        
        result
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