# Synthesis (Nederlands)

**Universele creatieve programmeertaal voor kunstenaars, muzikanten en creatieve nerds.**

Stream-gebaseerd | Real-time | Cross-platform

## Snel Starten

```bash
git clone https://github.com/m-ligtenberg/synthesis-lang.git
cd synthesis-lang
cargo build --release  
cargo run examples/audio_visualizer.syn
```

## Functies

- **Audio**: Real-time verwerking, MIDI I/O, synthese, professionele effecten
- **Graphics**: GPU-versnelde rendering, demo-effecten, particle systemen
- **GUI**: Immediate-mode controls, professionele layouts
- **Hardware**: Controllers, webcam, Arduino, OSC protocollen
- **Timeline**: Sequencing, automatisering, synchronisatie

## Taal

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

## Architectuur

```
synthesis-lang/
├── src/
│   ├── parser/          # Taal parsing
│   ├── runtime/         # Uitvoering engine
│   ├── graphics/        # GPU rendering  
│   ├── audio/           # Real-time audio
│   ├── gui/             # Interface controls
│   ├── hardware/        # Apparaat integratie
│   └── modules/         # Ingebouwde bibliotheek
├── examples/            # Demo programma's
└── docs/               # Documentatie
```

## Vereisten

- **Rust** 1.70+
- **Linux** (primair), macOS, Windows
- **GPU** met Vulkan/Metal/DX12  
- **Audio** ALSA/PulseAudio

## Voorbeelden

| Bestand | Beschrijving |
|---------|-------------|
| `hello.syn` | Basis audio-visuele loop |
| `audio_visualizer.syn` | Real-time audio analyse |
| `professional_daw.syn` | Multi-track DAW interface |

## Bouwen

```bash
# Ontwikkeling
cargo build

# Release
cargo build --release

# Met alle functies  
cargo build --release --all-features

# Web target (toekomst)
cargo build --target wasm32-unknown-unknown
```

## Gebruik

```bash
# Programma uitvoeren
synthesis examples/demo.syn

# Of via cargo
cargo run examples/demo.syn

# Debug modus
RUST_LOG=debug synthesis examples/demo.syn
```

## Hardware Ondersteuning

- **MIDI**: Input/output met message parsing
- **Controllers**: Xbox, PlayStation, generieke HID
- **Webcam**: Bewegingsdetectie, kleuranalyse
- **Arduino**: Seriële communicatie, sensor data
- **OSC**: Netwerkprotocol voor externe besturing

## Ontwikkeling

```bash
# Tests
cargo test

# Formattering
cargo fmt

# Lint
cargo clippy

# Documentatie
cargo doc --open
```

## Licentie

MIT
