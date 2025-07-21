# Examples & Tutorials

## Basic Examples

### Hello World
```synthesis
# examples/hello.syn
import Graphics.{clear, text, rgb}

loop {
    Graphics.clear(Graphics.rgb(0.1, 0.1, 0.2))
    Graphics.text("Hello, Synthesis!", x: 0.5, y: 0.5, color: Graphics.rgb(1.0, 1.0, 1.0))
}
```

### Audio Input Visualization
```synthesis
# examples/audio_viz.syn
import Audio.{mic_input, analyze_fft}
import Graphics.{clear, bars, spectrum_analyzer}

loop {
    audio = Audio.mic_input()
    fft_data = Audio.analyze_fft(audio, 64)
    
    Graphics.clear()
    Graphics.spectrum_analyzer(fft_data, style: "bars", color: "rainbow")
}
```

### MIDI Piano
```synthesis
# examples/midi_piano.syn
import Audio.{MIDIInput, Oscillator}
import Graphics.{clear, piano_keys, highlight_key}

midi = Audio.MIDIInput.new()
midi.open_all_devices()

oscillators = {}

loop {
    messages = midi.get_messages()
    
    for msg in messages {
        match msg.type {
            "note_on" => {
                osc = Audio.Oscillator.new(msg.note_frequency(), 0.3)
                oscillators[msg.note] = osc
                Graphics.highlight_key(msg.note, "on")
            }
            "note_off" => {
                oscillators.remove(msg.note)
                Graphics.highlight_key(msg.note, "off")
            }
        }
    }
    
    Graphics.clear()
    Graphics.piano_keys()
}
```

## Creative Applications

### Interactive Particle System
```synthesis
# examples/particles.syn
import Hardware.{controller_manager}
import Graphics.{clear, particles, set_gravity}

controllers = Hardware.controller_manager()
particle_system = Graphics.ParticleSystem.new(1000)

loop {
    controllers.update()
    
    # Left stick controls gravity
    grav_x = controllers.get_axis_value(0, 0) * 2.0
    grav_y = controllers.get_axis_value(0, 1) * 2.0
    Graphics.set_gravity(grav_x, grav_y)
    
    # Right trigger spawns particles  
    spawn_rate = controllers.get_axis_value(0, 5) * 50.0
    particle_system.spawn(spawn_rate)
    
    # A button resets
    if controllers.is_button_pressed(0, "A") {
        particle_system.reset()
    }
    
    Graphics.clear()
    particle_system.update()
    particle_system.render()
}
```

### Beat-Reactive Visuals
```synthesis
# examples/beat_visuals.syn
import Audio.{mic_input, beat_detect, analyze_fft}
import Graphics.{clear, flash, circle, plasma}

beat_history = []
energy = 0.0

loop {
    audio = Audio.mic_input()
    fft = Audio.analyze_fft(audio, 8)
    beat = Audio.beat_detect(audio, threshold: 0.6)
    
    # Track energy
    energy = energy * 0.95 + fft.sum() * 0.05
    
    # Beat detection
    if beat {
        beat_history.push(Graphics.get_time())
        Graphics.flash(Graphics.white, 0.2)
    }
    
    # Clean old beats
    beat_history = beat_history.filter(|time| Graphics.get_time() - time < 2.0)
    
    Graphics.clear()
    
    # Background plasma
    Graphics.plasma(speed: energy * 2.0, palette: "cool")
    
    # Beat circles
    for beat_time in beat_history {
        age = Graphics.get_time() - beat_time
        radius = age * 0.3
        alpha = 1.0 - age / 2.0
        
        Graphics.circle(
            x: 0.5, y: 0.5, 
            radius: radius,
            color: Graphics.rgba(1.0, 1.0, 1.0, alpha),
            fill: false
        )
    }
}
```

