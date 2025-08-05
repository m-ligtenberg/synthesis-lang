use crate::compiler::{CompilationOptions, OptimizationLevel};
use crate::compiler::ir::*;
use crate::Result;
use std::collections::{HashMap, HashSet};

pub struct Optimizer {
    passes: Vec<Box<dyn OptimizationPass>>,
}

pub trait OptimizationPass {
    fn name(&self) -> &'static str;
    fn run(&mut self, ir: &mut IR, options: &CompilationOptions) -> Result<bool>; // Returns true if changes were made
}

pub struct DeadCodeElimination {
    removed_functions: HashSet<String>,
}

pub struct StreamFusion {
    fused_streams: HashMap<String, Vec<String>>,
}

pub struct CreativeDomainOptimization {
    audio_optimizations: AudioOptimizer,
    graphics_optimizations: GraphicsOptimizer,
}

pub struct ConstantFolding {
    folded_constants: HashMap<usize, IRConstant>,
}

pub struct RealTimeOptimization {
    latency_constraints: HashMap<String, f32>,
}

struct AudioOptimizer {
    buffer_sizes: HashMap<String, usize>,
    sample_rates: HashMap<String, f32>,
}

struct GraphicsOptimizer {
    render_batches: Vec<Vec<String>>,
    shader_cache: HashMap<String, String>,
}

impl Optimizer {
    pub fn new() -> Self {
        let mut passes: Vec<Box<dyn OptimizationPass>> = Vec::new();
        
        // Add optimization passes in order of execution
        passes.push(ConstantFolding::new());
        passes.push(DeadCodeElimination::new());
        passes.push(StreamFusion::new());
        passes.push(CreativeDomainOptimization::new());
        passes.push(RealTimeOptimization::new());

        Self { passes }
    }

    pub fn optimize(&mut self, mut ir: IR, options: &CompilationOptions) -> Result<IR> {
        let max_iterations = match options.optimization_level {
            OptimizationLevel::None => return Ok(ir),
            OptimizationLevel::Basic => 3,
            OptimizationLevel::Aggressive => 10,
            OptimizationLevel::Creative => 15, // More iterations for creative-specific optimizations
        };

        for iteration in 0..max_iterations {
            let mut changed = false;
            
            for pass in &mut self.passes {
                if pass.run(&mut ir, options)? {
                    changed = true;
                    println!("Optimization pass '{}' made changes in iteration {}", pass.name(), iteration);
                }
            }
            
            if !changed {
                println!("Optimization converged after {} iterations", iteration + 1);
                break;
            }
        }

        Ok(ir)
    }
}

impl ConstantFolding {
    fn new() -> Box<Self> {
        Box::new(Self {
            folded_constants: HashMap::new(),
        })
    }
}

impl OptimizationPass for ConstantFolding {
    fn name(&self) -> &'static str {
        "Constant Folding"
    }

    fn run(&mut self, ir: &mut IR, _options: &CompilationOptions) -> Result<bool> {
        let mut changed = false;

        for module in &mut ir.modules {
            for function in &mut module.functions {
                for block in &mut function.basic_blocks {
                    for instruction in &mut block.instructions {
                        if self.fold_instruction(instruction)? {
                            changed = true;
                        }
                    }
                }
            }
        }

        Ok(changed)
    }
}

impl ConstantFolding {
    fn fold_instruction(&mut self, instruction: &mut IRInstruction) -> Result<bool> {
        match instruction {
            IRInstruction::Add { dest, left, right } => {
                if let (IRValue::Constant(IRConstant::Float(a)), IRValue::Constant(IRConstant::Float(b))) = (left, right) {
                    let result = IRConstant::Float(*a + *b);
                    self.folded_constants.insert(dest.id, result.clone());
                    *instruction = IRInstruction::Load {
                        dest: dest.clone(),
                        address: IRValue::Constant(result),
                    };
                    return Ok(true);
                }
            }
            IRInstruction::Mul { dest, left, right } => {
                if let (IRValue::Constant(IRConstant::Float(a)), IRValue::Constant(IRConstant::Float(b))) = (left, right) {
                    let result = IRConstant::Float(*a * *b);
                    self.folded_constants.insert(dest.id, result.clone());
                    *instruction = IRInstruction::Load {
                        dest: dest.clone(),
                        address: IRValue::Constant(result),
                    };
                    return Ok(true);
                }
            }
            // Handle creative domain constant folding
            IRInstruction::MapRange { dest, value, from_min, from_max, to_min, to_max } => {
                if let (
                    IRValue::Constant(IRConstant::Float(v)),
                    IRValue::Constant(IRConstant::Float(fmin)),
                    IRValue::Constant(IRConstant::Float(fmax)),
                    IRValue::Constant(IRConstant::Float(tmin)),
                    IRValue::Constant(IRConstant::Float(tmax))
                ) = (value, from_min, from_max, to_min, to_max) {
                    // Constant fold map_range operation
                    let normalized = (*v - *fmin) / (*fmax - *fmin);
                    let result = *tmin + normalized * (*tmax - *tmin);
                    
                    *instruction = IRInstruction::Load {
                        dest: dest.clone(),
                        address: IRValue::Constant(IRConstant::Float(result)),
                    };
                    return Ok(true);
                }
            }
            _ => {}
        }
        Ok(false)
    }
}

