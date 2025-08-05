use crate::graphics::{primitives::Color, blend_modes::*};
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct NoiseGenerator {
    seed: u64,
    octaves: u32,
    frequency: f32,
    amplitude: f32,
    lacunarity: f32,
    persistence: f32,
}

impl NoiseGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            octaves: 4,
            frequency: 0.01,
            amplitude: 1.0,
            lacunarity: 2.0,
            persistence: 0.5,
        }
    }
    
    pub fn with_params(mut self, octaves: u32, frequency: f32, amplitude: f32) -> Self {
        self.octaves = octaves;
        self.frequency = frequency;
        self.amplitude = amplitude;
        self
    }
    
    pub fn noise_2d(&self, x: f32, y: f32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = self.amplitude;
        let mut frequency = self.frequency;
        
        for _ in 0..self.octaves {
            value += self.simplex_noise(x * frequency, y * frequency) * amplitude;
            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }
        
        value
    }
    
    fn simplex_noise(&self, x: f32, y: f32) -> f32 {
        // Simplified 2D simplex noise implementation
        let f2 = 0.5 * (3.0_f32.sqrt() - 1.0);
        let g2 = (3.0 - 3.0_f32.sqrt()) / 6.0;
        
        let s = (x + y) * f2;
        let i = (x + s).floor() as i32;
        let j = (y + s).floor() as i32;
        
        let t = ((i + j) as f32) * g2;
        let x0 = x - (i as f32) + t;
        let y0 = y - (j as f32) + t;
        
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
        
        let x1 = x0 - i1 as f32 + g2;
        let y1 = y0 - j1 as f32 + g2;
        let x2 = x0 - 1.0 + 2.0 * g2;
        let y2 = y0 - 1.0 + 2.0 * g2;
        
        let ii = i & 255;
        let jj = j & 255;
        
        let gi0 = self.perm(ii + self.perm(jj)) % 12;
        let gi1 = self.perm(ii + i1 + self.perm(jj + j1)) % 12;
        let gi2 = self.perm(ii + 1 + self.perm(jj + 1)) % 12;
        
        let mut n0 = 0.0;
        let mut n1 = 0.0;
        let mut n2 = 0.0;
        
        let t0 = 0.5 - x0 * x0 - y0 * y0;
        if t0 >= 0.0 {
            n0 = t0 * t0 * t0 * t0 * self.dot_grad(gi0, x0, y0);
        }
        
        let t1 = 0.5 - x1 * x1 - y1 * y1;
        if t1 >= 0.0 {
            n1 = t1 * t1 * t1 * t1 * self.dot_grad(gi1, x1, y1);
        }
        
        let t2 = 0.5 - x2 * x2 - y2 * y2;
        if t2 >= 0.0 {
            n2 = t2 * t2 * t2 * t2 * self.dot_grad(gi2, x2, y2);
        }
        
        70.0 * (n0 + n1 + n2)
    }
    
    fn perm(&self, i: i32) -> i32 {
        // Simple permutation function based on seed
        ((i as u64 * 1664525 + 1013904223 + self.seed) % 256) as i32
    }
    
    fn dot_grad(&self, gi: i32, x: f32, y: f32) -> f32 {
        let grad = [
            [1.0, 1.0], [-1.0, 1.0], [1.0, -1.0], [-1.0, -1.0],
            [1.0, 0.0], [-1.0, 0.0], [1.0, 0.0], [-1.0, 0.0],
            [0.0, 1.0], [0.0, -1.0], [0.0, 1.0], [0.0, -1.0]
        ];
        let g = grad[gi as usize % grad.len()];
        g[0] * x + g[1] * y
    }
}

#[derive(Debug, Clone)]
pub struct PlasmaEffect {
    pub time: f32,
    pub speed: f32,
    pub scale: f32,
    pub color_shift: f32,
    pub intensity: f32,
}

