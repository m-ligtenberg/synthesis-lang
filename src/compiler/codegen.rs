use crate::compiler::ir::*;
use crate::compiler::{CompilationOptions, CompilationTarget};
use crate::errors::{SynthesisError, ErrorKind};
use crate::Result;

/// Code generation utilities for different target platforms
pub struct CodeGenerator {
    wasm_generator: WasmCodeGen,
    native_generator: NativeCodeGen,
}

pub struct WasmCodeGen {
    function_index: u32,
    import_index: u32,
    type_index: u32,
}

pub struct NativeCodeGen {
    register_allocator: RegisterAllocator,
    instruction_selector: InstructionSelector,
}

struct RegisterAllocator {
    virtual_registers: std::collections::HashMap<usize, PhysicalRegister>,
    free_registers: Vec<PhysicalRegister>,
}

#[derive(Debug, Clone)]
enum PhysicalRegister {
    // x86_64 registers
    RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP,
    R8, R9, R10, R11, R12, R13, R14, R15,
    // SSE/AVX registers for SIMD operations
    XMM0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6, XMM7,
    XMM8, XMM9, XMM10, XMM11, XMM12, XMM13, XMM14, XMM15,
}

struct InstructionSelector {
    target_features: Vec<String>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            wasm_generator: WasmCodeGen::new(),
            native_generator: NativeCodeGen::new(),
        }
    }

    pub fn generate_code(&mut self, ir: &IR, options: &CompilationOptions) -> Result<Vec<u8>> {
        match &options.target {
            CompilationTarget::WebAssembly => {
                self.wasm_generator.generate(ir, options)
            }
            CompilationTarget::Native(target) => {
                self.native_generator.generate(ir, target, options)
            }
        }
    }
}

impl WasmCodeGen {
    fn new() -> Self {
        Self {
            function_index: 0,
            import_index: 0,
            type_index: 0,
        }
    }

    fn generate(&mut self, ir: &IR, _options: &CompilationOptions) -> Result<Vec<u8>> {
        let mut module = WasmModule::new();

        // Generate type section
        self.generate_types(&mut module, ir)?;

        // Generate import section
        self.generate_imports(&mut module, ir)?;

        // Generate function section
        self.generate_functions(&mut module, ir)?;

        // Generate export section
        self.generate_exports(&mut module, ir)?;

        // Generate code section
        self.generate_code_section(&mut module, ir)?;

        // Encode to WebAssembly binary format
        module.encode()
    }

    fn generate_types(&mut self, module: &mut WasmModule, ir: &IR) -> Result<()> {
        // Generate function types for all functions
        for ir_module in &ir.modules {
            for function in &ir_module.functions {
                let param_types = function.parameters.iter()
                    .map(|p| self.ir_type_to_wasm(&p.ir_type))
                    .collect::<Result<Vec<_>>>()?;

                let return_types = match &function.return_type {
                    IRType::Void => vec![],
                    t => vec![self.ir_type_to_wasm(t)?],
                };

                module.add_type(WasmFunctionType {
                    params: param_types,
                    results: return_types,
                });
                self.type_index += 1;
            }
        }

        Ok(())
    }

    fn generate_imports(&mut self, module: &mut WasmModule, _ir: &IR) -> Result<()> {
        // Import runtime functions for synthesis operations
        let synthesis_imports = vec![
            ("audio_input", vec![], vec![WasmValueType::I32]),
            ("audio_fft", vec![WasmValueType::I32, WasmValueType::I32], vec![WasmValueType::I32]),
            ("graphics_clear", vec![WasmValueType::F32, WasmValueType::F32, WasmValueType::F32], vec![]),
            ("graphics_plasma", vec![WasmValueType::F32, WasmValueType::I32], vec![]),
            ("stream_create", vec![WasmValueType::I32, WasmValueType::I32], vec![WasmValueType::I32]),
            ("stream_read", vec![WasmValueType::I32], vec![WasmValueType::F32]),
            ("stream_write", vec![WasmValueType::I32, WasmValueType::F32], vec![]),
        ];

        for (name, params, results) in synthesis_imports {
            module.add_import(WasmImport {
                module_name: "synthesis".to_string(),
                field_name: name.to_string(),
                descriptor: WasmImportDescriptor::Function(WasmFunctionType {
                    params,
                    results,
                }),
            });
            self.import_index += 1;
        }

        // Import memory
        module.add_import(WasmImport {
            module_name: "synthesis".to_string(),
            field_name: "memory".to_string(),
            descriptor: WasmImportDescriptor::Memory(WasmMemoryType {
                minimum: 1,
                maximum: Some(256),
            }),
        });

        Ok(())
    }

