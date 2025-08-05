use crate::parser::ast::*;
use crate::Result;
use std::collections::HashMap;

/// Intermediate Representation for Synthesis Language
/// Optimized for stream-based computation and creative coding
#[derive(Debug, Clone)]
pub struct IR {
    pub modules: Vec<IRModule>,
    pub entry_point: String,
    pub global_streams: Vec<StreamDefinition>,
}

#[derive(Debug, Clone)]
pub struct IRModule {
    pub name: String,
    pub functions: Vec<IRFunction>,
    pub streams: Vec<StreamDefinition>,
    pub constants: Vec<IRConstant>,
}

#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub parameters: Vec<IRParameter>,
    pub return_type: IRType,
    pub basic_blocks: Vec<BasicBlock>,
    pub is_stream_processor: bool,
    pub latency_constraint: Option<f32>, // Max latency in milliseconds
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub label: String,
    pub instructions: Vec<IRInstruction>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone)]
pub enum IRInstruction {
    // Arithmetic operations
    Add { dest: IRRegister, left: IRValue, right: IRValue },
    Sub { dest: IRRegister, left: IRValue, right: IRValue },
    Mul { dest: IRRegister, left: IRValue, right: IRValue },
    Div { dest: IRRegister, left: IRValue, right: IRValue },
    
    // Stream operations (unique to Synthesis)
    StreamCreate { dest: IRRegister, stream_type: IRType, buffer_size: usize },
    StreamRead { dest: IRRegister, stream: IRRegister },
    StreamWrite { stream: IRRegister, value: IRValue },
    StreamConnect { source: IRRegister, sink: IRRegister },
    StreamProcess { dest: IRRegister, processor: String, input: IRRegister },
    
    // Audio-specific operations
    AudioAnalyzeFFT { dest: IRRegister, audio: IRRegister, bands: usize },
    AudioApplyEffect { dest: IRRegister, audio: IRRegister, effect: String, params: Vec<IRValue> },
    
    // Graphics operations
    GraphicsDraw { primitive: String, params: Vec<IRValue> },
    GraphicsApplyShader { dest: IRRegister, input: IRRegister, shader: String },
    
    // Control flow
    Call { dest: Option<IRRegister>, function: String, args: Vec<IRValue> },
    Load { dest: IRRegister, address: IRValue },
    Store { address: IRValue, value: IRValue },
    
    // Type conversions (automatic in Synthesis)
    Convert { dest: IRRegister, value: IRValue, target_type: IRType },
    
    // Creative domain operations
    MapRange { dest: IRRegister, value: IRValue, from_min: IRValue, from_max: IRValue, to_min: IRValue, to_max: IRValue },
    Interpolate { dest: IRRegister, a: IRValue, b: IRValue, t: IRValue },
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Return(Option<IRValue>),
    Branch { condition: IRValue, true_block: String, false_block: String },
    Jump(String),
    StreamLoop, // Special terminator for stream processing loops
}

#[derive(Debug, Clone)]
pub struct IRRegister {
    pub id: usize,
    pub name: Option<String>,
    pub ir_type: IRType,
}

#[derive(Debug, Clone)]
pub enum IRValue {
    Register(IRRegister),
    Constant(IRConstant),
    Global(String),
}

#[derive(Debug, Clone)]
pub enum IRConstant {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    AudioFrequency(f64), // Special type for audio frequencies
    ColorRGB(u8, u8, u8),
    Percentage(f64), // 0.0 to 1.0
}

#[derive(Debug, Clone)]
pub enum IRType {
    // Basic types
    Integer,
    Float,
    Boolean,
    String,
    
    // Creative domain types
    AudioSample,
    AudioBuffer,
    AudioFrequency,
    Color,
    Coordinate,
    Percentage,
    
    // Stream types
    Stream(Box<IRType>),
    
    // Composite types
    Array(Box<IRType>, Option<usize>),
    Struct(Vec<(String, IRType)>),
    
    // Function type
    Function(Vec<IRType>, Box<IRType>),
    
    // Special types
    Any, // For dynamic typing
    Void,
}

#[derive(Debug, Clone)]
pub struct StreamDefinition {
    pub name: String,
    pub input_type: IRType,
    pub output_type: IRType,
    pub buffer_size: usize,
    pub processing_function: Option<String>,
    pub real_time: bool,
}

#[derive(Debug, Clone)]
pub struct IRParameter {
    pub name: String,
    pub ir_type: IRType,
    pub default_value: Option<IRConstant>,
}

pub struct IRGenerator {
    next_register_id: usize,
    next_block_id: usize,
    current_function: Option<String>,
    symbol_table: HashMap<String, IRRegister>,
}

