pub mod parser;
pub mod runtime;
pub mod graphics;
pub mod audio;
pub mod modules;
pub mod gui;
pub mod hardware;

pub use parser::*;
pub use runtime::*;

pub type Result<T> = anyhow::Result<T>;