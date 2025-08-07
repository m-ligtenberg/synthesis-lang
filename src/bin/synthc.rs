use std::env;
use std::fs;
use std::path::Path;
use synthesis::parser::{lexer, Parser};
use synthesis::compiler::{Compiler, CompilationOptions, CompilationTarget, OptimizationLevel, NativeTarget};
use synthesis::errors::{SynthesisError, ErrorKind, Result};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help(&args[0]);
        return Ok(());
    }

    match args[1].as_str() {
        "--version" => {
            println!("Synthesis Language Compiler v0.1.0");
            println!("A universal creative programming language compiler");
            return Ok(());
        }
        "--help" => {
            print_help(&args[0]);
            return Ok(());
        }
        _ => {}
    }

    let mut options = CompilationOptions::default();
    let mut input_file = None;
    let mut output_file = None;
    
    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--target" => {
                if i + 1 >= args.len() {
                    return Err(SynthesisError::new(
                        ErrorKind::InvalidExpression,
                        "--target option needs a value"
                    ).with_suggestion("Try: --target wasm or --target native-linux"));
                }
                i += 1;
                options.target = parse_target(&args[i])?;
            }
            "--optimization" | "-O" => {
                if i + 1 >= args.len() {
                    return Err(SynthesisError::new(
                        ErrorKind::InvalidExpression,
                        "--optimization option needs a value"
                    ).with_suggestion("Try: -O basic, -O aggressive, or -O creative"));
                }
                i += 1;
                options.optimization_level = parse_optimization_level(&args[i])?;
            }
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --output requires an argument");
                    return Ok(());
                }
                i += 1;
                output_file = Some(args[i].clone());
            }
            "--debug" => {
                options.include_debug_info = true;
            }
            "--no-debug" => {
                options.include_debug_info = false;
            }
            "--buffer-size" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --buffer-size requires an argument");
                    return Ok(());
                }
                i += 1;
                options.stream_buffer_size = args[i].parse()
                    .map_err(|_| SynthesisError::new(
                        ErrorKind::InvalidExpression,
                        &format!("🎚️ Buffer size '{}' isn't a valid number", args[i])
                    )
                    .with_suggestion("Try a number like 512, 1024, or 2048")
                    .with_suggestion("Smaller buffers = lower latency, larger = more stable"))?;
            }
            "--no-realtime" => {
                options.real_time_priority = false;
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: Unknown option: {}", arg);
                return Ok(());
            }
            _ => {
                if input_file.is_none() {
                    input_file = Some(args[i].clone());
                } else {
                    eprintln!("Error: Multiple input files not supported");
                    return Ok(());
                }
            }
        }
        i += 1;
    }

    let input_path = match input_file {
        Some(path) => path,
        None => {
            eprintln!("Error: No input file specified");
            print_help(&args[0]);
            return Ok(());
        }
    };

    if !input_path.ends_with(".syn") {
        eprintln!("Error: Input file must have a .syn extension");
        return Ok(());
    }

    // Determine output file name if not specified
    let output_path = match output_file {
        Some(path) => path,
        None => {
            let input_stem = Path::new(&input_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow::anyhow!("Invalid input filename"))?;
            
            match options.target {
                CompilationTarget::WebAssembly => format!("{}.wasm", input_stem),
                CompilationTarget::Native(_) => {
                    if cfg!(windows) {
                        format!("{}.exe", input_stem)
                    } else {
                        input_stem.to_string()
                    }
                }
            }
        }
    };

    println!("Synthesis Language Compiler v0.1.0");
    println!("Compiling: {} -> {}", input_path, output_path);
    println!("Target: {:?}", options.target);
    println!("Optimization: {:?}", options.optimization_level);

    // Read source code
    let source_code = fs::read_to_string(&input_path)
        .map_err(|_| SynthesisError::file_not_found(&input_path))?
        .trim()
        .to_string();

    if source_code.is_empty() {
        return Err(SynthesisError::new(
            ErrorKind::SyntaxError,
            "Your Synthesis file is empty"
        ).with_suggestion("Add some creative code to get started!"));
    }

    // Parse the source code
    println!("Parsing...");
    let (_, tokens) = lexer::tokenize(&source_code)
        .map_err(|_| SynthesisError::new(
            ErrorKind::SyntaxError,
            "🎨 There's a syntax issue in your creative code"
        )
        .with_suggestion("Check for typos or missing punctuation")
        .with_suggestion("Make sure quotes and brackets are balanced"))?;

    let mut parser = Parser::new(&tokens);
    let program = parser.parse()?;

    // Compile the program
    println!("Compiling...");
    let mut compiler = Compiler::new();
    let artifact = compiler.compile(&program, options)?;

    // Write output
    println!("Writing output...");
    fs::write(&output_path, &artifact.bytecode)
        .map_err(|e| anyhow::anyhow!("Failed to write output file: {}", e))?;

    // Print compilation summary
    println!("\nCompilation successful!");
    println!("Output: {} ({} bytes)", output_path, artifact.bytecode.len());
    println!("Entry point: {}", artifact.metadata.entry_point);
    if !artifact.metadata.stream_interfaces.is_empty() {
        println!("Stream interfaces: {}", artifact.metadata.stream_interfaces.len());
        for stream in &artifact.metadata.stream_interfaces {
            println!("  - {} ({} -> {}, {}ms latency)", 
                stream.name, stream.input_type, stream.output_type, stream.latency_ms);
        }
    }
    if !artifact.metadata.dependencies.is_empty() {
        println!("Dependencies: {}", artifact.metadata.dependencies.join(", "));
    }

    match options.target {
        CompilationTarget::WebAssembly => {
            println!("\nTo run this WebAssembly module:");
            println!("1. Use a WebAssembly runtime like wasmtime or Node.js");
            println!("2. Provide the synthesis runtime environment");
            println!("3. Example: wasmtime --allow-unknown-exports {}", output_path);
        }
        CompilationTarget::Native(_) => {
            println!("\nTo run the compiled binary:");
            if cfg!(unix) {
                println!("  ./{}", output_path);
            } else {
                println!("  {}", output_path);
            }
        }
    }

    Ok(())
}

