use std::sync::{Arc, Mutex};
use std::time::Instant;

#[cfg(feature = "opencv")]
use opencv::{
    core::{Mat, Vector},
    imgproc,
    prelude::*,
    videoio::{VideoCapture, CAP_ANY},
};

#[derive(Debug, Clone)]
pub struct WebcamFrame {
    pub width: u32,
    pub height: u32,
    pub timestamp: Instant,
    pub data: Vec<u8>, // RGB data
}

#[derive(Debug, Clone)]
pub struct MotionData {
    pub motion_amount: f32,
    pub motion_center: (f32, f32),
    pub optical_flow: Vec<(f32, f32)>,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct ColorAnalysis {
    pub dominant_color: [f32; 3],
    pub average_brightness: f32,
    pub color_histogram: Vec<f32>,
    pub timestamp: Instant,
}

pub struct WebcamManager {
    #[cfg(feature = "opencv")]
    capture: Option<VideoCapture>,
    current_frame: Arc<Mutex<Option<WebcamFrame>>>,
    #[cfg(feature = "opencv")]
    previous_frame: Option<Mat>,
    motion_threshold: f32,
    is_capturing: bool,
    frame_skip: u32,
    frame_counter: u32,
}

impl WebcamManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "opencv")]
            capture: None,
            current_frame: Arc::new(Mutex::new(None)),
            #[cfg(feature = "opencv")]
            previous_frame: None,
            motion_threshold: 30.0,
            is_capturing: false,
            frame_skip: 2, // Process every 3rd frame for performance
            frame_counter: 0,
        }
    }
    
    #[cfg(feature = "opencv")]
    pub fn start_capture(&mut self, device_index: i32) -> crate::Result<()> {
        let mut capture = VideoCapture::new(device_index, CAP_ANY)?;
        
        if !capture.is_opened()? {
            return Err(anyhow::anyhow!("Failed to open webcam device {}", device_index));
        }
        
        // Set capture properties
        capture.set(opencv::videoio::CAP_PROP_FRAME_WIDTH, 640.0)?;
        capture.set(opencv::videoio::CAP_PROP_FRAME_HEIGHT, 480.0)?;
        capture.set(opencv::videoio::CAP_PROP_FPS, 30.0)?;
        
        self.capture = Some(capture);
        self.is_capturing = true;
        
        Ok(())
    }
    
    #[cfg(not(feature = "opencv"))]
    pub fn start_capture(&mut self, _device_index: i32) -> crate::Result<()> {
        // Stub implementation when OpenCV is not available
        self.is_capturing = true;
        Ok(())
    }
    
    pub fn stop_capture(&mut self) {
        self.is_capturing = false;
        #[cfg(feature = "opencv")]
        {
            self.capture = None;
            self.previous_frame = None;
        }
    }
    
    #[cfg(feature = "opencv")]
    pub fn update(&mut self) -> crate::Result<()> {
        if !self.is_capturing {
            return Ok(());
        }
        
        self.frame_counter += 1;
        if self.frame_counter % (self.frame_skip + 1) != 0 {
            return Ok(()); // Skip this frame
        }
        
        if let Some(ref mut capture) = self.capture {
            let mut frame = Mat::default();
            if capture.read(&mut frame)? && !frame.empty() {
                // Convert BGR to RGB
                let mut rgb_frame = Mat::default();
                imgproc::cvt_color(&frame, &mut rgb_frame, imgproc::COLOR_BGR2RGB, 0)?;
                
                // Extract frame data
                let width = rgb_frame.cols() as u32;
                let height = rgb_frame.rows() as u32;
                let data = rgb_frame.data_bytes()?.to_vec();
                
                let webcam_frame = WebcamFrame {
                    width,
                    height,
                    timestamp: Instant::now(),
                    data,
                };
                
                // Update current frame
                {
                    let mut current = self.current_frame.lock().unwrap();
                    *current = Some(webcam_frame);
                }
                
                // Store for motion detection
                self.previous_frame = Some(frame);
            }
        }
        
        Ok(())
    }
    
    #[cfg(not(feature = "opencv"))]
    pub fn update(&mut self) -> crate::Result<()> {
        // Stub implementation
        Ok(())
    }
    
    pub fn get_current_frame(&self) -> Option<WebcamFrame> {
        let frame = self.current_frame.lock().unwrap();
        frame.clone()
    }
    
    #[cfg(feature = "opencv")]
    pub fn analyze_motion(&self) -> Option<MotionData> {
        if let (Some(ref capture), Some(ref prev_frame)) = (&self.capture, &self.previous_frame) {
            let mut current_frame = Mat::default();
            if capture.read(&mut current_frame).unwrap_or(false) && !current_frame.empty() {
                // Convert to grayscale
                let mut gray_current = Mat::default();
                let mut gray_previous = Mat::default();
                
                imgproc::cvt_color(&current_frame, &mut gray_current, imgproc::COLOR_BGR2GRAY, 0).ok()?;
                imgproc::cvt_color(prev_frame, &mut gray_previous, imgproc::COLOR_BGR2GRAY, 0).ok()?;
                
                // Calculate frame difference
                let mut diff = Mat::default();
                opencv::core::absdiff(&gray_current, &gray_previous, &mut diff).ok()?;
                
                // Threshold the difference
                let mut thresh = Mat::default();
                imgproc::threshold(&diff, &mut thresh, self.motion_threshold as f64, 255.0, imgproc::THRESH_BINARY).ok()?;
                
                // Calculate motion amount (percentage of pixels that changed)
                let motion_pixels = opencv::core::count_non_zero(&thresh).unwrap_or(0);
                let total_pixels = thresh.rows() * thresh.cols();
                let motion_amount = motion_pixels as f32 / total_pixels as f32;
                
                // Calculate motion center (centroid of motion)
                let moments = imgproc::moments(&thresh, false).ok()?;
                let motion_center = if moments.m00 > 0.0 {
                    (
                        (moments.m10 / moments.m00) as f32 / current_frame.cols() as f32,
                        (moments.m01 / moments.m00) as f32 / current_frame.rows() as f32,
                    )
                } else {
                    (0.5, 0.5)
                };
                
                return Some(MotionData {
                    motion_amount,
                    motion_center,
                    optical_flow: Vec::new(), // Could be implemented with Lucas-Kanade
                    timestamp: Instant::now(),
                });
            }
        }
        
        None
    }
    
    #[cfg(not(feature = "opencv"))]
    pub fn analyze_motion(&self) -> Option<MotionData> {
        // Stub implementation
        None
    }
    
    pub fn analyze_color(&self) -> Option<ColorAnalysis> {
        if let Some(frame) = self.get_current_frame() {
            let mut r_total = 0u64;
            let mut g_total = 0u64;
            let mut b_total = 0u64;
            let mut brightness_total = 0u64;
            let pixel_count = (frame.width * frame.height) as usize;
            
            // Color histogram (simplified to 8 bins per channel)
            let mut histogram = vec![0; 512]; // 8*8*8
            
            for i in (0..frame.data.len()).step_by(3) {
                if i + 2 < frame.data.len() {
                    let r = frame.data[i] as u64;
                    let g = frame.data[i + 1] as u64;
                    let b = frame.data[i + 2] as u64;
                    
                    r_total += r;
                    g_total += g;
                    b_total += b;
                    
                    // Calculate brightness (luminance)
                    let brightness = ((r * 299 + g * 587 + b * 114) / 1000) as u64;
                    brightness_total += brightness;
                    
                    // Add to histogram
                    let r_bin = ((r * 7) / 255) as usize;
                    let g_bin = ((g * 7) / 255) as usize;
                    let b_bin = ((b * 7) / 255) as usize;
                    let bin_index = r_bin * 64 + g_bin * 8 + b_bin;
                    if bin_index < histogram.len() {
                        histogram[bin_index] += 1;
                    }
                }
            }
            
            // Calculate averages
            let dominant_color = [
                (r_total as f32) / (pixel_count as f32 * 255.0),
                (g_total as f32) / (pixel_count as f32 * 255.0),
                (b_total as f32) / (pixel_count as f32 * 255.0),
            ];
            
            let average_brightness = (brightness_total as f32) / (pixel_count as f32 * 255.0);
            
            // Normalize histogram
            let max_count = *histogram.iter().max().unwrap_or(&1) as f32;
            let color_histogram: Vec<f32> = histogram
                .iter()
                .map(|&count| count as f32 / max_count)
                .collect();
            
            return Some(ColorAnalysis {
                dominant_color,
                average_brightness,
                color_histogram,
                timestamp: Instant::now(),
            });
        }
        
        None
    }
    
    pub fn set_motion_threshold(&mut self, threshold: f32) {
        self.motion_threshold = threshold.clamp(0.0, 255.0);
    }
    
    pub fn set_frame_skip(&mut self, skip: u32) {
        self.frame_skip = skip;
    }
    
    pub fn is_capturing(&self) -> bool {
        self.is_capturing
    }
}

// Utility functions for webcam-based creative applications
pub fn brightness_to_frequency(brightness: f32, min_freq: f32, max_freq: f32) -> f32 {
    min_freq + brightness * (max_freq - min_freq)
}

pub fn color_to_hue(color: [f32; 3]) -> f32 {
    let r = color[0];
    let g = color[1];
    let b = color[2];
    
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    
    if delta == 0.0 {
        return 0.0;
    }
    
    let hue = if max == r {
        ((g - b) / delta) % 6.0
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    
    (hue * 60.0).max(0.0) % 360.0
}

pub fn motion_to_amplitude(motion_amount: f32, sensitivity: f32) -> f32 {
    (motion_amount * sensitivity).clamp(0.0, 1.0)
}

impl Default for WebcamManager {
    fn default() -> Self {
        Self::new()
    }
}