#[derive(Debug, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }
    
    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self::rgb(r, g, b)
    }
    
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn from_percent(x_percent: f32, y_percent: f32, width: f32, height: f32) -> Self {
        Self {
            x: x_percent * width / 100.0,
            y: y_percent * height / 100.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: Color,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32, color: Color) -> Self {
        Self { x, y, width, height, color }
    }
    
    pub fn from_percent(
        x_percent: f32,
        y_percent: f32,
        width_percent: f32,
        height_percent: f32,
        screen_width: f32,
        screen_height: f32,
        color: Color,
    ) -> Self {
        Self {
            x: x_percent * screen_width / 100.0,
            y: y_percent * screen_height / 100.0,
            width: width_percent * screen_width / 100.0,
            height: height_percent * screen_height / 100.0,
            color,
        }
    }
    
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: Color,
}

impl Circle {
    pub fn new(x: f32, y: f32, radius: f32, color: Color) -> Self {
        Self { x, y, radius, color }
    }
    
    pub fn from_percent(
        x_percent: f32,
        y_percent: f32,
        radius_percent: f32,
        screen_width: f32,
        screen_height: f32,
        color: Color,
    ) -> Self {
        let min_dimension = screen_width.min(screen_height);
        Self {
            x: x_percent * screen_width / 100.0,
            y: y_percent * screen_height / 100.0,
            radius: radius_percent * min_dimension / 100.0,
            color,
        }
    }
    
    pub fn contains(&self, x: f32, y: f32) -> bool {
        let dx = x - self.x;
        let dy = y - self.y;
        dx * dx + dy * dy <= self.radius * self.radius
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pub start: Point,
    pub end: Point,
    pub color: Color,
    pub thickness: f32,
}

impl Line {
    pub fn new(start: Point, end: Point, color: Color, thickness: f32) -> Self {
        Self { start, end, color, thickness }
    }
}