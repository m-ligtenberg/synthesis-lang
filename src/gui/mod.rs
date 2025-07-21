use egui::*;

pub struct SynthesisGui {
    open: bool,
}

impl Default for SynthesisGui {
    fn default() -> Self {
        Self { open: true }
    }
}

impl SynthesisGui {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn show(&mut self, ctx: &Context) {
        Window::new("Synthesis Editor")
            .open(&mut self.open)
            .default_size([400.0, 300.0])
            .show(ctx, |ui| {
                ui.heading("Synthesis Creative Programming Language");
                ui.separator();
                
                ui.label("Welcome to Synthesis!");
                ui.label("A creative programming language for artists and musicians.");
                
                if ui.button("Run Script").clicked() {
                    println!("Run button clicked");
                }
                
                if ui.button("Load Example").clicked() {
                    println!("Load example button clicked");
                }
            });
    }
    
    pub fn is_open(&self) -> bool {
        self.open
    }
}