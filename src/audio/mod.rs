pub mod input;
pub mod analysis;
pub mod effects;
pub mod processor;
pub mod midi;

// Re-export specific items to avoid naming conflicts
pub use input::*;
pub use analysis::*;
pub use midi::*;

// From effects module
pub use effects::{AudioEffect as EffectsAudioEffect, Distortion as EffectsDistortion};

// From processor module  
pub use processor::{AudioProcessor, PitchDetector};