impl PlasmaEffect {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            speed: 1.0,
            scale: 0.01,
            color_shift: 0.0,
            intensity: 1.0,
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        self.time += dt * self.speed;
    }
    
    pub fn render_pixel(&self, x: f32, y: f32) -> Color {
        let v1 = (x * self.scale + self.time).sin();
        let v2 = (y * self.scale + self.time).cos();
        let v3 = ((x + y) * self.scale * 0.5 + self.time).sin();
        let v4 = ((x * x + y * y).sqrt() * self.scale + self.time).cos();
        
        let plasma = (v1 + v2 + v3 + v4) * 0.25 * self.intensity;
        
        let r = (plasma + self.color_shift).sin() * 0.5 + 0.5;
        let g = (plasma + self.color_shift + PI * 2.0 / 3.0).sin() * 0.5 + 0.5;
        let b = (plasma + self.color_shift + PI * 4.0 / 3.0).sin() * 0.5 + 0.5;
        
        Color::new(r, g, b, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct TunnelEffect {
    pub time: f32,
    pub speed: f32,
    pub twist: f32,
    pub depth: f32,
    pub texture_scale: f32,
}

impl TunnelEffect {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            speed: 1.0,
            twist: 1.0,
            depth: 1.0,
            texture_scale: 10.0,
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        self.time += dt * self.speed;
    }
    
    pub fn render_pixel(&self, x: f32, y: f32, center_x: f32, center_y: f32) -> Color {
        let dx = x - center_x;
        let dy = y - center_y;
        let distance = (dx * dx + dy * dy).sqrt().max(1.0);
        let angle = dy.atan2(dx);
        
        let u = angle / PI + self.time * 0.5 + self.twist / distance;
        let v = self.depth / distance + self.time;
        
        let pattern_u = (u * self.texture_scale).sin();
        let pattern_v = (v * self.texture_scale).cos();
        let pattern = (pattern_u + pattern_v) * 0.5;
        
        let intensity = (1.0 / distance * 100.0).min(1.0);
        let color_val = (pattern * intensity).max(0.0);
        
        Color::new(color_val, color_val * 0.8, color_val * 0.6, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct FeedbackEffect {
    pub previous_frame: Vec<Color>,
    pub width: u32,
    pub height: u32,
    pub feedback_amount: f32,
    pub zoom: f32,
    pub rotation: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub color_shift: Color,
}

impl FeedbackEffect {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            previous_frame: vec![Color::new(0.0, 0.0, 0.0, 1.0); (width * height) as usize],
            width,
            height,
            feedback_amount: 0.95,
            zoom: 1.01,
            rotation: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            color_shift: Color::new(1.0, 0.99, 0.98, 1.0),
        }
    }
    
    pub fn process_frame(&mut self, current_frame: &[Color]) -> Vec<Color> {
        let mut result = vec![Color::new(0.0, 0.0, 0.0, 1.0); (self.width * self.height) as usize];
        
        let center_x = self.width as f32 * 0.5;
        let center_y = self.height as f32 * 0.5;
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();
        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                
                // Apply current frame
                result[idx] = current_frame[idx];
                
                // Sample from previous frame with transformation
                let fx = x as f32 - center_x;
                let fy = y as f32 - center_y;
                
                // Apply zoom and rotation
                let tx = (fx * cos_rot - fy * sin_rot) / self.zoom + center_x + self.offset_x;
                let ty = (fx * sin_rot + fy * cos_rot) / self.zoom + center_y + self.offset_y;
                
                let sample = self.sample_previous_frame(tx, ty);
                
                // Apply color shift and feedback
                let shifted_sample = Color::new(
                    sample.r * self.color_shift.r,
                    sample.g * self.color_shift.g,
                    sample.b * self.color_shift.b,
                    sample.a,
                );
                
                // Blend with feedback
                let feedback_color = Color::new(
                    shifted_sample.r * self.feedback_amount,
                    shifted_sample.g * self.feedback_amount,
                    shifted_sample.b * self.feedback_amount,
                    shifted_sample.a,
                );
                
                result[idx] = BlendMode::Add.blend(result[idx], feedback_color);
            }
        }
        
        // Store current result as previous frame
        self.previous_frame = result.clone();
        
        result
    }
    
    fn sample_previous_frame(&self, x: f32, y: f32) -> Color {
        let ix = x as i32;
        let iy = y as i32;
        
        if ix >= 0 && ix < self.width as i32 - 1 && iy >= 0 && iy < self.height as i32 - 1 {
            // Bilinear interpolation
            let fx = x - ix as f32;
            let fy = y - iy as f32;
            
            let idx00 = (iy * self.width as i32 + ix) as usize;
            let idx01 = ((iy + 1) * self.width as i32 + ix) as usize;
            let idx10 = (iy * self.width as i32 + ix + 1) as usize;
            let idx11 = ((iy + 1) * self.width as i32 + ix + 1) as usize;
            
            let c00 = self.previous_frame[idx00];
            let c01 = self.previous_frame[idx01];
            let c10 = self.previous_frame[idx10];
            let c11 = self.previous_frame[idx11];
            
            let c0 = Color::new(
                c00.r * (1.0 - fx) + c10.r * fx,
                c00.g * (1.0 - fx) + c10.g * fx,
                c00.b * (1.0 - fx) + c10.b * fx,
                c00.a * (1.0 - fx) + c10.a * fx,
            );
            
            let c1 = Color::new(
                c01.r * (1.0 - fx) + c11.r * fx,
                c01.g * (1.0 - fx) + c11.g * fx,
                c01.b * (1.0 - fx) + c11.b * fx,
                c01.a * (1.0 - fx) + c11.a * fx,
            );
            
            Color::new(
                c0.r * (1.0 - fy) + c1.r * fy,
                c0.g * (1.0 - fy) + c1.g * fy,
                c0.b * (1.0 - fy) + c1.b * fy,
                c0.a * (1.0 - fy) + c1.a * fy,
            )
        } else {
            Color::new(0.0, 0.0, 0.0, 0.0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub max_particles: usize,
    pub emission_rate: f32,
    pub emission_timer: f32,
    pub gravity: (f32, f32),
    pub wind: (f32, f32),
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub size: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
}

impl ParticleSystem {
    pub fn new(max_particles: usize, emission_rate: f32) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            max_particles,
            emission_rate,
            emission_timer: 0.0,
            gravity: (0.0, -9.8),
            wind: (0.0, 0.0),
        }
    }
    
    pub fn update(&mut self, dt: f32, emitter_pos: (f32, f32)) {
        // Update existing particles
        self.particles.retain_mut(|particle| {
            particle.life -= dt;
            if particle.life <= 0.0 {
                return false;
            }
            
            // Apply forces
            particle.velocity.0 += (self.gravity.0 + self.wind.0) * dt;
            particle.velocity.1 += (self.gravity.1 + self.wind.1) * dt;
            
            // Update position
            particle.position.0 += particle.velocity.0 * dt;
            particle.position.1 += particle.velocity.1 * dt;
            
            // Update rotation
            particle.rotation += particle.angular_velocity * dt;
            
            // Fade out over time
            let life_ratio = particle.life / particle.max_life;
            particle.color.a = life_ratio;
            
            true
        });
        
        // Emit new particles
        self.emission_timer += dt;
        if self.emission_timer >= 1.0 / self.emission_rate && self.particles.len() < self.max_particles {
            self.emit_particle(emitter_pos);
            self.emission_timer = 0.0;
        }
    }
    
    fn emit_particle(&mut self, emitter_pos: (f32, f32)) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let angle = rng.gen::<f32>() * PI * 2.0;
        let speed = rng.gen_range(50.0..200.0);
        
        let particle = Particle {
            position: emitter_pos,
            velocity: (angle.cos() * speed, angle.sin() * speed),
            life: rng.gen_range(1.0..3.0),
            max_life: 2.0,
            color: Color::new(
                rng.gen_range(0.5..1.0),
                rng.gen_range(0.5..1.0),
                rng.gen_range(0.5..1.0),
                1.0,
            ),
            size: rng.gen_range(2.0..8.0),
            rotation: 0.0,
            angular_velocity: rng.gen_range(-5.0..5.0),
        };
        
        self.particles.push(particle);
    }
}

