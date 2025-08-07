pub struct FFTAnalyzer {
    fft_size: usize,
    window: Vec<f32>,
    magnitude_buffer: Vec<f32>,
}

impl FFTAnalyzer {
    pub fn new(fft_size: usize) -> Self {
        let window = (0..fft_size)
            .map(|i| {
                let t = i as f32 / (fft_size - 1) as f32;
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * t).cos())
            })
            .collect();

        Self {
            fft_size,
            window,
            magnitude_buffer: vec![0.0; fft_size / 2],
        }
    }

    pub fn analyze(&mut self, samples: &[f32], bands: usize) -> Vec<f32> {
        if samples.len() < self.fft_size {
            return vec![0.0; bands];
        }

        let windowed: Vec<f32> = samples[..self.fft_size]
            .iter()
            .zip(&self.window)
            .map(|(sample, window)| sample * window)
            .collect();

        let mut complex: Vec<num_complex::Complex<f32>> = windowed
            .into_iter()
            .map(|x| num_complex::Complex::new(x, 0.0))
            .collect();

        self.simple_fft(&mut complex);

        for (i, &c) in complex[..self.fft_size / 2].iter().enumerate() {
            self.magnitude_buffer[i] = c.norm();
        }

        self.bin_to_bands(bands)
    }

    fn simple_fft(&self, data: &mut [num_complex::Complex<f32>]) {
        let n = data.len();
        if n <= 1 {
            return;
        }

        let mut j = 0;
        for i in 1..n {
            let mut bit = n >> 1;
            while j & bit != 0 {
                j ^= bit;
                bit >>= 1;
            }
            j ^= bit;

            if i < j {
                data.swap(i, j);
            }
        }

        let mut length = 2;
        while length <= n {
            let angle = -2.0 * std::f32::consts::PI / length as f32;
            let wlen = num_complex::Complex::new(angle.cos(), angle.sin());

            for i in (0..n).step_by(length) {
                let mut w = num_complex::Complex::new(1.0, 0.0);
                for j in 0..length / 2 {
                    let u = data[i + j];
                    let v = data[i + j + length / 2] * w;
                    data[i + j] = u + v;
                    data[i + j + length / 2] = u - v;
                    w *= wlen;
                }
            }
            length <<= 1;
        }
    }

    fn bin_to_bands(&self, bands: usize) -> Vec<f32> {
        if bands == 0 {
            return Vec::new();
        }

        let mut result = vec![0.0; bands];
        let bins_per_band = self.magnitude_buffer.len() / bands;

        for (band_idx, band) in result.iter_mut().enumerate() {
            let start_bin = band_idx * bins_per_band;
            let end_bin = ((band_idx + 1) * bins_per_band).min(self.magnitude_buffer.len());

            let sum: f32 = self.magnitude_buffer[start_bin..end_bin].iter().sum();
            *band = sum / (end_bin - start_bin) as f32;
        }

        result
    }
}

pub struct BeatDetector {
    energy_buffer: Vec<f32>,  // Pre-allocated fixed-size buffer
    buffer_index: usize,      // Circular index
    buffer_size: usize,
    buffer_filled: bool,      // Track if we've filled the buffer once
    threshold_multiplier: f32,
    last_beat_time: std::time::Instant,
    min_beat_interval: std::time::Duration,
}

impl BeatDetector {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            energy_buffer: vec![0.0; buffer_size],  // Pre-allocate with zeros
            buffer_index: 0,
            buffer_size,
            buffer_filled: false,
            threshold_multiplier: 1.3,
            last_beat_time: std::time::Instant::now(),
            min_beat_interval: std::time::Duration::from_millis(300),
        }
    }

    pub fn detect_beat(&mut self, samples: &[f32]) -> bool {
        let energy = self.calculate_energy(samples);
        
        // Real-time safe circular buffer update (no allocations, O(1) time)
        self.energy_buffer[self.buffer_index] = energy;
        self.buffer_index = (self.buffer_index + 1) % self.buffer_size;
        
        // Mark buffer as filled after first complete cycle
        if self.buffer_index == 0 {
            self.buffer_filled = true;
        }

        // Only detect beats after buffer is filled
        if !self.buffer_filled {
            return false;
        }

        // Calculate average (O(n) but bounded and predictable)
        let average_energy = self.energy_buffer.iter().sum::<f32>() / self.buffer_size as f32;
        let threshold = average_energy * self.threshold_multiplier;

        let now = std::time::Instant::now();
        let time_since_last_beat = now.duration_since(self.last_beat_time);

        if energy > threshold && time_since_last_beat > self.min_beat_interval {
            self.last_beat_time = now;
            true
        } else {
            false
        }
    }

    fn calculate_energy(&self, samples: &[f32]) -> f32 {
        samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32
    }
}