    fn generate_functions(&mut self, module: &mut WasmModule, ir: &IR) -> Result<()> {
        for ir_module in &ir.modules {
            for function in &ir_module.functions {
                let type_idx = self.function_index; // Simplified mapping
                module.add_function(type_idx);
                self.function_index += 1;
            }
        }
        Ok(())
    }

    fn generate_exports(&mut self, module: &mut WasmModule, ir: &IR) -> Result<()> {
        // Export the main function
        let main_function_idx = self.import_index; // After imports
        module.add_export(WasmExport {
            name: ir.entry_point.clone(),
            descriptor: WasmExportDescriptor::Function(main_function_idx),
        });
        Ok(())
    }

    fn generate_code_section(&mut self, module: &mut WasmModule, ir: &IR) -> Result<()> {
        for ir_module in &ir.modules {
            for function in &ir_module.functions {
                let code = self.generate_function_code(function)?;
                module.add_code(code);
            }
        }
        Ok(())
    }

    fn generate_function_code(&self, function: &IRFunction) -> Result<WasmFunctionCode> {
        let mut locals = Vec::new();
        let mut instructions = Vec::new();

        // Collect locals from function analysis
        let mut local_count = std::collections::HashMap::new();
        for block in &function.basic_blocks {
            for instruction in &block.instructions {
                self.collect_locals_from_instruction(instruction, &mut local_count)?;
            }
        }

        // Convert local counts to WebAssembly format
        for (wasm_type, count) in local_count {
            locals.push(WasmLocal { count, value_type: wasm_type });
        }

        // Generate instructions for each basic block
        for block in &function.basic_blocks {
            for instruction in &block.instructions {
                self.generate_wasm_instructions(instruction, &mut instructions)?;
            }

            // Generate terminator
            self.generate_terminator(&block.terminator, &mut instructions)?;
        }

        Ok(WasmFunctionCode { locals, body: instructions })
    }

