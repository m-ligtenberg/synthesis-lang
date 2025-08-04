use crate::runtime::types::Value;
use std::collections::HashMap;

// L-System implementation
#[derive(Debug, Clone)]
pub struct LSystem {
    pub axiom: String,
    pub rules: HashMap<char, String>,
    pub angle: f64,
    pub iterations: usize,
}

impl LSystem {
    pub fn new(axiom: String, rules: HashMap<char, String>, angle: f64, iterations: usize) -> Self {
        Self {
            axiom,
            rules,
            angle,
            iterations,
        }
    }
    
    pub fn generate(&self) -> String {
        let mut current = self.axiom.clone();
        
        for _ in 0..self.iterations {
            let mut next = String::new();
            for ch in current.chars() {
                if let Some(replacement) = self.rules.get(&ch) {
                    next.push_str(replacement);
                } else {
                    next.push(ch);
                }
            }
            current = next;
        }
        
        current
    }
}

// Perlin Noise implementation
pub struct PerlinNoise {
    permutation: [usize; 512],
}

impl PerlinNoise {
    pub fn new(seed: u32) -> Self {
        let mut perm = [0; 256];
        for i in 0..256 {
            perm[i] = i;
        }
        
        // Fisher-Yates shuffle with seed
        let mut rng_state = seed;
        for i in (1..256).rev() {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let j = (rng_state as usize) % (i + 1);
            perm.swap(i, j);
        }
        
        let mut permutation = [0; 512];
        for i in 0..512 {
            permutation[i] = perm[i % 256];
        }
        
        Self { permutation }
    }
    
    pub fn noise(&self, x: f64, y: f64, z: f64) -> f64 {
        let xi = (x.floor() as i32 & 255) as usize;
        let yi = (y.floor() as i32 & 255) as usize;
        let zi = (z.floor() as i32 & 255) as usize;
        
        let x = x - x.floor();
        let y = y - y.floor();
        let z = z - z.floor();
        
        let u = Self::fade(x);
        let v = Self::fade(y);
        let w = Self::fade(z);
        
        let a = self.permutation[xi] + yi;
        let aa = self.permutation[a] + zi;
        let ab = self.permutation[a + 1] + zi;
        let b = self.permutation[xi + 1] + yi;
        let ba = self.permutation[b] + zi;
        let bb = self.permutation[b + 1] + zi;
        
        Self::lerp(w,
            Self::lerp(v,
                Self::lerp(u, Self::grad(self.permutation[aa], x, y, z),
                             Self::grad(self.permutation[ba], x - 1.0, y, z)),
                Self::lerp(u, Self::grad(self.permutation[ab], x, y - 1.0, z),
                             Self::grad(self.permutation[bb], x - 1.0, y - 1.0, z))),
            Self::lerp(v,
                Self::lerp(u, Self::grad(self.permutation[aa + 1], x, y, z - 1.0),
                             Self::grad(self.permutation[ba + 1], x - 1.0, y, z - 1.0)),
                Self::lerp(u, Self::grad(self.permutation[ab + 1], x, y - 1.0, z - 1.0),
                             Self::grad(self.permutation[bb + 1], x - 1.0, y - 1.0, z - 1.0))))
    }
    
    fn fade(t: f64) -> f64 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }
    
    fn lerp(t: f64, a: f64, b: f64) -> f64 {
        a + t * (b - a)
    }
    
    fn grad(hash: usize, x: f64, y: f64, z: f64) -> f64 {
        let h = hash & 15;
        let u = if h < 8 { x } else { y };
        let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
        (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
    }
}

// Euclidean Rhythm implementation
pub struct EuclideanRhythm {
    hits: usize,
    steps: usize,
    pattern: Vec<bool>,
    position: usize,
}

impl EuclideanRhythm {
    pub fn new(hits: usize, steps: usize) -> Self {
        let pattern = Self::generate_pattern(hits, steps);
        Self {
            hits,
            steps,
            pattern,
            position: 0,
        }
    }
    
