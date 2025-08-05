# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Synthesis is a universal creative programming language designed for artists, musicians, and creative technologists. It combines audio processing, graphics, GUI development, and hardware integration into a unified platform optimized for real-time creative applications.

## Build System & Development Commands

### Build Commands
```bash
# Build the project (using custom build system)
./build.synt build                    # Release build (default)
./build.synt build --debug           # Debug build
./build.synt build --target wasm     # WebAssembly build
./build.synt build --verbose         # Verbose output

# Alternative: Direct Cargo (internal development)
cargo build --release                # Release build
cargo build                         # Debug build
cargo test                          # Run tests
```

### Testing
```bash
./build.synt test                    # Run test suite
./build.synt test --verbose         # Run tests with verbose output
```

### Installation & Distribution
```bash
./install.synt                      # System installation
./install.synt --dev               # Development installation
./release.synt                     # Create release packages
```

### Package Management
```bash
syn-pkg build                       # Build using package.syn
syn-pkg run examples/demo.syn       # Run example
syn-pkg test                        # Run tests defined in package.syn
```

## Architecture Overview

### Core Components
- **Parser/Lexer** (`src/parser/`): Language parsing with nom combinator library
- **Runtime** (`src/runtime/`): Stream-based interpreter with real-time optimizations
- **Graphics** (`src/graphics/`): wgpu-based rendering with creative effects
- **Audio** (`src/audio/`): Real-time audio processing with cpal
- **GUI** (`src/gui/`): Immediate-mode interface with egui
- **Hardware** (`src/hardware/`): Controller, sensor, and OSC integration
- **Modules** (`src/modules/`): Built-in standard library modules

### Language Features
- **Stream-based programming**: Everything flows as composable streams
- **Creative-friendly syntax**: Percentage coordinates, automatic type conversion
- **Real-time performance**: <1ms audio latency, 60fps graphics
- **Multi-target compilation**: Native binaries and WebAssembly

### Key Dependencies
- `wgpu` (0.19): Modern GPU API for graphics
- `cpal` (0.15): Cross-platform audio I/O
- `egui` (0.25): Immediate-mode GUI
- `nom` (7.1): Parser combinators
- `midir` (0.9): MIDI I/O
- `rosc` (0.10): OSC protocol support

## File Structure

```
synthesis-lang/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library interface  
│   ├── bin/                 # Additional binaries (synthc, syn-pkg)
│   ├── parser/              # Language parsing
│   ├── runtime/             # Execution engine with stream system
│   ├── compiler/            # Compilation backend and optimization
│   ├── graphics/            # Rendering and visual effects
│   ├── audio/               # Audio processing and analysis
│   ├── gui/                 # Development GUI components
│   ├── hardware/            # Hardware integration
│   └── modules/             # Built-in standard library
├── examples/                # .syn example programs
├── package.syn              # Project configuration
├── build.synt              # Custom build system
├── install.synt            # Installation script
└── _internal_dev/          # Internal development files
```

## Language Examples

### Basic Audio Visualizer
```synthesis
import Audio.{mic_input, analyze_fft}
import Graphics.{clear, plasma, flash}

loop {
    audio = Audio.mic_input()
    frequencies = Audio.analyze_fft(audio, 8)
    
    Graphics.clear(Graphics.black)
    Graphics.plasma(speed: frequencies[0] * 2.0, palette: Graphics.neon)
    
    if frequencies[0] > 0.7 {
        Graphics.flash(Graphics.white, 0.3)
    }
}
```

## Development Workflow

### Building and Testing
1. Use `./build.synt build` for main builds (hides Rust implementation)
2. Use `cargo build` for internal development
3. Always run tests before committing: `./build.synt test`
4. Test examples: `./target/release/synthesis examples/audio_visualizer.syn`

### Code Style
- Follow Rust naming conventions and formatting (rustfmt)
- Audio code must be real-time safe (no allocations in audio thread)
- Graphics should maintain 60fps on reasonable hardware
- Use stream-based APIs for new modules
- Maintain cross-platform compatibility

### Module Development
- New built-in modules go in `src/modules/`
- Follow established patterns: stream-based APIs, percentage coordinates
- Add comprehensive error handling
- Include examples in the module documentation

## Performance Requirements
- **Audio**: Real-time processing at 48kHz with <1ms latency
- **Graphics**: 60fps on reasonable hardware
- **Memory**: Predictable and bounded usage patterns
- **Real-time**: Garbage collection optimized for low latency

## Common Development Tasks

### Adding New Language Features
1. Update lexer in `src/parser/lexer.rs`
2. Extend AST in `src/parser/ast.rs` 
3. Update parser in `src/parser/parser.rs`
4. Implement in interpreter `src/runtime/interpreter.rs`
5. Add tests and examples

### Adding Built-in Modules
1. Create module file in `src/modules/`
2. Implement stream-based API
3. Register in `src/modules/mod.rs`
4. Add documentation and examples
5. Update language reference

### Compilation Targets
- `native`: Default native binary (fastest)
- `wasm32-unknown-unknown`: WebAssembly target
- Cross-compilation available for Linux, macOS, Windows

## Notes
- This is a creative programming language focused on real-time performance
- The build system intentionally hides Rust implementation details from users
- Stream-based programming model is central to the language design
- Graphics and audio systems are optimized for live performance scenarios