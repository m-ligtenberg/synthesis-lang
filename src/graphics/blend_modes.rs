use crate::graphics::primitives::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    Normal,
    Add,
    Subtract,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
    ColorDodge,
    ColorBurn,
    Darken,
    Lighten,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    ColorBlend,
    Luminosity,
}

impl BlendMode {
    pub fn blend(&self, base: Color, blend: Color) -> Color {
        match self {
            BlendMode::Normal => blend,
            BlendMode::Add => Color::new(
                (base.r + blend.r).min(1.0),
                (base.g + blend.g).min(1.0),
                (base.b + blend.b).min(1.0),
                blend.a,
            ),
            BlendMode::Subtract => Color::new(
                (base.r - blend.r).max(0.0),
                (base.g - blend.g).max(0.0),
                (base.b - blend.b).max(0.0),
                blend.a,
            ),
            BlendMode::Multiply => Color::new(
                base.r * blend.r,
                base.g * blend.g,
                base.b * blend.b,
                blend.a,
            ),
            BlendMode::Screen => Color::new(
                1.0 - (1.0 - base.r) * (1.0 - blend.r),
                1.0 - (1.0 - base.g) * (1.0 - blend.g),
                1.0 - (1.0 - base.b) * (1.0 - blend.b),
                blend.a,
            ),
            BlendMode::Overlay => Color::new(
                overlay_channel(base.r, blend.r),
                overlay_channel(base.g, blend.g),
                overlay_channel(base.b, blend.b),
                blend.a,
            ),
            BlendMode::SoftLight => Color::new(
                soft_light_channel(base.r, blend.r),
                soft_light_channel(base.g, blend.g),
                soft_light_channel(base.b, blend.b),
                blend.a,
            ),
            BlendMode::HardLight => Color::new(
                hard_light_channel(base.r, blend.r),
                hard_light_channel(base.g, blend.g),
                hard_light_channel(base.b, blend.b),
                blend.a,
            ),
            BlendMode::ColorDodge => Color::new(
                color_dodge_channel(base.r, blend.r),
                color_dodge_channel(base.g, blend.g),
                color_dodge_channel(base.b, blend.b),
                blend.a,
            ),
            BlendMode::ColorBurn => Color::new(
                color_burn_channel(base.r, blend.r),
                color_burn_channel(base.g, blend.g),
                color_burn_channel(base.b, blend.b),
                blend.a,
            ),
            BlendMode::Darken => Color::new(
                base.r.min(blend.r),
                base.g.min(blend.g),
                base.b.min(blend.b),
                blend.a,
            ),
            BlendMode::Lighten => Color::new(
                base.r.max(blend.r),
                base.g.max(blend.g),
                base.b.max(blend.b),
                blend.a,
            ),
            BlendMode::Difference => Color::new(
                (base.r - blend.r).abs(),
                (base.g - blend.g).abs(),
                (base.b - blend.b).abs(),
                blend.a,
            ),
            BlendMode::Exclusion => Color::new(
                base.r + blend.r - 2.0 * base.r * blend.r,
                base.g + blend.g - 2.0 * base.g * blend.g,
                base.b + blend.b - 2.0 * base.b * blend.b,
                blend.a,
            ),
            BlendMode::Hue => {
                let base_hsl = rgb_to_hsl(base.r, base.g, base.b);
                let blend_hsl = rgb_to_hsl(blend.r, blend.g, blend.b);
                let result = hsl_to_rgb(blend_hsl.0, base_hsl.1, base_hsl.2);
                Color::new(result.0, result.1, result.2, blend.a)
            },
            BlendMode::Saturation => {
                let base_hsl = rgb_to_hsl(base.r, base.g, base.b);
                let blend_hsl = rgb_to_hsl(blend.r, blend.g, blend.b);
                let result = hsl_to_rgb(base_hsl.0, blend_hsl.1, base_hsl.2);
                Color::new(result.0, result.1, result.2, blend.a)
            },
            BlendMode::ColorBlend => {
                let base_hsl = rgb_to_hsl(base.r, base.g, base.b);
                let blend_hsl = rgb_to_hsl(blend.r, blend.g, blend.b);
                let result = hsl_to_rgb(blend_hsl.0, blend_hsl.1, base_hsl.2);
                Color::new(result.0, result.1, result.2, blend.a)
            },
            BlendMode::Luminosity => {
                let base_hsl = rgb_to_hsl(base.r, base.g, base.b);
                let blend_hsl = rgb_to_hsl(blend.r, blend.g, blend.b);
                let result = hsl_to_rgb(base_hsl.0, base_hsl.1, blend_hsl.2);
                Color::new(result.0, result.1, result.2, blend.a)
            },
        }
    }
}