    fn generate_pattern(hits: usize, steps: usize) -> Vec<bool> {
        if hits == 0 || steps == 0 {
            return vec![false; steps];
        }
        
        let mut pattern = vec![false; steps];
        let mut counts = vec![0; steps];
        let mut remainders = vec![0; steps];
        
        let mut divisor = steps - hits;
        remainders[0] = hits;
        
        let mut level = 0;
        loop {
            counts[level] = divisor / remainders[level];
            remainders[level + 1] = divisor % remainders[level];
            divisor = remainders[level];
            level += 1;
            if remainders[level] <= 1 {
                break;
            }
        }
        
        counts[level] = divisor;
        
        Self::build_pattern(&mut pattern, level, counts, remainders, 0);
        pattern
    }
    
    fn build_pattern(pattern: &mut Vec<bool>, level: usize, counts: Vec<usize>, remainders: Vec<usize>, pos: usize) -> usize {
        let mut pos = pos;
        if level == 0 {
            for _ in 0..counts[0] {
                pattern[pos] = false;
                pos += 1;
            }
            if remainders[0] > 0 {
                pattern[pos] = true;
                pos += 1;
            }
        } else {
            for _ in 0..counts[level] {
                pos = Self::build_pattern(pattern, level - 1, counts.clone(), remainders.clone(), pos);
            }
            if remainders[level] > 0 {
                pos = Self::build_pattern(pattern, level - 1, counts.clone(), remainders.clone(), pos);
            }
        }
        pos
    }
    
    pub fn tick(&mut self) -> bool {
        let result = self.pattern[self.position];
        self.position = (self.position + 1) % self.steps;
        result
    }
    
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

// Fractal terrain generation
pub struct FractalTerrain {
    width: usize,
    height: usize,
    octaves: usize,
    persistence: f64,
    scale: f64,
    height_multiplier: f64,
    noise: PerlinNoise,
}

impl FractalTerrain {
    pub fn new(width: usize, height: usize, octaves: usize, persistence: f64, scale: f64, height_multiplier: f64, seed: u32) -> Self {
        Self {
            width,
            height,
            octaves,
            persistence,
            scale,
            height_multiplier,
            noise: PerlinNoise::new(seed),
        }
    }
    
    pub fn generate_heightmap(&self) -> Vec<Vec<f64>> {
        let mut heightmap = vec![vec![0.0; self.width]; self.height];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let mut value = 0.0;
                let mut amplitude = 1.0;
                let mut frequency = self.scale;
                
                for _ in 0..self.octaves {
                    let sample_x = x as f64 * frequency;
                    let sample_y = y as f64 * frequency;
                    
                    let noise_value = self.noise.noise(sample_x, sample_y, 0.0);
                    value += noise_value * amplitude;
                    
                    amplitude *= self.persistence;
                    frequency *= 2.0;
                }
                
                heightmap[y][x] = value * self.height_multiplier;
            }
        }
        
        heightmap
    }
}

// Module functions for the runtime
pub fn l_system(_args: &[Value]) -> crate::Result<Value> {
    // For now, return a placeholder object
    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::String("l_system".to_string()));
    Ok(Value::Object(result))
}

pub fn perlin_noise(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 3 {
        return Err(anyhow::anyhow!("perlin_noise requires 3 arguments (x, y, z)"));
    }
    
    let x = args[0].as_number().ok_or_else(|| anyhow::anyhow!("First argument must be a number"))?;
    let y = args[1].as_number().ok_or_else(|| anyhow::anyhow!("Second argument must be a number"))?;
    let z = args[2].as_number().ok_or_else(|| anyhow::anyhow!("Third argument must be a number"))?;
    
    let noise = PerlinNoise::new(0); // Default seed
    let value = noise.noise(x, y, z);
    
    Ok(Value::Float(value))
}

pub fn euclidean(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(anyhow::anyhow!("euclidean requires 2 arguments (hits, steps)"));
    }
    
    let hits = match &args[0] {
        Value::Integer(n) => *n as usize,
        _ => return Err(anyhow::anyhow!("First argument must be an integer")),
    };
    
    let steps = match &args[1] {
        Value::Integer(n) => *n as usize,
        _ => return Err(anyhow::anyhow!("Second argument must be an integer")),
    };
    
    let rhythm = EuclideanRhythm::new(hits, steps);
    let pattern: Vec<Value> = rhythm.pattern.iter().map(|&b| Value::Boolean(b)).collect();
    
    Ok(Value::Array(pattern))
}

pub fn fractal_terrain(_args: &[Value]) -> crate::Result<Value> {
    // For now, return a placeholder object
    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::String("fractal_terrain".to_string()));
    Ok(Value::Object(result))
}