impl IRGenerator {
    pub fn new() -> Self {
        Self {
            next_register_id: 0,
            next_block_id: 0,
            current_function: None,
            symbol_table: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<IR> {
        let mut ir = IR {
            modules: Vec::new(),
            entry_point: "main".to_string(),
            global_streams: Vec::new(),
        };

        // Generate IR for main program
        let mut main_module = IRModule {
            name: "main".to_string(),
            functions: Vec::new(),
            streams: Vec::new(),
            constants: Vec::new(),
        };

        // Create main function
        let mut main_function = IRFunction {
            name: "main".to_string(),
            parameters: Vec::new(),
            return_type: IRType::Void,
            basic_blocks: Vec::new(),
            is_stream_processor: false,
            latency_constraint: None,
        };

        let mut entry_block = BasicBlock {
            label: "entry".to_string(),
            instructions: Vec::new(),
            terminator: Terminator::Return(None),
        };

        // Process program items
        for item in &program.items {
            match item {
                Item::Import(import) => {
                    self.generate_import(&mut entry_block, import)?;
                }
                Item::Statement(stmt) => {
                    self.generate_statement(&mut entry_block, stmt)?;
                }
                Item::Loop(loop_block) => {
                    self.generate_loop(&mut main_function, loop_block)?;
                }
                Item::Function(_func_def) => {
                    // TODO: Implement function definition handling
                }
                Item::Class(_class_def) => {
                    // TODO: Implement class definition handling
                }
                Item::Struct(_struct_def) => {
                    // TODO: Implement struct definition handling
                }
            }
        }

        main_function.basic_blocks.push(entry_block);
        main_module.functions.push(main_function);
        ir.modules.push(main_module);

        Ok(ir)
    }

    fn generate_import(&mut self, block: &mut BasicBlock, _import: &crate::parser::ast::ImportItem) -> Result<()> {
        // For now, imports are handled at compile time
        // In the future, this could generate dynamic loading instructions
        Ok(())
    }

    fn generate_statement(&mut self, block: &mut BasicBlock, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Assignment { name, value } => {
                let dest_reg = self.allocate_register(Some(name.clone()), IRType::Any);
                let value_ir = self.generate_expression(block, value)?;
                
                block.instructions.push(IRInstruction::Load {
                    dest: dest_reg.clone(),
                    address: value_ir,
                });
                
                self.symbol_table.insert(name.clone(), dest_reg);
            }
            Statement::Expression(expr) => {
                self.generate_expression(block, expr)?;
            }
            Statement::If { .. } => {
                // TODO: Implement if statement IR generation
            }
            Statement::Match { .. } => {
                // TODO: Implement match statement IR generation
            }
            Statement::Every { .. } => {
                // TODO: Implement every statement IR generation
            }
            Statement::After { .. } => {
                // TODO: Implement after statement IR generation
            }
            Statement::While { .. } => {
                // TODO: Implement while statement IR generation
            }
            Statement::For { .. } => {
                // TODO: Implement for statement IR generation
            }
            Statement::Let { .. } => {
                // TODO: Implement let statement IR generation
            }
            Statement::Return(_) => {
                // TODO: Implement return statement IR generation
            }
            Statement::Break => {
                // TODO: Implement break statement IR generation
            }
            Statement::Continue => {
                // TODO: Implement continue statement IR generation
            }
        }
        Ok(())
    }

    fn generate_expression(&mut self, block: &mut BasicBlock, expr: &Expression) -> Result<IRValue> {
        match expr {
            Expression::Literal(literal) => {
                match literal {
                    Literal::Integer(n) => Ok(IRValue::Constant(IRConstant::Integer(*n))),
                    Literal::Float(f) => Ok(IRValue::Constant(IRConstant::Float(*f))),
                    Literal::String(s) => Ok(IRValue::Constant(IRConstant::String(s.clone()))),
                    Literal::Boolean(b) => Ok(IRValue::Constant(IRConstant::Boolean(*b))),
                }
            }
            Expression::Identifier(name) => {
                if let Some(reg) = self.symbol_table.get(name) {
                    Ok(IRValue::Register(reg.clone()))
                } else {
                    Ok(IRValue::Global(name.clone()))
                }
            }
            Expression::FunctionCall { module: _, name, args, named_args: _ } => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.generate_expression(block, arg)?);
                }

                // Special handling for stream operations
                if name.contains('.') {
                    let parts: Vec<&str> = name.split('.').collect();
                    if parts.len() == 2 {
                        let module = parts[0];
                        let function = parts[1];
                        
                        return self.generate_module_call(block, module, function, arg_values);
                    }
                }

                let dest_reg = self.allocate_register(None, IRType::Any);
                block.instructions.push(IRInstruction::Call {
                    dest: Some(dest_reg.clone()),
                    function: name.clone(),
                    args: arg_values,
                });

                Ok(IRValue::Register(dest_reg))
            }
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.generate_expression(block, left)?;
                let right_val = self.generate_expression(block, right)?;
                let dest_reg = self.allocate_register(None, IRType::Any);

