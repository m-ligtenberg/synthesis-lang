use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct GameController {
    pub name: String,
    pub id: u32,
    pub connected: bool,
    pub axes: Vec<f32>,
    pub buttons: Vec<bool>,
    pub last_update: Instant,
}

#[derive(Debug, Clone)]
pub struct ControllerEvent {
    pub controller_id: u32,
    pub timestamp: Instant,
    pub event_type: ControllerEventType,
}

#[derive(Debug, Clone)]
pub enum ControllerEventType {
    ButtonPressed(u8),
    ButtonReleased(u8),
    AxisMoved { axis: u8, value: f32 },
    Connected,
    Disconnected,
}

pub struct ControllerManager {
    controllers: HashMap<u32, GameController>,
    events: Vec<ControllerEvent>,
    deadzone: f32,
}

impl ControllerManager {
    pub fn new() -> Self {
        Self {
            controllers: HashMap::new(),
            events: Vec::new(),
            deadzone: 0.1,
        }
    }
    
    pub fn set_deadzone(&mut self, deadzone: f32) {
        self.deadzone = deadzone.clamp(0.0, 1.0);
    }
    
    pub fn update(&mut self) {
        // This would typically interface with a gamepad library like gilrs
        // For now, we'll simulate the interface
        self.poll_controllers();
    }
    
    fn poll_controllers(&mut self) {
        // In a real implementation, this would check for connected controllers
        // and poll their current state
    }
    
    pub fn get_controller(&self, id: u32) -> Option<&GameController> {
        self.controllers.get(&id)
    }
    
    pub fn get_connected_controllers(&self) -> Vec<&GameController> {
        self.controllers
            .values()
            .filter(|c| c.connected)
            .collect()
    }
    
    pub fn is_button_pressed(&self, controller_id: u32, button: u8) -> bool {
        self.controllers
            .get(&controller_id)
            .and_then(|c| c.buttons.get(button as usize))
            .copied()
            .unwrap_or(false)
    }
    
    pub fn get_axis_value(&self, controller_id: u32, axis: u8) -> f32 {
        let raw_value = self.controllers
            .get(&controller_id)
            .and_then(|c| c.axes.get(axis as usize))
            .copied()
            .unwrap_or(0.0);
        
        // Apply deadzone
        if raw_value.abs() < self.deadzone {
            0.0
        } else {
            // Scale to account for deadzone
            let sign = raw_value.signum();
            let scaled = (raw_value.abs() - self.deadzone) / (1.0 - self.deadzone);
            sign * scaled
        }
    }
    
    pub fn get_events_since(&mut self, since: Instant) -> Vec<ControllerEvent> {
        let events = self.events
            .iter()
            .filter(|e| e.timestamp >= since)
            .cloned()
            .collect();
        
        // Clear old events
        self.events.retain(|e| e.timestamp >= since);
        
        events
    }
    
    pub fn clear_events(&mut self) {
        self.events.clear();
    }
    
    // Convenience methods for common controller mappings
    pub fn get_left_stick(&self, controller_id: u32) -> (f32, f32) {
        (
            self.get_axis_value(controller_id, 0), // Left stick X
            self.get_axis_value(controller_id, 1), // Left stick Y
        )
    }
    
    pub fn get_right_stick(&self, controller_id: u32) -> (f32, f32) {
        (
            self.get_axis_value(controller_id, 2), // Right stick X
            self.get_axis_value(controller_id, 3), // Right stick Y
        )
    }
    
    pub fn get_triggers(&self, controller_id: u32) -> (f32, f32) {
        (
            self.get_axis_value(controller_id, 4), // Left trigger
            self.get_axis_value(controller_id, 5), // Right trigger
        )
    }
    
    pub fn is_face_button_pressed(&self, controller_id: u32, button: FaceButton) -> bool {
        let button_index = match button {
            FaceButton::A => 0,
            FaceButton::B => 1,
            FaceButton::X => 2,
            FaceButton::Y => 3,
        };
        self.is_button_pressed(controller_id, button_index)
    }
    
