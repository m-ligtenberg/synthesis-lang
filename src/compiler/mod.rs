pub mod codegen;
pub mod ir;
pub mod optimizer;
pub mod backend;

use crate::parser::ast::Program;
use crate::Result;

pub struct Compiler {
    pub ir_generator: ir::IRGenerator,
    pub optimizer: optimizer::Optimizer,
    pub wasm_backend: backend::WasmBackend,
    pub native_backend: backend::NativeBackend,
}

#[derive(Debug, Clone)]
pub enum CompilationTarget {
    WebAssembly,
    Native(NativeTarget),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NativeTarget {
    X86_64Linux,
    X86_64Windows,
    X86_64MacOS,
    AArch64Linux,
    AArch64MacOS,
}

pub struct CompilationOptions {
    pub target: CompilationTarget,
    pub optimization_level: OptimizationLevel,
    pub include_debug_info: bool,
    pub stream_buffer_size: usize,
    pub real_time_priority: bool,
}

#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
    Creative, // Special optimizations for creative coding patterns
}

impl Default for CompilationOptions {
    fn default() -> Self {
        Self {
            target: CompilationTarget::WebAssembly,
            optimization_level: OptimizationLevel::Basic,
            include_debug_info: true,
            stream_buffer_size: 1024,
            real_time_priority: true,
        }
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            ir_generator: ir::IRGenerator::new(),
            optimizer: optimizer::Optimizer::new(),
            wasm_backend: backend::WasmBackend::new(),
            native_backend: backend::NativeBackend::new(),
        }
    }

    pub fn compile(&mut self, program: &Program, options: CompilationOptions) -> Result<CompiledArtifact> {
        // Step 1: Generate Intermediate Representation
        let ir = self.ir_generator.generate(program)?;
        
        // Step 2: Optimize IR
        let optimized_ir = self.optimizer.optimize(ir, &options)?;
        
        // Step 3: Generate target code
        let artifact = match options.target {
            CompilationTarget::WebAssembly => {
                self.wasm_backend.generate(&optimized_ir, &options)?
            }
            CompilationTarget::Native(ref target) => {
                self.native_backend.generate(&optimized_ir, target.clone(), &options)?
            }
        };

        Ok(artifact)
    }
}

#[derive(Debug)]
pub struct CompiledArtifact {
    pub bytecode: Vec<u8>,
    pub metadata: ArtifactMetadata,
}

#[derive(Debug)]
pub struct ArtifactMetadata {
    pub target: CompilationTarget,
    pub entry_point: String,
    pub dependencies: Vec<String>,
    pub stream_interfaces: Vec<StreamInterface>,
    pub exported_functions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StreamInterface {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
    pub buffer_size: usize,
    pub latency_ms: f32,
}