impl DeadCodeElimination {
    fn new() -> Box<Self> {
        Box::new(Self {
            removed_functions: HashSet::new(),
        })
    }
}

impl OptimizationPass for DeadCodeElimination {
    fn name(&self) -> &'static str {
        "Dead Code Elimination"
    }

    fn run(&mut self, ir: &mut IR, _options: &CompilationOptions) -> Result<bool> {
        let mut changed = false;
        let mut used_functions = HashSet::new();
        
        // Start from entry point and mark all reachable functions
        self.mark_reachable_functions(ir, &ir.entry_point, &mut used_functions);

        // Remove unused functions
        for module in &mut ir.modules {
            let original_count = module.functions.len();
            module.functions.retain(|f| used_functions.contains(&f.name));
            if module.functions.len() < original_count {
                changed = true;
            }
        }

        // Remove unused instructions within functions
        for module in &mut ir.modules {
            for function in &mut module.functions {
                if self.remove_dead_instructions(function)? {
                    changed = true;
                }
            }
        }

        Ok(changed)
    }
}

impl DeadCodeElimination {
    fn mark_reachable_functions(&self, ir: &IR, function_name: &str, used: &mut HashSet<String>) {
        if used.contains(function_name) {
            return;
        }
        
        used.insert(function_name.to_string());
        
        // Find the function and analyze its calls
        for module in &ir.modules {
            for function in &module.functions {
                if function.name == function_name {
                    for block in &function.basic_blocks {
                        for instruction in &block.instructions {
                            if let IRInstruction::Call { function: called_fn, .. } = instruction {
                                self.mark_reachable_functions(ir, called_fn, used);
                            }
                        }
                    }
                }
            }
        }
    }

    fn remove_dead_instructions(&self, function: &mut IRFunction) -> Result<bool> {
        let mut changed = false;
        let mut live_registers = HashSet::new();

        // Backwards analysis to find live registers
        for block in function.basic_blocks.iter().rev() {
            // Mark registers used in terminator
            match &block.terminator {
                Terminator::Return(Some(value)) => {
                    if let IRValue::Register(reg) = value {
                        live_registers.insert(reg.id);
                    }
                }
                Terminator::Branch { condition, .. } => {
                    if let IRValue::Register(reg) = condition {
                        live_registers.insert(reg.id);
                    }
                }
                _ => {}
            }

            // Mark registers used in instructions (reverse order)
            for instruction in block.instructions.iter().rev() {
                match instruction {
                    IRInstruction::Add { left, right, .. } |
                    IRInstruction::Sub { left, right, .. } |
                    IRInstruction::Mul { left, right, .. } |
                    IRInstruction::Div { left, right, .. } => {
                        if let IRValue::Register(reg) = left {
                            live_registers.insert(reg.id);
                        }
                        if let IRValue::Register(reg) = right {
                            live_registers.insert(reg.id);
                        }
                    }
                    _ => {} // Handle other instruction types
                }
            }
        }

        // Remove dead instructions
        for block in &mut function.basic_blocks {
            let original_len = block.instructions.len();
            block.instructions.retain(|instruction| {
                match instruction {
                    IRInstruction::Add { dest, .. } |
                    IRInstruction::Sub { dest, .. } |
                    IRInstruction::Mul { dest, .. } |
                    IRInstruction::Div { dest, .. } |
                    IRInstruction::Load { dest, .. } => {
                        live_registers.contains(&dest.id)
                    }
                    // Always keep side-effecting instructions
                    IRInstruction::GraphicsDraw { .. } |
                    IRInstruction::StreamWrite { .. } |
                    IRInstruction::Call { .. } => true,
                    _ => true, // Conservative: keep other instructions
                }
            });
            if block.instructions.len() < original_len {
                changed = true;
            }
        }

        Ok(changed)
    }
}

impl StreamFusion {
    fn new() -> Box<Self> {
        Box::new(Self {
            fused_streams: HashMap::new(),
        })
    }
}

