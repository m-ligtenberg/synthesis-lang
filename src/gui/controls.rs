use egui::*;
use std::collections::HashMap;
//use crate::runtime::Value;

#[derive(Debug, Clone)]
pub struct ControlState {
    pub sliders: HashMap<String, f32>,
    pub buttons: HashMap<String, bool>,
    pub toggles: HashMap<String, bool>,
    pub dropdowns: HashMap<String, String>,
    pub knobs: HashMap<String, f32>,
    pub xy_pads: HashMap<String, (f32, f32)>,
    pub color_pickers: HashMap<String, [f32; 3]>,
}

impl Default for ControlState {
    fn default() -> Self {
        Self {
            sliders: HashMap::new(),
            buttons: HashMap::new(),
            toggles: HashMap::new(),
            dropdowns: HashMap::new(),
            knobs: HashMap::new(),
            xy_pads: HashMap::new(),
            color_pickers: HashMap::new(),
        }
    }
}

pub struct SynthesisGUI {
    pub control_state: ControlState,
    pub windows: HashMap<String, WindowState>,
    pub theme: GuiTheme,
}

#[derive(Debug, Clone)]
pub struct WindowState {
    pub open: bool,
    pub size: Option<Vec2>,
    pub position: Option<Pos2>,
    pub resizable: bool,
    pub collapsible: bool,
}

#[derive(Debug, Clone)]
pub enum GuiTheme {
    Dark,
    Light,
    Neon,
    Custom {
        background: Color32,
        text: Color32,
        accent: Color32,
        widget: Color32,
    },
}

impl SynthesisGUI {
    pub fn new() -> Self {
        Self {
            control_state: ControlState::default(),
            windows: HashMap::new(),
            theme: GuiTheme::Dark,
        }
    }
    