impl Default for PlasmaEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TunnelEffect {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BloomEffect {
    pub threshold: f32,
    pub intensity: f32,
    pub radius: f32,
    pub downsample_passes: u32,
}

impl BloomEffect {
    pub fn new() -> Self {
        Self {
            threshold: 0.8,
            intensity: 1.0,
            radius: 5.0,
            downsample_passes: 3,
        }
    }
    
    pub fn apply(&self, input: &[Color], width: u32, height: u32) -> Vec<Color> {
        // Extract bright pixels above threshold
        let bright_pixels: Vec<Color> = input.iter().map(|color| {
            let luminance = color.r * 0.299 + color.g * 0.587 + color.b * 0.114;
            if luminance > self.threshold {
                let multiplier = (luminance - self.threshold) / (1.0 - self.threshold);
                Color::new(
                    color.r * multiplier * self.intensity,
                    color.g * multiplier * self.intensity,
                    color.b * multiplier * self.intensity,
                    color.a,
                )
            } else {
                Color::new(0.0, 0.0, 0.0, 0.0)
            }
        }).collect();
        
        // Apply gaussian blur to bright pixels
        let blurred = self.gaussian_blur(&bright_pixels, width, height);
        
        // Combine with original
        input.iter().zip(blurred.iter()).map(|(original, bloom)| {
            Color::new(
                (original.r + bloom.r).min(1.0),
                (original.g + bloom.g).min(1.0),
                (original.b + bloom.b).min(1.0),
                original.a,
            )
        }).collect()
    }
    
