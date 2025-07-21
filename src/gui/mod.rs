pub mod controls;

use egui::*;

pub use controls::*;

pub struct SynthesisGui {
    open: bool,
    pub gui: SynthesisGUI,
}

impl Default for SynthesisGui {
    fn default() -> Self {
        Self { 
            open: true,
            gui: SynthesisGUI::new(),
        }
    }
}

impl SynthesisGui {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn show(&mut self, ctx: &Context) {
        self.gui.apply_theme(ctx);
        
        self.gui.show_window(ctx, "Synthesis Editor", |ui, _control_state| {
            ui.heading("Synthesis Creative Programming Language");
            ui.separator();
            
            ui.label("Welcome to Synthesis!");
            ui.label("A creative programming language for artists and musicians.");
            
            ui.separator();
            
            // Example controls
            ui.horizontal(|ui| {
                if ui.button("Run Script").clicked() {
                    println!("Run button clicked");
                }
                
                if ui.button("Load Example").clicked() {
                    println!("Load example button clicked");
                }
                
                if ui.button("New Project").clicked() {
                    println!("New project button clicked");
                }
            });
            
            ui.separator();
            
            // Basic controls demo
            ui.collapsing("Controls", |ui| {
                ui.label("Phase 2 GUI System Demo");
                ui.horizontal(|ui| {
                    ui.button("Test Button");
                    ui.checkbox(&mut true, "Test Toggle");
                });
                ui.add(egui::Slider::new(&mut 0.5f32, 0.0..=1.0).text("Test Slider"));
            });
        });
    }
    
    pub fn is_open(&self) -> bool {
        self.open
    }
}