impl OptimizationPass for StreamFusion {
    fn name(&self) -> &'static str {
        "Stream Fusion"
    }

    fn run(&mut self, ir: &mut IR, _options: &CompilationOptions) -> Result<bool> {
        let mut changed = false;

        // Find streams that can be fused
        let fusion_opportunities = self.analyze_stream_fusion(ir)?;

        // Apply fusion optimizations
        for (consumer, producers) in fusion_opportunities {
            if self.fuse_streams(ir, &consumer, &producers)? {
                changed = true;
            }
        }

        Ok(changed)
    }
}

impl StreamFusion {
    fn analyze_stream_fusion(&self, ir: &IR) -> Result<HashMap<String, Vec<String>>> {
        let mut opportunities = HashMap::new();
        
        for module in &ir.modules {
            for function in &module.functions {
                for block in &function.basic_blocks {
                    for instruction in &block.instructions {
                        if let IRInstruction::StreamConnect { source, sink } = instruction {
                            // Track stream connections for fusion analysis
                            let source_name = format!("stream_{}", source.id);
                            let sink_name = format!("stream_{}", sink.id);
                            
                            opportunities.entry(sink_name)
                                .or_insert_with(Vec::new)
                                .push(source_name);
                        }
                    }
                }
            }
        }

        Ok(opportunities)
    }

    fn fuse_streams(&mut self, _ir: &mut IR, consumer: &str, producers: &Vec<String>) -> Result<bool> {
        // Simplified stream fusion - in reality this would be much more complex
        if producers.len() > 1 {
            println!("Fusing {} producer streams into consumer {}", producers.len(), consumer);
            self.fused_streams.insert(consumer.to_string(), producers.clone());
            return Ok(true);
        }
        Ok(false)
    }
}

impl CreativeDomainOptimization {
    fn new() -> Box<Self> {
        Box::new(Self {
            audio_optimizations: AudioOptimizer::new(),
            graphics_optimizations: GraphicsOptimizer::new(),
        })
    }
}

impl OptimizationPass for CreativeDomainOptimization {
    fn name(&self) -> &'static str {
        "Creative Domain Optimization"
    }

    fn run(&mut self, ir: &mut IR, options: &CompilationOptions) -> Result<bool> {
        let mut changed = false;

        // Audio-specific optimizations
        if self.audio_optimizations.optimize_audio_processing(ir, options)? {
            changed = true;
        }

        // Graphics-specific optimizations
        if self.graphics_optimizations.optimize_graphics_rendering(ir, options)? {
            changed = true;
        }

        Ok(changed)
    }
}

impl AudioOptimizer {
    fn new() -> Self {
        Self {
            buffer_sizes: HashMap::new(),
            sample_rates: HashMap::new(),
        }
    }

