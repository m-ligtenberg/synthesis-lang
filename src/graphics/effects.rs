use crate::runtime::Value;

#[derive(Debug, Clone)]
pub struct Plasma {
    pub speed: f32,
    pub palette: Palette,
    pub time: f32,
}

#[derive(Debug, Clone)]
pub struct Starfield {
    pub count: usize,
    pub speed: f32,
    pub stars: Vec<Star>,
}

#[derive(Debug, Clone)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub brightness: f32,
}

#[derive(Debug, Clone)]
pub enum Palette {
    Neon,
    Classic,
    Custom(Vec<[f32; 3]>),
}

impl Plasma {
    pub fn new(speed: f32, palette: Palette) -> Self {
        Self {
            speed,
            palette,
            time: 0.0,
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        self.time += dt * self.speed;
    }
    
    pub fn render_pixel(&self, x: f32, y: f32) -> [f32; 3] {
        let value = (self.time + (x * 0.1).sin() + (y * 0.1).cos()).sin() * 0.5 + 0.5;
        self.palette.color_at(value)
    }
}

impl Starfield {
    pub fn new(count: usize, speed: f32) -> Self {
        let mut stars = Vec::with_capacity(count);
        
        for _ in 0..count {
            stars.push(Star {
                x: (rand::random::<f32>() - 0.5) * 200.0,
                y: (rand::random::<f32>() - 0.5) * 200.0,
                z: rand::random::<f32>() * 100.0,
                brightness: rand::random::<f32>(),
            });
        }
        
        Self { count, speed, stars }
    }
    
    pub fn update(&mut self, dt: f32) {
        for star in &mut self.stars {
            star.z -= self.speed * dt;
            
            if star.z <= 0.0 {
                star.x = (rand::random::<f32>() - 0.5) * 200.0;
                star.y = (rand::random::<f32>() - 0.5) * 200.0;
                star.z = 100.0;
                star.brightness = rand::random::<f32>();
            }
        }
    }
    
    pub fn get_projected_stars(&self, width: f32, height: f32) -> Vec<ProjectedStar> {
        self.stars
            .iter()
            .filter_map(|star| {
                if star.z > 0.0 {
                    let screen_x = (star.x / star.z) * 100.0 + width * 0.5;
                    let screen_y = (star.y / star.z) * 100.0 + height * 0.5;
                    
                    if screen_x >= 0.0 && screen_x < width && screen_y >= 0.0 && screen_y < height {
                        Some(ProjectedStar {
                            x: screen_x,
                            y: screen_y,
                            brightness: star.brightness * (1.0 - star.z / 100.0),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ProjectedStar {
    pub x: f32,
    pub y: f32,
    pub brightness: f32,
}

impl Palette {
    pub fn neon() -> Self {
        Self::Neon
    }
    
    pub fn classic() -> Self {
        Self::Classic
    }
    
    pub fn color_at(&self, t: f32) -> [f32; 3] {
        match self {
            Palette::Neon => [
                (t * 2.0 * std::f32::consts::PI).sin() * 0.5 + 0.5,
                (t * 2.0 * std::f32::consts::PI + 2.0).sin() * 0.5 + 0.5,
                (t * 2.0 * std::f32::consts::PI + 4.0).sin() * 0.5 + 0.5,
            ],
            Palette::Classic => [
                t,
                t * 0.7,
                t * 0.3,
            ],
            Palette::Custom(colors) => {
                if colors.is_empty() {
                    return [0.0, 0.0, 0.0];
                }
                
                let index = (t * colors.len() as f32) as usize;
                let index = index.min(colors.len() - 1);
                colors[index]
            }
        }
    }
}

pub struct EffectRenderer;

impl EffectRenderer {
    pub fn create_plasma(args: &[Value]) -> crate::Result<Plasma> {
        let speed = args.get(0)
            .and_then(|v| v.as_number())
            .unwrap_or(1.0) as f32;
        
        let palette = Palette::neon();
        
        Ok(Plasma::new(speed, palette))
    }
    
    pub fn create_starfield(args: &[Value]) -> crate::Result<Starfield> {
        let count = args.get(0)
            .and_then(|v| v.as_number())
            .unwrap_or(100.0) as usize;
        
        let speed = args.get(1)
            .and_then(|v| v.as_number())
            .unwrap_or(1.0) as f32;
        
        Ok(Starfield::new(count, speed))
    }
}