### Arduino Sensor Garden
```synthesis
# examples/sensor_garden.syn
import Hardware.{arduino_manager}
import Graphics.{clear, plant, set_growth_rate, set_water_level}

arduino = Hardware.arduino_manager()
arduino.connect("/dev/ttyUSB0", 115200)
arduino.setup_environmental_parser()

plants = [
    Graphics.Plant.new(x: 0.2, species: "rose"),
    Graphics.Plant.new(x: 0.5, species: "sunflower"), 
    Graphics.Plant.new(x: 0.8, species: "tulip")
]

loop {
    sensor_data = arduino.read_all_sensors()
    
    # Temperature affects growth
    if let Some(temp) = arduino.get_sensor_value("temperature") {
        growth_rate = (temp - 15.0) / 20.0  # Optimal at 35°C
        Graphics.set_global_growth_rate(growth_rate)
    }
    
    # Humidity affects water level
    if let Some(humidity) = arduino.get_sensor_value("humidity") {
        water_level = humidity / 100.0
        Graphics.set_global_water_level(water_level)
    }
    
    # Light affects flowering
    if let Some(light) = arduino.get_sensor_value("light") {
        light_intensity = light
        Graphics.set_global_light_intensity(light_intensity)
    }
    
    Graphics.clear(Graphics.rgb(0.6, 0.8, 1.0))  # Sky blue
    
    # Ground
    Graphics.rectangle(x: 0.0, y: 0.8, width: 1.0, height: 0.2, color: Graphics.rgb(0.4, 0.2, 0.1))
    
    for plant in plants {
        plant.update()
        plant.render()
    }
    
    # Weather effects
    if arduino.get_sensor_value("pressure").unwrap_or(1013.0) < 1000.0 {
        Graphics.rain_effect(intensity: 0.3)
    }
}
```

## Performance Applications

### Live Looper
```synthesis
# examples/live_looper.syn
import Audio.{line_input, Looper, MIDIInput}
import Hardware.{controller_manager}
import GUI.{window, transport_controls, loop_controls}

midi = Audio.MIDIInput.new()
midi.open_all_devices()

controllers = Hardware.controller_manager()

loopers = [
    Audio.Looper.new(max_length: 16.0),  # 16 bars
    Audio.Looper.new(max_length: 8.0),   # 8 bars  
    Audio.Looper.new(max_length: 4.0),   # 4 bars
    Audio.Looper.new(max_length: 2.0)    # 2 bars
]

current_looper = 0

loop {
    controllers.update()
    audio_in = Audio.line_input()
    
    # Controller mappings
    for i in 0..4 {
        # Face buttons = record/play/stop
        if controllers.is_button_pressed(0, i) {
            match i {
                0 => loopers[current_looper].record(),
                1 => loopers[current_looper].play(),
                2 => loopers[current_looper].stop(),
                3 => loopers[current_looper].clear()
            }
        }
    }
    
    # D-pad selects looper
    if controllers.is_button_pressed(0, "dpad_up") and current_looper > 0 {
        current_looper -= 1
    }
    if controllers.is_button_pressed(0, "dpad_down") and current_looper < 3 {
        current_looper += 1
    }
    
    # Process audio
    mixed_output = audio_in
    for looper in loopers {
        looper.process(audio_in)
        mixed_output += looper.get_output() * 0.7
    }
    
    Audio.output(mixed_output)
    
    # GUI
    GUI.window("Live Looper") {
        content: {
            for i, looper in loopers.enumerate() {
                GUI.loop_controls(looper, i) {
                    active: i == current_looper
                    recording: looper.is_recording()
                    playing: looper.is_playing()
                    position: looper.get_position()
                    length: looper.get_length()
                }
            }
            
            GUI.text("Active Loop: ${current_looper + 1}")
            GUI.text("Status: ${loopers[current_looper].get_status()}")
        }
    }
}
```

