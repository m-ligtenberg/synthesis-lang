# Synthesis Language Reference

## Syntax

### Variables

```synthesis
# Declaration
name = "Synthesis"
bpm = 120.0
active = true

# Arrays
notes = [60, 64, 67, 72]  
colors = ["red", "green", "blue"]

# Objects
track = {
    name: "Lead",
    volume: 0.8,
    muted: false
}
```

### Functions

```synthesis
# Definition
func calculate_frequency(note) {
    440.0 * (2.0 ** ((note - 69) / 12.0))
}

# Call
freq = calculate_frequency(60)

# With multiple parameters
func mix_audio(left, right, pan) {
    left_gain = (1.0 - pan) * 0.5
    right_gain = (1.0 + pan) * 0.5
    (left * left_gain, right * right_gain)
}
```

### Control Flow

```synthesis
# If statements
if audio_level > 0.5 {
    Graphics.flash(Graphics.white, 0.1)
} else if audio_level > 0.2 {
    Graphics.pulse(Graphics.blue, audio_level)
} else {
    Graphics.fade_to_black()
}

# Loops
for i in 0..16 {
    if pattern[i] {
        Audio.play_drum(i)
    }
}

# While loops
while Audio.is_playing() {
    Audio.process_buffer()
}

# Match expressions
output = match input_type {
    "audio" => Audio.process_input(),
    "midi" => MIDI.process_messages(),
    "osc" => OSC.process_data(),
    _ => 0.0
}
```

### Imports

```synthesis
# Single import
import Audio.mic_input

# Multiple imports
import Graphics.{clear, plasma, starfield}

# Module import
import Audio.*

# Aliased import
import Hardware.controller_manager as Controllers
```

### Classes

```synthesis
class Oscillator {
    frequency: Float = 440.0
    amplitude: Float = 1.0
    phase: Float = 0.0
    
    func new(freq, amp) -> Self {
        Self {
            frequency: freq,
            amplitude: amp,
            phase: 0.0
        }
    }
    
    func process(sample_rate) -> Float {
        output = self.amplitude * sin(2.0 * PI * self.frequency * self.phase / sample_rate)
        self.phase += 1.0
        output
    }
    
    func set_frequency(freq) {
        self.frequency = freq
    }
}

# Usage
osc = Oscillator.new(220.0, 0.5)
sample = osc.process(48000)
```

## Built-in Modules

### Audio

```synthesis
# Input
audio_data = Audio.mic_input()
line_data = Audio.line_input(channel: 1)

# Analysis
fft_data = Audio.analyze_fft(audio_data, bins: 512)
beat_detected = Audio.beat_detect(audio_data, threshold: 0.6)
pitch = Audio.detect_pitch(audio_data)

# MIDI
midi_input = Audio.MIDIInput.new()
midi_input.open_device("USB MIDI Device")
messages = midi_input.get_messages()

# Effects
reverb = Audio.Reverb.new(sample_rate: 48000)
filtered = Audio.Filter.new("lowpass", cutoff: 1000.0)
compressed = Audio.Compressor.new(threshold: -20.0, ratio: 4.0)
```

### Graphics

```synthesis
# Basic operations
Graphics.clear(Graphics.black)
Graphics.fill(Graphics.red)

# Primitives
Graphics.circle(x: 0.5, y: 0.5, radius: 0.1, color: Graphics.blue)
Graphics.rectangle(x: 0.2, y: 0.3, width: 0.6, height: 0.4)
Graphics.line(x1: 0.0, y1: 0.0, x2: 1.0, y2: 1.0)

# Effects
Graphics.plasma(speed: 1.0, palette: Graphics.rainbow)
Graphics.starfield(count: 200, speed: 2.0)
Graphics.particles(count: 1000, gravity: -0.1)

# Colors (0.0 to 1.0 range)
Graphics.rgb(r: 1.0, g: 0.5, b: 0.0)
Graphics.hsv(h: 240.0, s: 1.0, v: 0.8)

# Blend modes
Graphics.set_blend_mode("multiply")
Graphics.set_blend_mode("screen")
Graphics.set_blend_mode("overlay")
```

### GUI

```synthesis
# Window
GUI.window("Audio Mixer", size: [800, 600]) {
    content: {
        # Controls
        volume = GUI.slider("Volume", 0.0, 1.0, 0.7)
        frequency = GUI.knob("Frequency", 20.0, 20000.0, 440.0)
        enabled = GUI.toggle("Enable", true)
        
        # Buttons
        if GUI.button("Play") {
            Audio.start_playback()
        }
        
        # Display
        GUI.text("BPM: ${current_bpm}")
        GUI.progress_bar(Audio.get_playback_position())
    }
}

# Layouts
GUI.horizontal_layout() {
    left: GUI.track_controls()
    right: GUI.mixer_panel()
}

GUI.vertical_layout() {
    top: GUI.transport_controls()
    bottom: GUI.waveform_display()
}
```

