# Synthesis - Universal Creative Programming Language

## Project Overview

Synthesis is a modern creative programming language designed for artists, musicians, and creative technologists. It seamlessly combines audio processing, visual graphics, interactive GUI development, and web deployment into a unified platform.

### Core Philosophy
- **Stream-based**: Everything flows as streams that can be connected and composed
- **Creative-friendly**: Intuitive syntax with percentage-based coordinates and auto-type conversion
- **Real-time focused**: Optimized for live performance and interactive installations
- **Universal platform**: From simple visualizers to professional DAWs to web applications

### Key Features
- **Audio**: Real-time processing, MIDI I/O, synthesis, effects, beat detection
- **Graphics**: Classic demo effects (plasma, starfield), particle systems, blend modes
- **GUI**: Immediate-mode native controls, professional layouts, low memory usage
- **Hardware**: Game controllers, webcam, Arduino, OSC protocols
- **Web Export**: Convert desktop creations to web apps and embeddable widgets
- **Timeline**: Sequencing, animation curves, synchronization tools

## Technology Stack

- **Language**: Rust (performance, safety, cross-platform)
- **Graphics**: wgpu (modern GPU API, WebGPU standard)
- **Audio**: cpal (cross-platform audio I/O)
- **GUI**: egui (immediate mode GUI)
- **Parser**: nom (parser combinators)

## Project Structure

```
synthesis-lang/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library interface
│   ├── parser/              # Language parsing (lexer, parser, AST)
│   │   ├── lexer.rs
│   │   ├── parser.rs
│   │   └── ast.rs
│   ├── runtime/             # Execution engine
│   │   ├── interpreter.rs
│   │   ├── streams.rs
│   │   └── types.rs
│   ├── graphics/            # Rendering engine
│   │   ├── renderer.rs
│   │   ├── effects.rs
│   │   └── primitives.rs
│   ├── audio/               # Audio processing
│   │   ├── input.rs
│   │   ├── analysis.rs
│   │   └── effects.rs
│   ├── modules/             # Built-in standard library
│   │   ├── graphics.rs      # Graphics module
│   │   ├── audio.rs         # Audio module
│   │   ├── gui.rs           # GUI module
│   │   ├── math.rs          # Math utilities
│   │   ├── time.rs          # Timeline/sequencing
│   │   └── web.rs           # Web export
│   └── gui/                 # Development environment GUI
├── examples/                # Example .syn programs
├── docs/                    # Language documentation
├── tests/                   # Integration tests
└── benchmarks/              # Performance tests
```

## Language Example

```synthesis
// Import required modules
import Audio.{mic_input, analyze_fft, beat_detect}
import Graphics.{clear, plasma, starfield, flash}
import GUI.{window, slider, button, control_group}
import Web.export_webapp

// Main program loop
loop {
    // Audio processing
    audio = Audio.mic_input()
    fft_data = Audio.analyze_fft(audio, 8)
    beat = Audio.beat_detect(audio)
    
    // GUI with controls
    GUI.window("Audio Visualizer", theme: "dark") {
        content: {
            controls = GUI.control_group("Settings") {
                sensitivity: GUI.slider("Sensitivity", 0.1, 5.0, 1.0)
                effect_type: GUI.dropdown("Effect", ["plasma", "starfield"], "plasma")
            }
            
            // Web export button
            if GUI.button("Export to Web", style: "primary") {
                Web.export_webapp("my_visualizer") {
                    controls: ["sensitivity", "effect_type"]
                    canvas: true
                    audio_input: true
                }
            }
            
            // Visual output
            Graphics.clear(Graphics.black)
            
            if controls.effect_type == "plasma" {
                Graphics.plasma(
                    speed: fft_data[0] * controls.sensitivity,
                    palette: Graphics.neon
                )
            } else {
                Graphics.starfield(
                    count: 200,
                    speed: fft_data[1] * controls.sensitivity
                )
            }
            
            // Beat-reactive flash
            if beat {
                Graphics.flash(Graphics.white, 0.1)
            }
        }
    }
}
```

## Development Phases

### Phase 1: Foundation (Months 1-3)
- Language core (lexer, parser, basic interpreter)
- Graphics MVP (OpenGL context, basic primitives)
- Module system (imports, namespacing)
- Stream system basics

### Phase 2: Creative Tools (Months 4-6)
- Audio foundation (input, analysis, real-time processing)
- GUI system (immediate mode controls)
- Advanced graphics (effects, blend modes)

### Phase 3: Professional Features (Months 7-9)
- MIDI I/O, audio effects
- Timeline and sequencing
- Hardware integration (controllers, webcam, sensors)

### Phase 4: Platform & Distribution (Months 10-12)
- Web export (WebAssembly compilation)
- Development tools (language server, debugger)
- Community platform (sharing, collaboration)

## Build Instructions

```bash
# Clone repository
git clone https://github.com/yourusername/synthesis-lang.git
cd synthesis-lang

# Build with all features
cargo build --release

# Run example
cargo run --bin synthesis examples/plasma.syn

# Run tests
cargo test

# Build for web (future)
cargo build --target wasm32-unknown-unknown --features web
```

## Contributing Guidelines

### Code Style
- Use `rustfmt` for formatting
- Follow Rust naming conventions
- Document public APIs with doc comments
- Write tests for new features

### Performance Requirements
- Audio code must be real-time safe (no allocations in audio thread)
- Graphics should maintain 60fps on reasonable hardware
- Memory usage should be predictable and bounded

### Module Development
New built-in modules should follow the established patterns:
- Stream-based APIs
- Percentage coordinates where applicable
- Comprehensive error handling
- Cross-platform compatibility

## Roadmap

- **Q1 2025**: Core language and basic graphics
- **Q2 2025**: Audio processing and GUI system
- **Q3 2025**: Professional features and hardware integration
- **Q4 2025**: Web export and community platform
- **2026**: Advanced features (ML, 3D, VR/AR)

## Community

- **Discord**: [TBD - Community server]
- **Forum**: [TBD - Discussion forum]
- **Gallery**: [TBD - User creations showcase]
- **Docs**: [TBD - Official documentation site]

---

Synthesis aims to be the bridge between creative vision and technical implementation, making professional-quality creative coding accessible to artists while providing the depth needed for complex projects.