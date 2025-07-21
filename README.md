# Synthesis

**Universal creative programming language for artists, musicians, and creative technologists.**

Stream-based | Real-time | Cross-platform

## Quick Start

```bash
git clone https://github.com/m-ligtenberg/synthesis-lang.git
cd synthesis-lang
cargo build --release
cargo run examples/audio_visualizer.syn
```

## Features

- **Audio**: Real-time processing, MIDI I/O, synthesis, professional effects
- **Graphics**: GPU-accelerated rendering, demo effects, particle systems  
- **GUI**: Immediate-mode controls, professional layouts
- **Hardware**: Controllers, webcam, Arduino, OSC protocols
- **Timeline**: Sequencing, automation, synchronization

## Language

```synthesis
import Audio.{mic_input, analyze_fft}
import Graphics.{clear, plasma}

loop {
    audio = Audio.mic_input()
    fft = Audio.analyze_fft(audio, 8)
    
    Graphics.clear()
    Graphics.plasma(speed: fft[0] * 2.0)
}
```

## Architecture

```
synthesis-lang/
├── src/
│   ├── parser/          # Language parsing
│   ├── runtime/         # Execution engine  
│   ├── graphics/        # GPU rendering
│   ├── audio/           # Real-time audio
│   ├── gui/             # Interface controls
│   ├── hardware/        # Device integration
│   └── modules/         # Built-in library
├── examples/            # Demo programs
└── docs/               # Documentation
```

## Requirements

- **Rust** 1.70+
- **Linux** (primary), macOS, Windows
- **GPU** with Vulkan/Metal/DX12
- **Audio** ALSA/PulseAudio

## Examples

| File | Description |
|------|-------------|
| `hello.syn` | Basic audio-visual loop |
| `audio_visualizer.syn` | Real-time audio analysis |
| `professional_daw.syn` | Multi-track DAW interface |

## Build

```bash
# Development
cargo build

# Release
cargo build --release

# With all features
cargo build --release --all-features

# Web target (future)
cargo build --target wasm32-unknown-unknown
```

## Usage

```bash
# Run program
synthesis examples/demo.syn

# Or via cargo
cargo run examples/demo.syn

# Debug mode
RUST_LOG=debug synthesis examples/demo.syn
```

## Hardware Support

- **MIDI**: Input/output with message parsing
- **Controllers**: Xbox, PlayStation, generic HID
- **Webcam**: Motion detection, color analysis
- **Arduino**: Serial communication, sensor data
- **OSC**: Network protocol for external control

## Development

```bash
# Tests
cargo test

# Format
cargo fmt

# Lint
cargo clippy

# Documentation
cargo doc --open
```

## License

MIT