### OSC Controller Surface
```synthesis
# examples/osc_surface.syn
import Hardware.{osc_server, osc_client}
import Graphics.{clear, knob, fader, button}
import Audio.{EffectsChain, Reverb, Filter, Compressor}

# Setup OSC
osc_in = Hardware.osc_server()
osc_in.bind("0.0.0.0:8000")
osc_in.start_listening()

osc_out = Hardware.osc_client()
osc_out.connect("127.0.0.1:9000")  # Send to DAW

# Audio effects
reverb = Audio.Reverb.new(48000)
filter = Audio.Filter.new("lowpass", 1000.0, 0.7, 48000)
compressor = Audio.Compressor.new(48000)

effects_chain = Audio.EffectsChain.new()
effects_chain.add_effect(compressor)
effects_chain.add_effect(filter)  
effects_chain.add_effect(reverb)

# Control mappings
controls = {
    reverb_wet: 0.3,
    filter_cutoff: 1000.0,
    comp_threshold: -20.0,
    master_volume: 0.8
}

loop {
    # Receive OSC controls
    if let Some(value) = osc_in.get_float("/reverb/wet") {
        controls.reverb_wet = value
        reverb.set_wet_mix(value)
    }
    
    if let Some(value) = osc_in.get_float("/filter/cutoff") {
        controls.filter_cutoff = value * 10000.0  # Scale 0-1 to 0-10kHz
        filter.set_cutoff(controls.filter_cutoff)
    }
    
    if let Some(value) = osc_in.get_float("/comp/threshold") {
        controls.comp_threshold = -40.0 + value * 40.0  # Scale to -40dB to 0dB
        compressor.set_threshold(controls.comp_threshold)
    }
    
    # Send feedback to OSC controllers
    osc_out.send_float("/led/reverb", if controls.reverb_wet > 0.5 { 1.0 } else { 0.0 })
    osc_out.send_float("/meter/input", Audio.get_input_level())
    osc_out.send_float("/meter/output", Audio.get_output_level())
    
    # Visual interface
    Graphics.clear()
    
    # Knobs
    Graphics.knob("Reverb", controls.reverb_wet, x: 0.2, y: 0.3)
    Graphics.knob("Filter", controls.filter_cutoff / 10000.0, x: 0.5, y: 0.3)
    Graphics.knob("Threshold", (controls.comp_threshold + 40.0) / 40.0, x: 0.8, y: 0.3)
    
    # Master fader
    Graphics.fader("Master", controls.master_volume, x: 0.5, y: 0.7, vertical: true)
    
    # Status indicators
    Graphics.text("OSC In: ${osc_in.get_connection_count()}", x: 0.1, y: 0.9)
    Graphics.text("Audio: ${Audio.get_cpu_usage()}%", x: 0.7, y: 0.9)
}
```

## Running Examples

```bash
# Basic examples
cargo run examples/hello.syn
cargo run examples/audio_viz.syn

# MIDI (requires MIDI device)
cargo run examples/midi_piano.syn

# Hardware (requires game controller)  
cargo run examples/particles.syn

# Arduino (requires Arduino with sensors)
cargo run examples/sensor_garden.syn

# Performance (requires audio interface)
cargo run examples/live_looper.syn

# OSC (requires TouchOSC or similar)
cargo run examples/osc_surface.syn
```

---

# Voorbeelden & Tutorials (Nederlands)

## Basis Voorbeelden

### Hallo Wereld
```synthesis
# examples/hallo.syn
import Graphics.{clear, text, rgb}

loop {
    Graphics.clear(Graphics.rgb(0.1, 0.1, 0.2))
    Graphics.text("Hallo, Synthesis!", x: 0.5, y: 0.5, color: Graphics.rgb(1.0, 1.0, 1.0))
}
```

### Audio Input Visualisatie
```synthesis
# examples/audio_viz.syn
import Audio.{microfoon_input, analyseer_fft}
import Graphics.{clear, balken, spectrum_analyzer}

loop {
    audio = Audio.microfoon_input()
    fft_data = Audio.analyseer_fft(audio, 64)
    
    Graphics.clear()
    Graphics.spectrum_analyzer(fft_data, stijl: "balken", kleur: "regenboog")
}
```

## Creatieve Toepassingen

### Interactief Deeltjes Systeem
```synthesis
# examples/deeltjes.syn
import Hardware.{controller_manager}
import Graphics.{clear, deeltjes, zet_zwaartekracht}

controllers = Hardware.controller_manager()
deeltjes_systeem = Graphics.DeeltjesSysteem.nieuw(1000)

loop {
    controllers.update()
    
    # Linker stick regelt zwaartekracht
    zwaar_x = controllers.krijg_as_waarde(0, 0) * 2.0
    zwaar_y = controllers.krijg_as_waarde(0, 1) * 2.0
    Graphics.zet_zwaartekracht(zwaar_x, zwaar_y)
    
    # A knop reset
    if controllers.is_knop_ingedrukt(0, "A") {
        deeltjes_systeem.reset()
    }
    
    Graphics.clear()
    deeltjes_systeem.update()
    deeltjes_systeem.render()
}
```

## Voorbeelden Uitvoeren

```bash
# Basis voorbeelden
cargo run examples/hallo.syn
cargo run examples/audio_viz.syn

# MIDI (vereist MIDI apparaat)
cargo run examples/midi_piano.syn

# Hardware (vereist game controller)
cargo run examples/deeltjes.syn
```