### Hardware

```synthesis
# Controllers
controllers = Hardware.controller_manager()
controllers.update()

if controllers.is_button_pressed(0, "A") {
    Audio.trigger_sample("kick.wav")
}

stick_x = controllers.get_axis_value(0, 0)  # Left stick X
stick_y = controllers.get_axis_value(0, 1)  # Left stick Y

# Webcam
webcam = Hardware.webcam_manager()
webcam.start_capture(device: 0)
frame = webcam.get_current_frame()
motion = webcam.analyze_motion()

# Arduino sensors
arduino = Hardware.arduino_manager()
arduino.connect("/dev/ttyUSB0", baud_rate: 115200)
sensor_data = arduino.read_sensors()
temperature = arduino.get_sensor_value("temperature")

# OSC
osc_server = Hardware.osc_server()
osc_server.bind("0.0.0.0:8000")
osc_server.start_listening()
fader_value = osc_server.get_float("/mixer/fader1")
```

### Time

```synthesis
# Timeline
timeline = Time.Timeline.new(bpm: 120.0)
timeline.play()
current_beat = timeline.get_beat_position()

# Sequencer
sequencer = Time.Sequencer.new(steps: 16, bpm: 128.0)
sequencer.set_step(0, velocity: 127)
sequencer.set_step(4, velocity: 100)

if sequencer.is_step_active() {
    step = sequencer.get_current_step()
    Audio.play_drum(step)
}

# Animation
curve = Time.AnimationCurve.new()
curve.add_keyframe(time: 0.0, value: 0.0, ease: "linear")
curve.add_keyframe(time: 2.0, value: 1.0, ease: "ease_out")
current_value = curve.sample(Time.get_time())
```

## Data Types

### Primitives

```synthesis
# Numbers
integer = 42
float = 3.14159
scientific = 1.23e-4

# Strings
text = "Hello, World!"
multiline = """
This is a
multiline string
"""

# Booleans
enabled = true
disabled = false

# Null
empty = null
```

### Collections

```synthesis
# Arrays
numbers = [1, 2, 3, 4, 5]
mixed = [1, "text", true, 3.14]

# Array operations
length = numbers.length
first = numbers[0]
last = numbers[-1]
slice = numbers[1:3]  # [2, 3]

# Objects
person = {
    name: "Alice",
    age: 30,
    skills: ["audio", "graphics"]
}

# Object access
name = person.name
age = person["age"]
```

### Ranges

```synthesis
# Inclusive range
for i in 0..10 {
    print(i)  # 0 to 10
}

# Exclusive range  
for i in 0...10 {
    print(i)  # 0 to 9
}

# Step range
for i in step(0, 100, 10) {
    print(i)  # 0, 10, 20, ..., 90
}
```

## Coordinate System

Synthesis uses percentage-based coordinates (0.0 to 1.0):

```synthesis
# Screen positions
center = (0.5, 0.5)
top_left = (0.0, 0.0)  
bottom_right = (1.0, 1.0)

# Audio ranges
silent = 0.0
unity_gain = 1.0
doubled = 2.0

# Colors (RGB, 0.0 to 1.0)
red = Graphics.rgb(1.0, 0.0, 0.0)
half_bright = Graphics.rgb(0.5, 0.5, 0.5)
```

## Real-time Programming

### Main Loop

```synthesis
# Basic loop
loop {
    # Process input
    audio = Audio.mic_input()
    
    # Update graphics  
    Graphics.clear()
    Graphics.plasma(speed: audio_level)
    
    # Frame timing handled automatically
}
```

### Stream Processing

```synthesis
# Audio stream
audio_stream = Audio.create_stream(sample_rate: 48000, buffer_size: 256)

audio_stream.on_buffer(|buffer| {
    for i in 0..buffer.length {
        # Process each sample
        buffer[i] = apply_effects(buffer[i])
    }
})
```

### Event Handling

```synthesis
# MIDI events
midi_input.on_message(|message| {
    match message.type {
        "note_on" => Audio.play_note(message.note, message.velocity),
        "note_off" => Audio.stop_note(message.note),
        "control_change" => process_cc(message.controller, message.value)
    }
})

# GUI events
GUI.on_button_click("record", || {
    if is_recording {
        stop_recording()
    } else {
        start_recording()
    }
})
```

## Error Handling

```synthesis
# Try-catch
result = try {
    Audio.load_sample("kick.wav")
} catch error {
    print("Failed to load sample: ${error}")
    Audio.generate_sine_wave(60.0, 1.0)  # Fallback
}

# Optional values
sample = Audio.try_load_sample("snare.wav")
if sample.is_some() {
    Audio.play(sample.unwrap())
} else {
    print("Sample not found")
}

# Result types
func divide(a, b) -> Result<Float, String> {
    if b == 0.0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}
```

## Performance

### Memory Management