    pub fn apply_theme(&self, ctx: &Context) {
        let visuals = match &self.theme {
            GuiTheme::Dark => Visuals::dark(),
            GuiTheme::Light => Visuals::light(),
            GuiTheme::Neon => {
                let mut visuals = Visuals::dark();
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(20, 20, 30);
                visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 40, 60);
                visuals.widgets.hovered.bg_fill = Color32::from_rgb(0, 255, 150);
                visuals.widgets.active.bg_fill = Color32::from_rgb(0, 200, 100);
                visuals.selection.bg_fill = Color32::from_rgb(0, 150, 255);
                visuals
            }
            GuiTheme::Custom { background, text: _, accent, widget } => {
                let mut visuals = Visuals::dark();
                visuals.panel_fill = *background;
                visuals.window_fill = *background;
                // Note: text_color is handled automatically by egui based on theme
                visuals.widgets.inactive.bg_fill = *widget;
                visuals.widgets.hovered.bg_fill = *accent;
                visuals.widgets.active.bg_fill = *accent;
                visuals
            }
        };
        ctx.set_visuals(visuals);
    }
    
    pub fn show_window<F>(&mut self, ctx: &Context, title: &str, content: F)
    where
        F: FnOnce(&mut Ui, &mut ControlState),
    {
        let window_state = self.windows.entry(title.to_string()).or_insert(WindowState {
            open: true,
            size: None,
            position: None,
            resizable: true,
            collapsible: true,
        });
        
        let mut window = Window::new(title)
            .open(&mut window_state.open)
            .resizable(window_state.resizable)
            .collapsible(window_state.collapsible);
        
        if let Some(size) = window_state.size {
            window = window.default_size(size);
        }
        
        if let Some(pos) = window_state.position {
            window = window.default_pos(pos);
        }
        
        window.show(ctx, |ui| {
            content(ui, &mut self.control_state);
        });
    }
    
    // Control widgets
    pub fn slider(
        &mut self,
        ui: &mut Ui,
        id: &str,
        label: &str,
        min: f32,
        max: f32,
        default: f32,
    ) -> f32 {
        let value = self.control_state.sliders.entry(id.to_string()).or_insert(default);
        ui.add(Slider::new(value, min..=max).text(label));
        *value
    }
    
    pub fn knob(
        &mut self,
        ui: &mut Ui,
        id: &str,
        label: &str,
        min: f32,
        max: f32,
        default: f32,
    ) -> f32 {
        let value = self.control_state.knobs.entry(id.to_string()).or_insert(default);
        
        ui.vertical(|ui| {
            ui.label(label);
            let response = ui.allocate_response(Vec2::splat(60.0), Sense::drag());
            
            if response.dragged() {
                let delta = response.drag_delta().y * -0.01;
                *value = (*value + delta * (max - min)).clamp(min, max);
            }
            
            // Draw knob
            let painter = ui.painter();
            let rect = response.rect;
            let center = rect.center();
            let radius = rect.width().min(rect.height()) * 0.4;
            
            // Background circle
            painter.circle_filled(center, radius, ui.visuals().widgets.inactive.bg_fill);
            painter.circle_stroke(center, radius, Stroke::new(2.0, ui.visuals().text_color()));
            
            // Value indicator
            let angle = ((*value - min) / (max - min)) * std::f32::consts::PI * 1.5 - std::f32::consts::PI * 0.75;
            let indicator_end = center + Vec2::new(angle.cos(), angle.sin()) * radius * 0.8;
            painter.line_segment([center, indicator_end], Stroke::new(3.0, ui.visuals().selection.bg_fill));
            
            // Value text
            ui.label(format!("{:.2}", value));
        });
        
        *value
    }
    
    pub fn button(&mut self, ui: &mut Ui, id: &str, label: &str) -> bool {
        let pressed = ui.button(label).clicked();
        if pressed {
            self.control_state.buttons.insert(id.to_string(), true);
        }
        pressed
    }
    
    pub fn toggle(
        &mut self,
        ui: &mut Ui,
        id: &str,
        label: &str,
        default: bool,
    ) -> bool {
        let value = self.control_state.toggles.entry(id.to_string()).or_insert(default);
        ui.checkbox(value, label);
        *value
    }
    
    pub fn dropdown<T: Clone + PartialEq>(
        &mut self,
        ui: &mut Ui,
        id: &str,
        label: &str,
        options: &[(String, T)],
        default_index: usize,
    ) -> T {
        let selected_text = self.control_state.dropdowns
            .entry(id.to_string())
            .or_insert_with(|| options.get(default_index).map_or(String::new(), |x| x.0.clone()));
        
        ComboBox::from_label(label)
            .selected_text(selected_text.as_str())
            .show_ui(ui, |ui| {
                for (text, _value) in options {
                    ui.selectable_value(selected_text, text.clone(), text);
                }
            });
        
        options
            .iter()
            .find(|(text, _)| text == selected_text)
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| options.get(default_index).unwrap().1.clone())
    }
    
    pub fn xy_pad(
        &mut self,
        ui: &mut Ui,
        id: &str,
        label: &str,
        x_range: (f32, f32),
        y_range: (f32, f32),
        default: (f32, f32),
    ) -> (f32, f32) {
        let value = self.control_state.xy_pads.entry(id.to_string()).or_insert(default);
        
        ui.vertical(|ui| {
            ui.label(label);
            let response = ui.allocate_response(Vec2::new(150.0, 150.0), Sense::drag());
            
            if response.dragged() {
                let rect = response.rect;
                let relative_pos = response.interact_pointer_pos()
                    .unwrap_or(rect.center()) - rect.min;
                
                let x_norm = (relative_pos.x / rect.width()).clamp(0.0, 1.0);
                let y_norm = 1.0 - (relative_pos.y / rect.height()).clamp(0.0, 1.0);
                
                value.0 = x_range.0 + x_norm * (x_range.1 - x_range.0);
                value.1 = y_range.0 + y_norm * (y_range.1 - y_range.0);
            }
            
            // Draw XY pad
            let painter = ui.painter();
            let rect = response.rect;
            
            // Background
            painter.rect_filled(rect, 4.0, ui.visuals().widgets.inactive.bg_fill);
            painter.rect_stroke(rect, 4.0, Stroke::new(2.0, ui.visuals().text_color()));
            
            // Position indicator
            let x_norm = (value.0 - x_range.0) / (x_range.1 - x_range.0);
            let y_norm = 1.0 - (value.1 - y_range.0) / (y_range.1 - y_range.0);
            let indicator_pos = rect.min + Vec2::new(
                x_norm * rect.width(),
                y_norm * rect.height()
            );
            
            painter.circle_filled(indicator_pos, 6.0, ui.visuals().selection.bg_fill);
            painter.circle_stroke(indicator_pos, 6.0, Stroke::new(2.0, ui.visuals().text_color()));
            
            ui.label(format!("X: {:.2}, Y: {:.2}", value.0, value.1));
        });
        
        *value
    }
    
    pub fn color_picker(
        &mut self,
        ui: &mut Ui,
        id: &str,
        label: &str,
        default: [f32; 3],
    ) -> [f32; 3] {
        let value = self.control_state.color_pickers.entry(id.to_string()).or_insert(default);
        
        ui.horizontal(|ui| {
            ui.label(label);
            ui.color_edit_button_rgb(value);
        });
        
        *value
    }
    
    pub fn spectrum_analyzer(
        &mut self,
        ui: &mut Ui,
        fft_data: &[f32],
        width: f32,
        height: f32,
    ) {
        let response = ui.allocate_response(Vec2::new(width, height), Sense::hover());
        let rect = response.rect;
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(10, 10, 15));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, ui.visuals().text_color()));
        
        if !fft_data.is_empty() {
            let bar_width = rect.width() / fft_data.len() as f32;
            
            for (i, &magnitude) in fft_data.iter().enumerate() {
                let bar_height = magnitude * rect.height() * 0.8;
                let x = rect.min.x + i as f32 * bar_width;
                let bar_rect = Rect::from_min_size(
                    Pos2::new(x, rect.max.y - bar_height),
                    Vec2::new(bar_width - 1.0, bar_height)
                );
                
                // Color based on frequency (low = red, high = blue)
                let hue = i as f32 / fft_data.len() as f32;
                let color = Color32::from_rgb(
                    (255.0 * (1.0 - hue)) as u8,
                    (255.0 * magnitude * 2.0).min(255.0) as u8,
                    (255.0 * hue) as u8,
                );
                
                painter.rect_filled(bar_rect, 0.0, color);
            }
        }
    }
    
    pub fn oscilloscope(
        &mut self,
        ui: &mut Ui,
        samples: &[f32],
        width: f32,
        height: f32,
    ) {
        let response = ui.allocate_response(Vec2::new(width, height), Sense::hover());
        let rect = response.rect;
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(5, 10, 5));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, ui.visuals().text_color()));
        
        // Center line
        let center_y = rect.center().y;
        painter.line_segment(
            [Pos2::new(rect.min.x, center_y), Pos2::new(rect.max.x, center_y)],
            Stroke::new(1.0, Color32::DARK_GRAY)
        );
        
        if samples.len() > 1 {
            let mut points = Vec::new();
            let x_step = rect.width() / samples.len() as f32;
            
            for (i, &sample) in samples.iter().enumerate() {
                let x = rect.min.x + i as f32 * x_step;
                let y = center_y - sample * rect.height() * 0.4;
                points.push(Pos2::new(x, y));
            }
            
            if points.len() > 1 {
                painter.add(Shape::line(points, Stroke::new(2.0, Color32::GREEN)));
            }
        }
    }
    
    pub fn level_meter(
        &mut self,
        ui: &mut Ui,
        level: f32,
        peak: f32,
        width: f32,
        height: f32,
        vertical: bool,
    ) {
        let response = ui.allocate_response(Vec2::new(width, height), Sense::hover());
        let rect = response.rect;
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(rect, 2.0, Color32::from_rgb(20, 20, 20));
        painter.rect_stroke(rect, 2.0, Stroke::new(1.0, ui.visuals().text_color()));
        
        if vertical {
            // Vertical level meter
            let level_height = level * rect.height();
            let level_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + 2.0, rect.max.y - level_height),
                Vec2::new(rect.width() - 4.0, level_height - 2.0)
            );
            
            // Color gradient based on level
            let color = if level > 0.8 {
                Color32::RED
            } else if level > 0.6 {
                Color32::YELLOW
            } else {
                Color32::GREEN
            };
            
            painter.rect_filled(level_rect, 0.0, color);
            
            // Peak indicator
            if peak > 0.01 {
                let peak_y = rect.max.y - peak * rect.height();
                painter.line_segment(
                    [Pos2::new(rect.min.x, peak_y), Pos2::new(rect.max.x, peak_y)],
                    Stroke::new(2.0, Color32::WHITE)
                );
            }
        } else {
            // Horizontal level meter
            let level_width = level * rect.width();
            let level_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + 2.0, rect.min.y + 2.0),
                Vec2::new(level_width - 2.0, rect.height() - 4.0)
            );
            
            let color = if level > 0.8 {
                Color32::RED
            } else if level > 0.6 {
                Color32::YELLOW
            } else {
                Color32::GREEN
            };
            
            painter.rect_filled(level_rect, 0.0, color);
            
            // Peak indicator
            if peak > 0.01 {
                let peak_x = rect.min.x + peak * rect.width();
                painter.line_segment(
                    [Pos2::new(peak_x, rect.min.y), Pos2::new(peak_x, rect.max.y)],
                    Stroke::new(2.0, Color32::WHITE)
                );
            }
        }
    }
}

impl Default for SynthesisGUI {
    fn default() -> Self {
        Self::new()
    }
}