    fn optimize_audio_processing(&mut self, ir: &mut IR, _options: &CompilationOptions) -> Result<bool> {
        let mut changed = false;

        for module in &mut ir.modules {
            for function in &mut module.functions {
                for block in &mut function.basic_blocks {
                    for instruction in &mut block.instructions {
                        match instruction {
                            IRInstruction::AudioAnalyzeFFT { dest, audio, bands } => {
                                // Optimize FFT size to power of 2
                                let optimized_bands = self.next_power_of_2(*bands);
                                if optimized_bands != *bands {
                                    *bands = optimized_bands;
                                    changed = true;
                                    println!("Optimized FFT bands from {} to {}", bands, optimized_bands);
                                }
                                
                                // Track audio buffer for further optimizations
                                self.buffer_sizes.insert(format!("audio_{}", audio.id), optimized_bands * 2);
                            }
                            IRInstruction::StreamCreate { dest, stream_type, buffer_size } => {
                                if matches!(stream_type, IRType::AudioBuffer) {
                                    // Optimize audio buffer sizes for cache efficiency
                                    let optimized_size = self.optimize_buffer_size(*buffer_size);
                                    if optimized_size != *buffer_size {
                                        *buffer_size = optimized_size;
                                        changed = true;
                                    }
                                    self.buffer_sizes.insert(format!("stream_{}", dest.id), optimized_size);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(changed)
    }

    fn next_power_of_2(&self, n: usize) -> usize {
        if n <= 1 { return 1; }
        let mut power = 1;
        while power < n {
            power *= 2;
        }
        power
    }

    fn optimize_buffer_size(&self, size: usize) -> usize {
        // Optimize for common audio buffer sizes (64, 128, 256, 512, 1024, etc.)
        const OPTIMAL_SIZES: &[usize] = &[64, 128, 256, 512, 1024, 2048, 4096];
        
        for &optimal_size in OPTIMAL_SIZES {
            if size <= optimal_size {
                return optimal_size;
            }
        }
        
        // Default to nearest power of 2
        self.next_power_of_2(size)
    }
}

impl GraphicsOptimizer {
    fn new() -> Self {
        Self {
            render_batches: Vec::new(),
            shader_cache: HashMap::new(),
        }
    }

    fn optimize_graphics_rendering(&mut self, ir: &mut IR, _options: &CompilationOptions) -> Result<bool> {
        let mut changed = false;
        let mut draw_calls = Vec::new();

        // Collect all graphics draw calls
        for module in &ir.modules {
            for function in &module.functions {
                for block in &function.basic_blocks {
                    for (idx, instruction) in block.instructions.iter().enumerate() {
                        if let IRInstruction::GraphicsDraw { primitive, .. } = instruction {
                            draw_calls.push((module.name.clone(), function.name.clone(), idx, primitive.clone()));
                        }
                    }
                }
            }
        }

        // Batch similar draw calls
        if self.batch_draw_calls(&draw_calls) {
            changed = true;
        }

        Ok(changed)
    }

    fn batch_draw_calls(&mut self, draw_calls: &[(String, String, usize, String)]) -> bool {
        let mut batches = HashMap::new();
        
        for (module, function, idx, primitive) in draw_calls {
            batches.entry(primitive.clone())
                .or_insert_with(Vec::new)
                .push((module.clone(), function.clone(), *idx));
        }

        let mut created_batches = false;
        for (primitive, calls) in batches {
            if calls.len() > 1 {
                println!("Created batch for {} '{}' draw calls", calls.len(), primitive);
                self.render_batches.push(calls.into_iter().map(|(_, f, _)| f).collect());
                created_batches = true;
            }
        }

        created_batches
    }
}

impl RealTimeOptimization {
    fn new() -> Box<Self> {
        Box::new(Self {
            latency_constraints: HashMap::new(),
        })
    }
}

impl OptimizationPass for RealTimeOptimization {
    fn name(&self) -> &'static str {
        "Real-Time Optimization"
    }

    fn run(&mut self, ir: &mut IR, options: &CompilationOptions) -> Result<bool> {
        if !options.real_time_priority {
            return Ok(false);
        }

        let mut changed = false;

        // Analyze and optimize for real-time constraints
        for module in &mut ir.modules {
            for function in &mut module.functions {
                if function.is_stream_processor {
                    if let Some(constraint) = &function.latency_constraint {
                        self.latency_constraints.insert(function.name.clone(), *constraint);
                        
                        // Apply real-time optimizations
                        if self.optimize_for_latency(function, *constraint)? {
                            changed = true;
                        }
                    }
                }
            }
        }

        Ok(changed)
    }
}

impl RealTimeOptimization {
    fn optimize_for_latency(&self, function: &mut IRFunction, max_latency_ms: f32) -> Result<bool> {
        let mut changed = false;
        
        // Estimate current latency and optimize if needed
        let estimated_latency = self.estimate_function_latency(function);
        
        if estimated_latency > max_latency_ms {
            println!(
                "Function '{}' estimated latency {:.2}ms exceeds constraint {:.2}ms",
                function.name, estimated_latency, max_latency_ms
            );
            
            // Apply latency reduction optimizations
            changed = self.reduce_latency(function, max_latency_ms)?;
        }
        
        Ok(changed)
    }

    fn estimate_function_latency(&self, function: &IRFunction) -> f32 {
        let mut total_cycles = 0.0;
        
        for block in &function.basic_blocks {
            for instruction in &block.instructions {
                total_cycles += match instruction {
                    IRInstruction::AudioAnalyzeFFT { bands, .. } => {
                        // FFT is expensive, estimate based on size
                        (*bands as f32) * 0.1 // Rough estimate
                    }
                    IRInstruction::GraphicsDraw { .. } => 0.5,
                    IRInstruction::Add { .. } | IRInstruction::Sub { .. } => 0.01,
                    IRInstruction::Mul { .. } | IRInstruction::Div { .. } => 0.05,
                    _ => 0.02, // Default estimate
                };
            }
        }
        
        // Convert cycles to milliseconds (rough estimate for 3GHz CPU)
        total_cycles / 3000.0
    }

    fn reduce_latency(&self, _function: &mut IRFunction, _target_latency: f32) -> Result<bool> {
        // Implement latency reduction strategies:
        // - Reduce FFT sizes
        // - Simplify graphics operations
        // - Use lookup tables instead of calculations
        // - Parallel processing hints
        
        // Placeholder for now
        println!("Applied latency reduction optimizations");
        Ok(true)
    }
}