```synthesis
# Automatic memory management
# No manual allocation/deallocation needed

# Large buffer hints
audio_buffer = Audio.create_buffer(size: 4096, hint: "large")

# Pooled objects for real-time code
particle_pool = Graphics.create_particle_pool(1000)
particle = particle_pool.get()
particle_pool.release(particle)
```

### Real-time Safety

```synthesis
# Audio thread - no allocations allowed
audio_callback(|buffer| {
    # ✓ Process existing data
    for sample in buffer {
        sample = filter.process(sample)
    }
    
    # ✗ Don't allocate memory
    # new_array = [1, 2, 3]  # This would cause audio dropouts
})

# Main thread - allocations OK
main_thread {
    samples = Audio.load_directory("samples/")  # ✓ OK here
}
```

---

# Synthesis Taalreferentie (Nederlands)

## Syntaxis

### Variabelen

```synthesis
# Declaratie
naam = "Synthesis"
bpm = 120.0
actief = true

# Arrays
noten = [60, 64, 67, 72]
kleuren = ["rood", "groen", "blauw"]

# Objecten
track = {
    naam: "Lead",
    volume: 0.8,
    gedempt: false
}
```

### Functies

```synthesis
# Definitie
func bereken_frequentie(noot) {
    440.0 * (2.0 ** ((noot - 69) / 12.0))
}

# Aanroep
freq = bereken_frequentie(60)

# Met meerdere parameters
func mix_audio(links, rechts, pan) {
    links_gain = (1.0 - pan) * 0.5
    rechts_gain = (1.0 + pan) * 0.5
    (links * links_gain, rechts * rechts_gain)
}
```

### Controle Structuren

```synthesis
# If statements
if audio_niveau > 0.5 {
    Graphics.flits(Graphics.wit, 0.1)
} else if audio_niveau > 0.2 {
    Graphics.puls(Graphics.blauw, audio_niveau)
} else {
    Graphics.fade_naar_zwart()
}

# Loops
for i in 0..16 {
    if patroon[i] {
        Audio.speel_drum(i)
    }
}

# While loops
while Audio.is_aan_het_spelen() {
    Audio.verwerk_buffer()
}
```

## Ingebouwde Modules

### Audio

```synthesis
# Input
audio_data = Audio.microfoon_input()
lijn_data = Audio.lijn_input(kanaal: 1)

# Analyse
fft_data = Audio.analyseer_fft(audio_data, bins: 512)
beat_gedetecteerd = Audio.detecteer_beat(audio_data, drempel: 0.6)

# MIDI
midi_input = Audio.MIDIInput.nieuw()
midi_input.open_apparaat("USB MIDI Apparaat")
berichten = midi_input.krijg_berichten()

# Effecten
galm = Audio.Galm.nieuw(sample_rate: 48000)
gefilterd = Audio.Filter.nieuw("laagdoorlaatfilter", cutoff: 1000.0)
gecomprimeerd = Audio.Compressor.nieuw(drempel: -20.0, verhouding: 4.0)
```

### Graphics

```synthesis
# Basis operaties
Graphics.leeg(Graphics.zwart)
Graphics.vul(Graphics.rood)

# Primitieven
Graphics.cirkel(x: 0.5, y: 0.5, straal: 0.1, kleur: Graphics.blauw)
Graphics.rechthoek(x: 0.2, y: 0.3, breedte: 0.6, hoogte: 0.4)
Graphics.lijn(x1: 0.0, y1: 0.0, x2: 1.0, y2: 1.0)

# Effecten
Graphics.plasma(snelheid: 1.0, palet: Graphics.regenboog)
Graphics.sterrenveld(aantal: 200, snelheid: 2.0)
Graphics.deeltjes(aantal: 1000, zwaartekracht: -0.1)
```

### GUI

```synthesis
# Venster
GUI.venster("Audio Mixer", grootte: [800, 600]) {
    inhoud: {
        # Controls
        volume = GUI.schuif("Volume", 0.0, 1.0, 0.7)
        frequentie = GUI.knop("Frequentie", 20.0, 20000.0, 440.0)
        ingeschakeld = GUI.schakelaar("Inschakelen", true)
        
        # Knoppen
        if GUI.knop("Afspelen") {
            Audio.start_afspelen()
        }
        
        # Weergave
        GUI.tekst("BPM: ${huidige_bpm}")
        GUI.voortgangsbalk(Audio.krijg_afspeelpositie())
    }
}
```

## Datatypes

### Primitieven

```synthesis
# Nummers
geheel_getal = 42
komma_getal = 3.14159

# Tekst
tekst = "Hallo, Wereld!"

# Booleaans
ingeschakeld = true
uitgeschakeld = false

# Leeg
leeg = null
```

### Coördinatensysteem

Synthesis gebruikt percentage-gebaseerde coördinaten (0.0 tot 1.0):

```synthesis
# Schermposities
centrum = (0.5, 0.5)
linksboben = (0.0, 0.0)
rechtsonder = (1.0, 1.0)
```