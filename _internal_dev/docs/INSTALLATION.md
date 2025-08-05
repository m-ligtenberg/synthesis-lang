# Installation Guide

## Requirements

### System
- **Linux** (Ubuntu 20.04+, Arch, Fedora)  
- **Rust** 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Git** (`apt install git` / `pacman -S git` / `dnf install git`)

### Audio
- **ALSA** (`apt install libasound2-dev`)
- **PulseAudio** (usually pre-installed)
- **JACK** (optional, `apt install libjack-jackd2-dev`)

### Graphics  
- **Vulkan** drivers for your GPU
- **OpenGL** 4.1+ fallback support

### Hardware (optional)
- **USB MIDI** device permissions (`usermod -a -G audio $USER`)
- **Serial** ports for Arduino (`usermod -a -G dialout $USER`)
- **Webcam** for computer vision (`apt install libopencv-dev`)

## Quick Install

```bash
# Clone
git clone https://github.com/m-ligtenberg/synthesis-lang.git
cd synthesis-lang

# Build
cargo build --release

# Test
cargo run examples/hello.syn
```

## Distribution Packages

### Ubuntu/Debian
```bash
sudo apt update
sudo apt install build-essential pkg-config libasound2-dev libudev-dev
```

### Arch Linux  
```bash
sudo pacman -S base-devel alsa-lib systemd
```

### Fedora
```bash
sudo dnf install gcc pkg-config alsa-lib-devel systemd-devel
```

## Hardware Setup

### MIDI Devices
```bash
# Check available devices
aconnect -l

# Test MIDI input
amidi -p hw:1,0 -d
```

### Audio Latency
```bash
# Low latency audio (optional)
sudo usermod -a -G audio $USER
echo "@audio - rtprio 95" | sudo tee -a /etc/security/limits.conf
echo "@audio - memlock unlimited" | sudo tee -a /etc/security/limits.conf
```

### Arduino/Serial
```bash
# Check serial ports
ls /dev/ttyUSB* /dev/ttyACM*

# Permissions
sudo usermod -a -G dialout $USER
```

## Troubleshooting

### Audio Issues
```bash
# Check ALSA devices
aplay -l

# Test audio output
speaker-test -t wav -c 2

# PulseAudio restart
pulseaudio -k && pulseaudio --start
```

### MIDI Issues
```bash
# List MIDI devices
amidi -l

# Check ALSA MIDI
cat /proc/asound/seq/clients
```

### Build Issues
```bash
# Update Rust
rustup update

# Clear cargo cache
cargo clean
```

### Permission Issues
```bash
# Audio group
groups $USER | grep audio

# Dialout group  
groups $USER | grep dialout

# Relogin required after group changes
```

---

# Installatiegids (Nederlands)

## Vereisten

### Systeem
- **Linux** (Ubuntu 20.04+, Arch, Fedora)
- **Rust** 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)  
- **Git** (`apt install git` / `pacman -S git` / `dnf install git`)

### Audio
- **ALSA** (`apt install libasound2-dev`)
- **PulseAudio** (meestal voorgeïnstalleerd)
- **JACK** (optioneel, `apt install libjack-jackd2-dev`)

### Graphics
- **Vulkan** drivers voor je GPU
- **OpenGL** 4.1+ fallback ondersteuning

## Snelle Installatie

```bash
# Klonen
git clone https://github.com/m-ligtenberg/synthesis-lang.git
cd synthesis-lang

# Bouwen
cargo build --release

# Testen
cargo run examples/hello.syn
```

## Hardware Setup

### MIDI Apparaten
```bash
# Beschikbare apparaten controleren
aconnect -l

# MIDI input testen
amidi -p hw:1,0 -d
```

### Audio Latentie
```bash
# Lage latentie audio (optioneel)
sudo usermod -a -G audio $USER
echo "@audio - rtprio 95" | sudo tee -a /etc/security/limits.conf
echo "@audio - memlock unlimited" | sudo tee -a /etc/security/limits.conf
```

## Probleemoplossing

### Audio Problemen
```bash
# ALSA apparaten controleren  
aplay -l

# Audio output testen
speaker-test -t wav -c 2

# PulseAudio herstart
pulseaudio -k && pulseaudio --start
```

### Build Problemen
```bash
# Rust updaten
rustup update

# Cargo cache legen
cargo clean
```