    fn collect_locals_from_instruction(
        &self,
        instruction: &IRInstruction,
        local_count: &mut std::collections::HashMap<WasmValueType, u32>,
    ) -> Result<()> {
        match instruction {
            IRInstruction::Add { dest, .. } |
            IRInstruction::Sub { dest, .. } |
            IRInstruction::Mul { dest, .. } |
            IRInstruction::Div { dest, .. } |
            IRInstruction::Load { dest, .. } => {
                let wasm_type = self.ir_type_to_wasm(&dest.ir_type)?;
                *local_count.entry(wasm_type).or_insert(0) += 1;
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_wasm_instructions(
        &self,
        instruction: &IRInstruction,
        wasm_instructions: &mut Vec<WasmInstruction>,
    ) -> Result<()> {
        match instruction {
            IRInstruction::Add { dest: _, left, right } => {
                self.generate_value_load(left, wasm_instructions)?;
                self.generate_value_load(right, wasm_instructions)?;
                
                // Determine the operation type based on operands
                match (left, right) {
                    (IRValue::Constant(IRConstant::Float(_)), _) |
                    (_, IRValue::Constant(IRConstant::Float(_))) => {
                        wasm_instructions.push(WasmInstruction::F64Add);
                    }
                    _ => {
                        wasm_instructions.push(WasmInstruction::I32Add);
                    }
                }
                
                // Store result (simplified - would need proper local tracking)
                wasm_instructions.push(WasmInstruction::LocalSet(0));
            }

            IRInstruction::AudioAnalyzeFFT { dest: _, audio: _, bands } => {
                // Load audio buffer (simplified)
                wasm_instructions.push(WasmInstruction::LocalGet(0)); // audio buffer
                wasm_instructions.push(WasmInstruction::I32Const(*bands as i32));
                wasm_instructions.push(WasmInstruction::Call(1)); // audio_fft import
                wasm_instructions.push(WasmInstruction::LocalSet(1)); // store result
            }

            IRInstruction::GraphicsDraw { primitive, params } => {
                match primitive.as_str() {
                    "clear" => {
                        // Load color parameters
                        for param in params.iter().take(3) {
                            self.generate_value_load(param, wasm_instructions)?;
                        }
                        wasm_instructions.push(WasmInstruction::Call(2)); // graphics_clear import
                    }
                    "plasma" => {
                        for param in params.iter().take(2) {
                            self.generate_value_load(param, wasm_instructions)?;
                        }
                        wasm_instructions.push(WasmInstruction::Call(3)); // graphics_plasma import
                    }
                    _ => return Err(SynthesisError::new(
                        ErrorKind::CodeGenerationFailed,
                        format!("Graphics primitive '{}' is not supported in this compilation target", primitive)
                    )
                    .with_suggestion("Use supported graphics functions for your target platform")
                    .with_docs("https://synthesis-lang.org/docs/graphics#compatibility")),
                }
            }

            IRInstruction::StreamCreate { dest: _, stream_type: _, buffer_size } => {
                wasm_instructions.push(WasmInstruction::I32Const(0)); // stream type
                wasm_instructions.push(WasmInstruction::I32Const(*buffer_size as i32));
                wasm_instructions.push(WasmInstruction::Call(4)); // stream_create import
                wasm_instructions.push(WasmInstruction::LocalSet(2)); // store stream handle
            }

            _ => {
                return Err(SynthesisError::new(
                    ErrorKind::CodeGenerationFailed,
                    "This operation is not supported when compiling to WebAssembly"
                )
                .with_suggestion("Some advanced features are only available in native compilation")
                .with_suggestion("Try compiling to native target instead")
                .with_docs("https://synthesis-lang.org/docs/compilation#webassembly-limitations"));
            }
        }

        Ok(())
    }

    fn generate_value_load(
        &self,
        value: &IRValue,
        wasm_instructions: &mut Vec<WasmInstruction>,
    ) -> Result<()> {
        match value {
            IRValue::Constant(constant) => {
                match constant {
                    IRConstant::Integer(i) => wasm_instructions.push(WasmInstruction::I32Const(*i as i32)),
                    IRConstant::Float(f) => wasm_instructions.push(WasmInstruction::F64Const(*f)),
                    IRConstant::Boolean(b) => wasm_instructions.push(WasmInstruction::I32Const(if *b { 1 } else { 0 })),
                    _ => return Err(SynthesisError::new(
                        ErrorKind::CodeGenerationFailed,
                        "This type of constant is not supported"
                    )
                    .with_suggestion("Use supported constant types: numbers, strings, booleans")
                    .with_docs("https://synthesis-lang.org/docs/types#constants")),
                }
            }
            IRValue::Register(reg) => {
                // Simplified - would need proper register to local mapping
                wasm_instructions.push(WasmInstruction::LocalGet(reg.id as u32));
            }
            IRValue::Global(_name) => {
                return Err(SynthesisError::new(
                    ErrorKind::CodeGenerationFailed,
                    "Global variables are not yet supported"
                )
                .with_suggestion("Use function parameters or local variables instead")
                .with_suggestion("This feature is planned for a future release"));
            }
        }
        Ok(())
    }

    fn generate_terminator(
        &self,
        terminator: &Terminator,
        wasm_instructions: &mut Vec<WasmInstruction>,
    ) -> Result<()> {
        match terminator {
            Terminator::Return(value) => {
                if let Some(val) = value {
                    self.generate_value_load(val, wasm_instructions)?;
                }
                wasm_instructions.push(WasmInstruction::Return);
            }
            Terminator::StreamLoop => {
                // Generate infinite loop for stream processing
                wasm_instructions.push(WasmInstruction::Loop(WasmBlockType::Empty));
                wasm_instructions.push(WasmInstruction::Br(0)); // Branch back to loop start
                wasm_instructions.push(WasmInstruction::End);
            }
            _ => {
                // Other terminators would be implemented here
            }
        }
        Ok(())
    }

    fn ir_type_to_wasm(&self, ir_type: &IRType) -> Result<WasmValueType> {
        match ir_type {
            IRType::Integer | IRType::Boolean => Ok(WasmValueType::I32),
            IRType::Float | IRType::AudioFrequency | IRType::Percentage => Ok(WasmValueType::F64),
            IRType::AudioSample => Ok(WasmValueType::F32),
            IRType::AudioBuffer | IRType::Stream(_) => Ok(WasmValueType::I32), // Pointer/handle
            _ => Ok(WasmValueType::I32), // Default for complex types
        }
    }
}

impl NativeCodeGen {
    fn new() -> Self {
        Self {
            register_allocator: RegisterAllocator::new(),
            instruction_selector: InstructionSelector::new(),
        }
    }

    fn generate(
        &mut self,
        ir: &IR,
        _target: &crate::compiler::NativeTarget,
        _options: &CompilationOptions,
    ) -> Result<Vec<u8>> {
        let mut machine_code = Vec::new();

        // Generate native code for each module
        for ir_module in &ir.modules {
            for function in &ir_module.functions {
                let function_code = self.generate_function(function)?;
                machine_code.extend(function_code);
            }
        }

        Ok(machine_code)
    }

    fn generate_function(&mut self, function: &IRFunction) -> Result<Vec<u8>> {
        let mut code = Vec::new();

        // Function prologue
        code.extend(self.generate_prologue(function)?);

        // Allocate registers for the function
        self.register_allocator.allocate_for_function(function)?;

        // Generate code for each basic block
        for block in &function.basic_blocks {
            code.extend(self.generate_basic_block(block)?);
        }

        // Function epilogue
        code.extend(self.generate_epilogue(function)?);

        Ok(code)
    }

    fn generate_prologue(&self, _function: &IRFunction) -> Result<Vec<u8>> {
        // x86_64 function prologue
        Ok(vec![
            0x55,       // push rbp
            0x48, 0x89, 0xE5, // mov rbp, rsp
        ])
    }

    fn generate_epilogue(&self, _function: &IRFunction) -> Result<Vec<u8>> {
        // x86_64 function epilogue
        Ok(vec![
            0x48, 0x89, 0xEC, // mov rsp, rbp
            0x5D,             // pop rbp
            0xC3,             // ret
        ])
    }

    fn generate_basic_block(&self, block: &BasicBlock) -> Result<Vec<u8>> {
        let mut code = Vec::new();

        for instruction in &block.instructions {
            code.extend(self.generate_native_instruction(instruction)?);
        }

        // Generate terminator
        code.extend(self.generate_native_terminator(&block.terminator)?);

        Ok(code)
    }

    fn generate_native_instruction(&self, instruction: &IRInstruction) -> Result<Vec<u8>> {
        match instruction {
            IRInstruction::Add { dest: _, left: _, right: _ } => {
                // Simplified x86_64 addition
                Ok(vec![0x48, 0x01, 0xC0]) // add rax, rax
            }
            IRInstruction::Call { function, .. } => {
                if function.starts_with("Audio_") || function.starts_with("Graphics_") {
                    // Generate call to runtime function
                    Ok(vec![0xE8, 0x00, 0x00, 0x00, 0x00]) // call rel32 (placeholder)
                } else {
                    Ok(vec![0xE8, 0x00, 0x00, 0x00, 0x00]) // call rel32 (placeholder)
                }
            }
            _ => {
                // Placeholder for other instructions
                Ok(vec![0x90]) // nop
            }
        }
    }

    fn generate_native_terminator(&self, terminator: &Terminator) -> Result<Vec<u8>> {
        match terminator {
            Terminator::Return(_) => Ok(vec![0xC3]), // ret
            Terminator::StreamLoop => {
                // Generate infinite loop
                Ok(vec![0xEB, 0xFE]) // jmp -2 (infinite loop)
            }
            _ => Ok(vec![]), // Other terminators
        }
    }
}

impl RegisterAllocator {
    fn new() -> Self {
        Self {
            virtual_registers: std::collections::HashMap::new(),
            free_registers: vec![
                PhysicalRegister::RAX, PhysicalRegister::RBX, PhysicalRegister::RCX,
                PhysicalRegister::RDX, PhysicalRegister::RSI, PhysicalRegister::RDI,
            ],
        }
    }

    fn allocate_for_function(&mut self, function: &IRFunction) -> Result<()> {
        // Simple register allocation - assign physical registers to virtual ones
        let mut virtual_regs = std::collections::HashSet::new();

        // Collect all virtual registers used
        for block in &function.basic_blocks {
            for instruction in &block.instructions {
                self.collect_virtual_registers(instruction, &mut virtual_regs);
            }
        }

        // Assign physical registers
        for (i, &virtual_reg) in virtual_regs.iter().enumerate() {
            if i < self.free_registers.len() {
                self.virtual_registers.insert(virtual_reg, self.free_registers[i].clone());
            } else {
                // Would need to spill to memory in a real implementation
                return Err(SynthesisError::new(
                    ErrorKind::OptimizationFailed,
                    "Your code is too complex for the available processor registers"
                )
                .with_suggestion("Try simplifying your expressions")
                .with_suggestion("Break complex calculations into smaller steps")
                .with_docs("https://synthesis-lang.org/docs/performance#optimization"));
            }
        }

        Ok(())
    }

    fn collect_virtual_registers(
        &self,
        instruction: &IRInstruction,
        virtual_regs: &mut std::collections::HashSet<usize>,
    ) {
        match instruction {
            IRInstruction::Add { dest, left, right } |
            IRInstruction::Sub { dest, left, right } |
            IRInstruction::Mul { dest, left, right } |
            IRInstruction::Div { dest, left, right } => {
                virtual_regs.insert(dest.id);
                if let IRValue::Register(reg) = left {
                    virtual_regs.insert(reg.id);
                }
                if let IRValue::Register(reg) = right {
                    virtual_regs.insert(reg.id);
                }
            }
            _ => {} // Handle other instruction types
        }
    }
}

impl InstructionSelector {
    fn new() -> Self {
        Self {
            target_features: vec!["sse2".to_string(), "sse4.1".to_string()],
        }
    }
}

// WebAssembly data structures for code generation
struct WasmModule {
    types: Vec<WasmFunctionType>,
    imports: Vec<WasmImport>,
    functions: Vec<u32>, // type indices
    exports: Vec<WasmExport>,
    code: Vec<WasmFunctionCode>,
}

#[derive(Debug, Clone)]
struct WasmFunctionType {
    params: Vec<WasmValueType>,
    results: Vec<WasmValueType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum WasmValueType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Debug)]
struct WasmImport {
    module_name: String,
    field_name: String,
    descriptor: WasmImportDescriptor,
}

#[derive(Debug)]
enum WasmImportDescriptor {
    Function(WasmFunctionType),
    Memory(WasmMemoryType),
}

#[derive(Debug)]
struct WasmMemoryType {
    minimum: u32,
    maximum: Option<u32>,
}

#[derive(Debug)]
struct WasmExport {
    name: String,
    descriptor: WasmExportDescriptor,
}

#[derive(Debug)]
enum WasmExportDescriptor {
    Function(u32),
}

#[derive(Debug)]
struct WasmFunctionCode {
    locals: Vec<WasmLocal>,
    body: Vec<WasmInstruction>,
}

#[derive(Debug)]
struct WasmLocal {
    count: u32,
    value_type: WasmValueType,
}

#[derive(Debug)]
enum WasmInstruction {
    I32Const(i32),
    F64Const(f64),
    LocalGet(u32),
    LocalSet(u32),
    I32Add,
    F64Add,
    Call(u32),
    Return,
    Loop(WasmBlockType),
    Br(u32),
    End,
}

#[derive(Debug)]
enum WasmBlockType {
    Empty,
    ValueType(WasmValueType),
}

impl WasmModule {
    fn new() -> Self {
        Self {
            types: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            exports: Vec::new(),
            code: Vec::new(),
        }
    }

    fn add_type(&mut self, function_type: WasmFunctionType) {
        self.types.push(function_type);
    }

    fn add_import(&mut self, import: WasmImport) {
        self.imports.push(import);
    }

    fn add_function(&mut self, type_index: u32) {
        self.functions.push(type_index);
    }

    fn add_export(&mut self, export: WasmExport) {
        self.exports.push(export);
    }

    fn add_code(&mut self, code: WasmFunctionCode) {
        self.code.push(code);
    }

    fn encode(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        // WebAssembly magic number and version
        bytes.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]); // "\0asm"
        bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // version 1

        // This is a simplified encoding - a full implementation would
        // properly encode all sections according to the WebAssembly spec
        
        Ok(bytes)
    }
}