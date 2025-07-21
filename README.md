# Synthesis

**Universele creatieve programmeertaal voor kunstenaars, muzikanten en creatieve nerds.**  
*Stream-gebaseerd – Realtime – Cross-platform*

---

## Overzicht

Synthesis is een moderne creatieve programmeertaal ontworpen voor makers die technologie als expressief medium gebruiken. Van audiovisuele installaties tot realtime optredens en interactieve tools — deze taal is ontwikkeld om het creatieve proces intuïtiever, performanter en toegankelijker te maken.
Ik miste een taal die voelt alsof hij gemaakt is door een kunstenaar, niet door een ingenieur. De focus ligt op realtime creatie, directe visuele en auditieve feedback, en een modulaire benadering die aansluit bij hoe kunstenaars denken: in lagen, flows en gevoel — niet in low-level systemen.

---

## Filosofie

- **Stream-based**: Alles stroomt als verbonden datastromen
- **Creatief-vriendelijk**: Intuïtieve syntaxis, percentage-coördinaten, auto-type conversie
- **Live performance**: Realtime optimalisatie voor installaties en optredens
- **Universeel platform**: Van eenvoudige visualisaties tot DAWs en webapps

---

## Quick Start

```bash
git clone https://github.com/m-ligtenberg/synthesis-lang.git
cd synthesis-lang
cargo build --release
cargo run examples/audio_visualizer.syn
```

---

## Kernfuncties

- **Audio**: Realtime verwerking, MIDI I/O, synthese, beat detectie, effecten
- **Graphics**: GPU-acceleratie, plasma, starfield, particle systems, blend modes
- **GUI**: Immediate-mode controls, professionele layouts, minimale memory-voetafdruk
- **Hardware**: Ondersteuning voor controllers, webcam, Arduino, OSC
- **Web Export**: Desktop creaties converteren naar webapps
- **Timeline**: Animatiecurves, sequencing en synchronisatie

---

## Voorbeeldcode

```synthesis
import Audio.{mic_input, analyze_fft, beat_detect}
import Graphics.{clear, plasma, starfield, flash}
import GUI.{window, slider, button, control_group}
import Web.export_webapp

loop {
    audio = Audio.mic_input()
    fft_data = Audio.analyze_fft(audio, 8)
    beat = Audio.beat_detect(audio)

    GUI.window("Audio Visualizer", theme: "dark") {
        content: {
            controls = GUI.control_group("Settings") {
                sensitivity: GUI.slider("Sensitivity", 0.1, 5.0, 1.0)
                effect_type: GUI.dropdown("Effect", ["plasma", "starfield"], "plasma")
            }

            if GUI.button("Export to Web", style: "primary") {
                Web.export_webapp("my_visualizer") {
                    controls: ["sensitivity", "effect_type"]
                    canvas: true
                    audio_input: true
                }
            }

            Graphics.clear(Graphics.black)
            if controls.effect_type == "plasma" {
                Graphics.plasma(speed: fft_data[0] * controls.sensitivity, palette: Graphics.neon)
            } else {
                Graphics.starfield(count: 200, speed: fft_data[1] * controls.sensitivity)
            }

            if beat {
                Graphics.flash(Graphics.white, 0.1)
            }
        }
    }
}
```

---

## Projectstructuur

```
synthesis-lang/
├── src/
│   ├── parser/          # Lexer, parser, AST
│   ├── runtime/         # Interpreter & stream engine
│   ├── graphics/        # Renderer en visuele effecten
│   ├── audio/           # Realtime audioverwerking
│   ├── gui/             # GUI systeem (immediate-mode)
│   ├── modules/         # Standaardbibliotheek (audio, gfx, time, math, web)
│   └── hardware/        # Gamepad, webcam, Arduino, OSC
├── examples/            # Demo .syn programma's
├── docs/                # Documentatie en tutorials
├── tests/               # Integratietests
└── benchmarks/          # Performance benchmarks
```

---

## Systeemvereisten

- **Rust** 1.70+
- Linux (primair), macOS, Windows
- GPU met ondersteuning voor Vulkan, Metal of DirectX12
- Audio: ALSA of PulseAudio

---

## Build en Gebruik

```bash
# Ontwikkelbuild
cargo build

# Releasebuild
cargo build --release

# Webtarget (in ontwikkeling)
cargo build --target wasm32-unknown-unknown --features web

# Programma uitvoeren
synthesis examples/demo.syn

# Debug modus
RUST_LOG=debug synthesis examples/demo.syn
```

---

## Voorbeelden

| Bestand | Beschrijving |
|---------|--------------|
| `hello.syn` | Basis audio-visuele loop |
| `audio_visualizer.syn` | Realtime audio-analyse |
| `professional_daw.syn` | Multi-track DAW interface |

---

## Ontwikkeling

```bash
cargo test         # Testen
cargo fmt          # Formatteren
cargo clippy       # Linter
cargo doc --open   # Documentatie genereren
```

---

## Roadmap

| Periode | Doelen |
|--------|--------|
| fase 1 | Taalkernel, basisgrafiek |
| fase 2 | Audio, GUI systeem |
| fase 3 | Timeline, hardware support |
| fase 4 | Web export, community platform |
| toekomst | ML integratie, 3D, VR/AR ondersteuning |

---

## Community (in aanbouw)

- Discord-server
- Gebruikersgalerij
- Forum en documentatieplatform

---

## Licentie

MIT