    pub fn is_shoulder_button_pressed(&self, controller_id: u32, button: ShoulderButton) -> bool {
        let button_index = match button {
            ShoulderButton::LeftBumper => 4,
            ShoulderButton::RightBumper => 5,
            ShoulderButton::LeftTrigger => 6,
            ShoulderButton::RightTrigger => 7,
        };
        self.is_button_pressed(controller_id, button_index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FaceButton {
    A,
    B,
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShoulderButton {
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
}

// Utility struct for controller mappings in creative applications
#[derive(Debug, Clone)]
pub struct CreativeControllerMapping {
    pub controller_id: u32,
    pub mappings: HashMap<String, ControlMapping>,
}

#[derive(Debug, Clone)]
pub enum ControlMapping {
    Axis { axis: u8, range: (f32, f32), curve: f32 },
    Button { button: u8, mode: ButtonMode },
    Combined { axes: Vec<u8>, operation: CombineOperation },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonMode {
    Momentary,   // Active only while pressed
    Toggle,      // Toggle state on press
    Trigger,     // Fire event on press
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CombineOperation {
    Add,
    Multiply,
    Max,
    Min,
    Distance, // For XY coordinates
}

impl CreativeControllerMapping {
    pub fn new(controller_id: u32) -> Self {
        Self {
            controller_id,
            mappings: HashMap::new(),
        }
    }
    
    pub fn map_axis(&mut self, name: String, axis: u8, range: (f32, f32), curve: f32) {
        self.mappings.insert(name, ControlMapping::Axis { axis, range, curve });
    }
    
    pub fn map_button(&mut self, name: String, button: u8, mode: ButtonMode) {
        self.mappings.insert(name, ControlMapping::Button { button, mode });
    }
    
    pub fn map_combined(&mut self, name: String, axes: Vec<u8>, operation: CombineOperation) {
        self.mappings.insert(name, ControlMapping::Combined { axes, operation });
    }
    
    pub fn evaluate(&self, name: &str, controller_manager: &ControllerManager) -> Option<f32> {
        let mapping = self.mappings.get(name)?;
        
        match mapping {
            ControlMapping::Axis { axis, range, curve } => {
                let raw_value = controller_manager.get_axis_value(self.controller_id, *axis);
                
                // Apply curve (exponential scaling)
                let curved_value = if *curve == 1.0 {
                    raw_value
                } else {
                    raw_value.signum() * raw_value.abs().powf(*curve)
                };
                
                // Map to range
                let normalized = (curved_value + 1.0) * 0.5; // -1..1 to 0..1
                Some(range.0 + normalized * (range.1 - range.0))
            }
            ControlMapping::Button { button, mode: _ } => {
                let pressed = controller_manager.is_button_pressed(self.controller_id, *button);
                Some(if pressed { 1.0 } else { 0.0 })
            }
            ControlMapping::Combined { axes, operation } => {
                let values: Vec<f32> = axes
                    .iter()
                    .map(|&axis| controller_manager.get_axis_value(self.controller_id, axis))
                    .collect();
                
                if values.is_empty() {
                    return Some(0.0);
                }
                
                match operation {
                    CombineOperation::Add => Some(values.iter().sum()),
                    CombineOperation::Multiply => Some(values.iter().product()),
                    CombineOperation::Max => Some(values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))),
                    CombineOperation::Min => Some(values.iter().fold(f32::INFINITY, |a, &b| a.min(b))),
                    CombineOperation::Distance => {
                        if values.len() >= 2 {
                            Some((values[0] * values[0] + values[1] * values[1]).sqrt())
                        } else {
                            Some(values[0].abs())
                        }
                    }
                }
            }
        }
    }
    
    // Preset mappings for common creative applications
    pub fn setup_audio_visualizer_mapping(&mut self) {
        self.map_axis("volume".to_string(), 5, (0.0, 1.0), 1.0); // Right trigger
        self.map_axis("frequency".to_string(), 0, (100.0, 8000.0), 2.0); // Left stick X with curve
        self.map_axis("resonance".to_string(), 1, (0.1, 10.0), 1.5); // Left stick Y
        self.map_combined("position".to_string(), vec![2, 3], CombineOperation::Distance); // Right stick distance
        self.map_button("beat_trigger".to_string(), 0, ButtonMode::Trigger); // A button
        self.map_button("effect_toggle".to_string(), 1, ButtonMode::Toggle); // B button
    }
    
    pub fn setup_graphics_control_mapping(&mut self) {
        self.map_combined("brush_position".to_string(), vec![0, 1], CombineOperation::Distance);
        self.map_axis("brush_size".to_string(), 4, (1.0, 50.0), 2.0); // Left trigger
        self.map_axis("brush_opacity".to_string(), 5, (0.1, 1.0), 1.0); // Right trigger
        self.map_combined("color_hue".to_string(), vec![2], CombineOperation::Add); // Right stick X
        self.map_combined("color_saturation".to_string(), vec![3], CombineOperation::Add); // Right stick Y
        self.map_button("clear_canvas".to_string(), 2, ButtonMode::Trigger); // X button
        self.map_button("undo".to_string(), 3, ButtonMode::Trigger); // Y button
    }
}

impl Default for ControllerManager {
    fn default() -> Self {
        Self::new()
    }
}