// Helper functions for blend mode calculations
fn overlay_channel(base: f32, blend: f32) -> f32 {
    if base < 0.5 {
        2.0 * base * blend
    } else {
        1.0 - 2.0 * (1.0 - base) * (1.0 - blend)
    }
}

fn soft_light_channel(base: f32, blend: f32) -> f32 {
    if blend < 0.5 {
        2.0 * base * blend + base * base * (1.0 - 2.0 * blend)
    } else {
        2.0 * base * (1.0 - blend) + base.sqrt() * (2.0 * blend - 1.0)
    }
}

fn hard_light_channel(base: f32, blend: f32) -> f32 {
    if blend < 0.5 {
        2.0 * base * blend
    } else {
        1.0 - 2.0 * (1.0 - base) * (1.0 - blend)
    }
}

fn color_dodge_channel(base: f32, blend: f32) -> f32 {
    if blend >= 1.0 {
        1.0
    } else {
        (base / (1.0 - blend)).min(1.0)
    }
}

fn color_burn_channel(base: f32, blend: f32) -> f32 {
    if blend <= 0.0 {
        0.0
    } else {
        (1.0 - (1.0 - base) / blend).max(0.0)
    }
}

// Color space conversion functions
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    
    let lightness = (max + min) / 2.0;
    
    if delta == 0.0 {
        return (0.0, 0.0, lightness);
    }
    
    let saturation = if lightness < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };
    
    let hue = if max == r {
        ((g - b) / delta) % 6.0
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    
    let hue = hue * 60.0;
    let hue = if hue < 0.0 { hue + 360.0 } else { hue };
    
    (hue, saturation, lightness)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l);
    }
    
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    
    let (r_prime, g_prime, b_prime) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };
    
    (r_prime + m, g_prime + m, b_prime + m)
}

#[derive(Debug, Clone)]
pub struct CompositeLayer {
    pub pixels: Vec<Color>,
    pub width: u32,
    pub height: u32,
    pub blend_mode: BlendMode,
    pub opacity: f32,
}

impl CompositeLayer {
    pub fn new(width: u32, height: u32, blend_mode: BlendMode) -> Self {
        Self {
            pixels: vec![Color::new(0.0, 0.0, 0.0, 0.0); (width * height) as usize],
            width,
            height,
            blend_mode,
            opacity: 1.0,
        }
    }
    
    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color);
    }
    
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.pixels[index] = color;
        }
    }
    
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.pixels[index]
        } else {
            Color::new(0.0, 0.0, 0.0, 0.0)
        }
    }
    
    pub fn composite_onto(&self, base: &mut CompositeLayer) {
        if base.width != self.width || base.height != self.height {
            return;
        }
        
        for y in 0..self.height {
            for x in 0..self.width {
                let base_color = base.get_pixel(x, y);
                let blend_color = self.get_pixel(x, y);
                
                // Apply opacity
                let mut blend_color = blend_color;
                blend_color.a *= self.opacity;
                
                // Apply blend mode
                let result = self.blend_mode.blend(base_color, blend_color);
                
                // Alpha compositing
                let final_color = alpha_composite(base_color, result);
                base.set_pixel(x, y, final_color);
            }
        }
    }
}

fn alpha_composite(base: Color, overlay: Color) -> Color {
    let alpha = overlay.a + base.a * (1.0 - overlay.a);
    
    if alpha == 0.0 {
        return Color::new(0.0, 0.0, 0.0, 0.0);
    }
    
    Color::new(
        (overlay.r * overlay.a + base.r * base.a * (1.0 - overlay.a)) / alpha,
        (overlay.g * overlay.a + base.g * base.a * (1.0 - overlay.a)) / alpha,
        (overlay.b * overlay.a + base.b * base.a * (1.0 - overlay.a)) / alpha,
        alpha,
    )
}