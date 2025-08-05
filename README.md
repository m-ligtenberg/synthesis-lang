# 🎨 Synthesis Language

**The Universal Creative Programming Language**

Synthesis bridges the gap between creative vision and technical implementation, making professional-quality creative coding accessible to artists while providing the depth needed for complex projects.

```synthesis
// Real-time audio visualizer in just a few lines
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

## ✨ Why Synthesis?

- **🎵 Audio-First**: Real-time audio processing, MIDI I/O, synthesis, effects
- **🎨 Visual**: GPU-accelerated graphics, shaders, particles, classic demo effects  
- **🖱️ Interactive**: Immediate-mode GUI, hardware controllers, sensors
- **🌐 Universal**: Compile to WebAssembly, native binaries, or web apps
- **🚀 Performance**: Real-time optimized with creative domain-specific optimizations

## 🚀 Quick Start

### Install Synthesis
```bash
curl -fsSL https://synthesis-lang.org/install | bash
```

### Your First Program
```bash
# Create a simple visualizer
echo 'import Graphics.plasma; loop { Graphics.plasma() }' > hello.syn

# Compile to WebAssembly
synthc hello.syn

# Run it
synthesis hello.wasm
```

### Try the Examples
```bash
# Audio visualizer
synthc examples/audio_visualizer.syn && synthesis audio_visualizer.wasm

# Interactive graphics
synthc examples/math_demo.syn && synthesis math_demo.wasm
```

## 🎯 Language Features

### Stream-Based Programming
Everything flows as streams that can be connected and composed:
```synthesis
audio = Audio.mic_input()
  |> Audio.apply_reverb(room_size: 0.8)
  |> Audio.analyze_fft(bands: 16)
  |> Graphics.spectrum_visualizer()
```

### Creative-friendly Syntax
- Percentage-based coordinates: `Graphics.circle(50%, 50%, 25%)`
- Automatic type conversions: `0.5` becomes `50%` when used as coordinate
- Intuitive color handling: `Graphics.red`, `#FF0000`, `rgb(255, 0, 0)`
- Time-based animations: `sin(time() * 2.0)`

### Real-time Performance
- Audio processing at 48kHz with <1ms latency
- 60fps graphics on reasonable hardware
- Predictable memory usage
- Real-time garbage collection

### Professional Workflows
- MIDI controller integration
- Hardware sensor support (Arduino, OSC)
- Timeline and sequencing tools
- Web export for sharing creations

## 📁 Project Structure

```
my-synthesis-project/
├── main.syn              # Your main program
├── package.syn           # Dependencies and metadata
├── assets/              # Audio samples, images, shaders
└── build/               # Compiled outputs
```

## 🛠️ Development

### Build from Source
```bash
git clone https://github.com/synthesis-lang/synthesis.git
cd synthesis
./build.synt build
./install.synt --dev
```

### Compilation Targets
- `--target wasm` - WebAssembly (default)
- `--target native-linux` - Linux x86_64
- `--target native-windows` - Windows x86_64  
- `--target native-macos` - macOS Universal

### Optimization Levels
- `-O none` - No optimizations (fastest compile)
- `-O basic` - Standard optimizations (default)
- `-O aggressive` - Maximum performance
- `-O creative` - Creative coding specific optimizations

## 🌟 Examples

### Audio Processing
```synthesis
import Audio.{mic_input, apply_reverb, synthesize_sine}

audio = Audio.mic_input()
reverb = Audio.apply_reverb(audio, room_size: 0.8, decay: 0.6)
sine = Audio.synthesize_sine(440.0, amplitude: 0.3)
Audio.output(reverb + sine)
```

### Interactive Graphics
```synthesis
import Graphics.{clear, circle, mouse_position}
import Math.{sin, time}

loop {
    Graphics.clear(Graphics.black)
    
    mouse = Graphics.mouse_position()
    radius = sin(time() * 3.0) * 50.0 + 100.0
    
    Graphics.circle(
        x: mouse.x,
        y: mouse.y, 
        radius: radius,
        color: Graphics.hsv(time() * 60.0, 100%, 80%)
    )
}
```

### MIDI Control
```synthesis
import Hardware.{midi_input}
import Audio.{synthesize_sine}

loop {
    midi = Hardware.midi_input()
    
    if midi.note_on {
        frequency = midi.note_to_frequency(midi.note)
        Audio.synthesize_sine(frequency, amplitude: midi.velocity / 127.0)
    }
}
```

## 📚 Learn More

- **📖 Documentation**: [synthesis-lang.org/docs](https://synthesis-lang.org/docs)
- **🎵 Tutorials**: [synthesis-lang.org/tutorials](https://synthesis-lang.org/tutorials)  
- **🎨 Gallery**: [synthesis-lang.org/gallery](https://synthesis-lang.org/gallery)
- **💬 Community**: [synthesis-lang.org/community](https://synthesis-lang.org/community)

## 🤝 Contributing

We welcome contributions! See our [Contributing Guide](_internal_dev/docs/CONTRIBUTING.md) for details.

### Areas We Need Help
- 🎵 Audio processing algorithms
- 🎨 Graphics effects and shaders
- 🌐 Web platform integration
- 📱 Mobile/embedded targets
- 📚 Documentation and tutorials
- 🧪 Testing and benchmarks

## 📄 License

Synthesis is dual-licensed under MIT and Apache 2.0. Use whichever works best for your project.

---

**Built with ❤️ for artists, musicians, and creative technologists.**

*Synthesis aims to be the bridge between creative vision and technical implementation, making professional-quality creative coding accessible to artists while providing the depth needed for complex projects.*