fn print_help(program_name: &str) {
    println!("Synthesis Language Compiler v0.1.0");
    println!("Usage: {} [OPTIONS] <input.syn>", program_name);
    println!();
    println!("OPTIONS:");
    println!("  --target <target>          Compilation target (wasm, native-linux, native-windows, native-macos)");
    println!("  -O, --optimization <level> Optimization level (none, basic, aggressive, creative)");
    println!("  -o, --output <file>        Output file path");
    println!("  --debug                    Include debug information (default)");
    println!("  --no-debug                 Exclude debug information");
    println!("  --buffer-size <size>       Default stream buffer size (default: 1024)");
    println!("  --no-realtime             Disable real-time optimization priority");
    println!("  --version                  Show version information");
    println!("  --help                     Show this help message");
    println!();
    println!("TARGETS:");
    println!("  wasm                       WebAssembly (default)");
    println!("  native-linux               Native Linux x86_64");
    println!("  native-windows             Native Windows x86_64");
    println!("  native-macos               Native macOS x86_64/ARM64");
    println!();
    println!("OPTIMIZATION LEVELS:");
    println!("  none                       No optimizations");
    println!("  basic                      Basic optimizations (default)");
    println!("  aggressive                 Aggressive optimizations");
    println!("  creative                   Creative coding specific optimizations");
    println!();
    println!("EXAMPLES:");
    println!("  {} audio_visualizer.syn", program_name);
    println!("  {} --target wasm -O creative visualizer.syn", program_name);
    println!("  {} --target native-linux -o myapp main.syn", program_name);
    println!("  {} --optimization aggressive --buffer-size 2048 performance_demo.syn", program_name);
}

fn parse_target(target_str: &str) -> synthesis::Result<CompilationTarget> {
    match target_str {
        "wasm" | "webassembly" => Ok(CompilationTarget::WebAssembly),
        "native-linux" => Ok(CompilationTarget::Native(NativeTarget::X86_64Linux)),
        "native-windows" => Ok(CompilationTarget::Native(NativeTarget::X86_64Windows)),
        "native-macos" => Ok(CompilationTarget::Native(NativeTarget::X86_64MacOS)),
        "native-macos-arm64" => Ok(CompilationTarget::Native(NativeTarget::AArch64MacOS)),
        "native-linux-arm64" => Ok(CompilationTarget::Native(NativeTarget::AArch64Linux)),
        _ => Err(anyhow::anyhow!("Unsupported target: {}", target_str)),
    }
}

fn parse_optimization_level(level_str: &str) -> synthesis::Result<OptimizationLevel> {
    match level_str {
        "none" | "0" => Ok(OptimizationLevel::None),
        "basic" | "1" => Ok(OptimizationLevel::Basic),
        "aggressive" | "2" => Ok(OptimizationLevel::Aggressive),
        "creative" | "3" => Ok(OptimizationLevel::Creative),
        _ => Err(anyhow::anyhow!("Invalid optimization level: {}", level_str)),
    }
}