    fn gaussian_blur(&self, input: &[Color], width: u32, height: u32) -> Vec<Color> {
        let kernel_size = (self.radius * 2.0) as usize + 1;
        let sigma = self.radius / 3.0;
        
        // Generate gaussian kernel
        let mut kernel = vec![0.0; kernel_size];
        let mut sum = 0.0;
        let center = kernel_size / 2;
        
        for i in 0..kernel_size {
            let x = (i as f32) - (center as f32);
            let weight = (-0.5 * x * x / (sigma * sigma)).exp();
            kernel[i] = weight;
            sum += weight;
        }
        
        // Normalize kernel
        for weight in &mut kernel {
            *weight /= sum;
        }
        
        // Horizontal pass
        let mut temp = vec![Color::new(0.0, 0.0, 0.0, 0.0); input.len()];
        for y in 0..height {
            for x in 0..width {
                let mut result = Color::new(0.0, 0.0, 0.0, 0.0);
                
                for i in 0..kernel_size {
                    let sample_x = (x as i32) + (i as i32) - (center as i32);
                    if sample_x >= 0 && sample_x < width as i32 {
                        let idx = (y * width + sample_x as u32) as usize;
                        let sample = input[idx];
                        let weight = kernel[i];
                        
                        result.r += sample.r * weight;
                        result.g += sample.g * weight;
                        result.b += sample.b * weight;
                        result.a += sample.a * weight;
                    }
                }
                
                temp[(y * width + x) as usize] = result;
            }
        }
        
        // Vertical pass
        let mut result = vec![Color::new(0.0, 0.0, 0.0, 0.0); input.len()];
        for y in 0..height {
            for x in 0..width {
                let mut pixel = Color::new(0.0, 0.0, 0.0, 0.0);
                
                for i in 0..kernel_size {
                    let sample_y = (y as i32) + (i as i32) - (center as i32);
                    if sample_y >= 0 && sample_y < height as i32 {
                        let idx = ((sample_y as u32) * width + x) as usize;
                        let sample = temp[idx];
                        let weight = kernel[i];
                        
                        pixel.r += sample.r * weight;
                        pixel.g += sample.g * weight;
                        pixel.b += sample.b * weight;
                        pixel.a += sample.a * weight;
                    }
                }
                
                result[(y * width + x) as usize] = pixel;
            }
        }
        
        result
    }
}

#[derive(Debug, Clone)]
pub struct DepthOfFieldEffect {
    pub focus_distance: f32,
    pub aperture: f32,
    pub focal_length: f32,
}

impl DepthOfFieldEffect {
    pub fn new() -> Self {
        Self {
            focus_distance: 5.0,
            aperture: 2.8,
            focal_length: 50.0,
        }
    }
    
    pub fn apply(&self, input: &[Color], depth_buffer: &[f32], width: u32, height: u32) -> Vec<Color> {
        let mut result = vec![Color::new(0.0, 0.0, 0.0, 0.0); input.len()];
        
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let depth = depth_buffer[idx];
                
                // Calculate circle of confusion
                let coc = self.circle_of_confusion(depth);
                let blur_radius = (coc * 10.0).max(0.0) as i32;
                
                if blur_radius <= 1 {
                    result[idx] = input[idx];
                } else {
                    // Apply blur based on depth
                    let mut accumulated = Color::new(0.0, 0.0, 0.0, 0.0);
                    let mut weight_sum = 0.0;
                    
                    for dy in -blur_radius..=blur_radius {
                        for dx in -blur_radius..=blur_radius {
                            let sample_x = (x as i32) + dx;
                            let sample_y = (y as i32) + dy;
                            
                            if sample_x >= 0 && sample_x < width as i32 && 
                               sample_y >= 0 && sample_y < height as i32 {
                                let sample_idx = ((sample_y as u32) * width + (sample_x as u32)) as usize;
                                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                                
                                if distance <= blur_radius as f32 {
                                    let weight = 1.0 - (distance / blur_radius as f32);
                                    let sample = input[sample_idx];
                                    
                                    accumulated.r += sample.r * weight;
                                    accumulated.g += sample.g * weight;
                                    accumulated.b += sample.b * weight;
                                    accumulated.a += sample.a * weight;
                                    weight_sum += weight;
                                }
                            }
                        }
                    }
                    
                    if weight_sum > 0.0 {
                        result[idx] = Color::new(
                            accumulated.r / weight_sum,
                            accumulated.g / weight_sum,
                            accumulated.b / weight_sum,
                            accumulated.a / weight_sum,
                        );
                    } else {
                        result[idx] = input[idx];
                    }
                }
            }
        }
        