                let instruction = match op {
                    BinaryOperator::Add => IRInstruction::Add { dest: dest_reg.clone(), left: left_val, right: right_val },
                    BinaryOperator::Subtract => IRInstruction::Sub { dest: dest_reg.clone(), left: left_val, right: right_val },
                    BinaryOperator::Multiply => IRInstruction::Mul { dest: dest_reg.clone(), left: left_val, right: right_val },
                    BinaryOperator::Divide => IRInstruction::Div { dest: dest_reg.clone(), left: left_val, right: right_val },
                    _ => return Err(anyhow::anyhow!("Unsupported binary operator: {:?}", op).into()),
                };

                block.instructions.push(instruction);
                Ok(IRValue::Register(dest_reg))
            }
            Expression::Block { .. } => {
                // TODO: Implement block expression
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::ArrayAccess { .. } => {
                // TODO: Implement array access
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::Pipe { .. } => {
                // TODO: Implement pipe expression
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::BiDirectionalPipe { .. } => {
                // TODO: Implement bidirectional pipe
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::StreamBranch { .. } => {
                // TODO: Implement stream branch
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::StreamMerge { .. } => {
                // TODO: Implement stream merge
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::UnitValue { .. } => {
                // TODO: Implement unit value
                Ok(IRValue::Constant(IRConstant::Float(0.0)))
            }
            Expression::ArrayLiteral(_elements) => {
                // TODO: Implement array literal generation
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::Range { start: _start, end: _end, inclusive: _inclusive } => {
                // TODO: Implement range generation
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::Lambda { parameters: _parameters, body: _body } => {
                // TODO: Implement lambda generation
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::MethodCall { object: _object, method: _method, args: _args, named_args: _named_args } => {
                // TODO: Implement method call generation
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::InterpolatedString(_parts) => {
                // TODO: Implement string interpolation generation
                Ok(IRValue::Constant(IRConstant::String("".to_string())))
            }
            Expression::ConditionalExpression { condition: _condition, true_expr: _true_expr, false_expr: _false_expr } => {
                // TODO: Implement conditional expression generation
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::MatchExpression { expr: _expr, arms: _arms } => {
                // TODO: Implement match expression generation
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
            Expression::TypeCast { expr: _expr, target_type: _target_type } => {
                // TODO: Implement type cast generation
                Ok(IRValue::Constant(IRConstant::Integer(0)))
            }
        }
    }

    fn generate_module_call(
        &mut self,
        block: &mut BasicBlock,
        module: &str,
        function: &str,
        args: Vec<IRValue>,
    ) -> Result<IRValue> {
        let dest_reg = self.allocate_register(None, IRType::Any);

        match module {
            "Audio" => {
                match function {
                    "analyze_fft" => {
                        if args.is_empty() {
                            return Err(anyhow::anyhow!("analyze_fft requires audio input").into());
                        }
                        let bands = if args.len() > 1 {
                            8 // Default or extract from args
                        } else {
                            8
                        };
                        
                        if let IRValue::Register(audio_reg) = &args[0] {
                            block.instructions.push(IRInstruction::AudioAnalyzeFFT {
                                dest: dest_reg.clone(),
                                audio: audio_reg.clone(),
                                bands,
                            });
                        }
                    }
                    _ => {
                        block.instructions.push(IRInstruction::Call {
                            dest: Some(dest_reg.clone()),
                            function: format!("{}_{}", module, function),
                            args,
                        });
                    }
                }
            }
            "Graphics" => {
                match function {
                    "clear" | "plasma" | "starfield" => {
                        block.instructions.push(IRInstruction::GraphicsDraw {
                            primitive: function.to_string(),
                            params: args,
                        });
                    }
                    _ => {
                        block.instructions.push(IRInstruction::Call {
                            dest: Some(dest_reg.clone()),
                            function: format!("{}_{}", module, function),
                            args,
                        });
                    }
                }
            }
            _ => {
                block.instructions.push(IRInstruction::Call {
                    dest: Some(dest_reg.clone()),
                    function: format!("{}_{}", module, function),
                    args,
                });
            }
        }

        Ok(IRValue::Register(dest_reg))
    }

    fn generate_loop(&mut self, function: &mut IRFunction, loop_block: &LoopBlock) -> Result<()> {
        let loop_label = format!("loop_{}", self.next_block_id);
        self.next_block_id += 1;
        let exit_label = format!("loop_exit_{}", self.next_block_id);
        self.next_block_id += 1;

        // Create loop header block
        let mut loop_header = BasicBlock {
            label: loop_label.clone(),
            instructions: Vec::new(),
            terminator: Terminator::Jump(loop_label.clone()),
        };

        // Generate loop body
        for stmt in &loop_block.body {
            self.generate_statement(&mut loop_header, stmt)?;
        }

        // For stream loops, use special terminator
        loop_header.terminator = Terminator::StreamLoop;

        function.basic_blocks.push(loop_header);
        Ok(())
    }

    fn allocate_register(&mut self, name: Option<String>, ir_type: IRType) -> IRRegister {
        let id = self.next_register_id;
        self.next_register_id += 1;
        
        IRRegister {
            id,
            name,
            ir_type,
        }
    }
}