        result
    }
    
    fn circle_of_confusion(&self, object_distance: f32) -> f32 {
        if object_distance <= 0.0 {
            return 0.0;
        }
        
        let focal_plane = 1.0 / ((1.0 / self.focal_length) - (1.0 / self.focus_distance));
        let object_plane = 1.0 / ((1.0 / self.focal_length) - (1.0 / object_distance));
        
        let coc = (self.focal_length / self.aperture) * 
                  ((object_plane - focal_plane).abs() / object_distance);
        
        coc / 1000.0 // Convert to normalized units
    }
}

#[derive(Debug, Clone)]
pub struct ScreenShakeEffect {
    pub intensity: f32,
    pub duration: f32,
    pub time_remaining: f32,
    pub frequency: f32,
    pub current_offset: (f32, f32),
}

impl ScreenShakeEffect {
    pub fn new() -> Self {
        Self {
            intensity: 0.0,
            duration: 0.0,
            time_remaining: 0.0,
            frequency: 15.0,
            current_offset: (0.0, 0.0),
        }
    }
    
    pub fn trigger(&mut self, intensity: f32, duration: f32) {
        self.intensity = intensity;
        self.duration = duration;
        self.time_remaining = duration;
    }
    
    pub fn update(&mut self, dt: f32) {
        if self.time_remaining > 0.0 {
            self.time_remaining -= dt;
            
            let progress = 1.0 - (self.time_remaining / self.duration);
            let current_intensity = self.intensity * (1.0 - progress * progress);
            
            use rand::Rng;
            let mut rng = rand::thread_rng();
            
            let angle = rng.gen::<f32>() * PI * 2.0;
            let magnitude = rng.gen::<f32>() * current_intensity;
            
            self.current_offset = (
                angle.cos() * magnitude,
                angle.sin() * magnitude,
            );
        } else {
            self.current_offset = (0.0, 0.0);
        }
    }
    
    pub fn get_offset(&self) -> (f32, f32) {
        self.current_offset
    }
    
    pub fn is_active(&self) -> bool {
        self.time_remaining > 0.0
    }
}

#[derive(Debug, Clone)]
pub struct WindEffect {
    pub direction: f32, // radians
    pub strength: f32,
    pub turbulence: f32,
    pub time: f32,
    pub noise: NoiseGenerator,
}

impl WindEffect {
    pub fn new() -> Self {
        Self {
            direction: 0.0,
            strength: 1.0,
            turbulence: 0.5,
            time: 0.0,
            noise: NoiseGenerator::new(12345),
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        self.time += dt;
    }
    
    pub fn get_force_at(&self, x: f32, y: f32) -> (f32, f32) {
        // Base wind direction
        let base_x = self.direction.cos() * self.strength;
        let base_y = self.direction.sin() * self.strength;
        
        // Add turbulence using noise
        let noise_scale = 0.01;
        let turbulence_x = self.noise.noise_2d(x * noise_scale, self.time * 0.5) * self.turbulence;
        let turbulence_y = self.noise.noise_2d((x + 1000.0) * noise_scale, self.time * 0.5) * self.turbulence;
        
        (base_x + turbulence_x, base_y + turbulence_y)
    }
    
    pub fn apply_to_particles(&self, particles: &mut [Particle]) {
        for particle in particles {
            let force = self.get_force_at(particle.position.0, particle.position.1);
            particle.velocity.0 += force.0 * 0.1;
            particle.velocity.1 += force.1 * 0.1;
        }
    }
}

impl Default for BloomEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DepthOfFieldEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ScreenShakeEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for WindEffect {
    fn default() -> Self {
        Self::new()
    }
}