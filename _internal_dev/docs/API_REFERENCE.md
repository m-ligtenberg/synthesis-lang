# API Reference

## Audio Module

### Input/Output

#### `Audio.mic_input() -> Array<Float>`
Capture audio from default microphone.
```synthesis
audio_data = Audio.mic_input()
```

#### `Audio.line_input(channel: Int) -> Array<Float>`  
Capture from line input channel.
```synthesis
stereo_in = Audio.line_input(channel: 1)
```

#### `Audio.output(data: Array<Float>)`
Send audio to default output.
```synthesis
Audio.output(processed_audio)
```

### Analysis

#### `Audio.analyze_fft(data: Array<Float>, bins: Int) -> Array<Float>`
Perform FFT analysis.
- `bins`: Number of frequency bins (power of 2)
- Returns: Magnitude spectrum 0.0-1.0

```synthesis
fft_data = Audio.analyze_fft(audio, 512)
bass_energy = fft_data[0..8].sum()
```

#### `Audio.beat_detect(data: Array<Float>, threshold: Float) -> Bool`
Detect beats in audio signal.
```synthesis
is_beat = Audio.beat_detect(audio, threshold: 0.6)
```

#### `Audio.detect_pitch(data: Array<Float>) -> Float`
Extract fundamental frequency.
```synthesis
freq = Audio.detect_pitch(audio)  # Returns Hz
```

### MIDI

#### `Audio.MIDIInput.new() -> MIDIInput`
Create MIDI input handler.
```synthesis
midi_in = Audio.MIDIInput.new()
midi_in.open_all_devices()
```

#### `MIDIInput.get_messages() -> Array<MIDIMessage>`
Get pending MIDI messages.
```synthesis
messages = midi_in.get_messages()
for msg in messages {
    # Process message
}
```

### Effects

#### `Audio.Reverb.new(sample_rate: Float) -> Reverb`
Create reverb effect.
```synthesis
reverb = Audio.Reverb.new(48000)
reverb.set_wet_mix(0.3)
wet_signal = reverb.process(dry_signal)
```

#### `Audio.Filter.new(type: String, cutoff: Float, resonance: Float, sample_rate: Float) -> Filter`
Create filter effect.
- `type`: "lowpass", "highpass", "bandpass"
```synthesis
lpf = Audio.Filter.new("lowpass", 1000.0, 0.7, 48000)
filtered = lpf.process(input)
```

#### `Audio.Compressor.new(sample_rate: Float) -> Compressor`
Create dynamics compressor.
```synthesis
comp = Audio.Compressor.new(48000)
comp.set_threshold(-20.0)
comp.set_ratio(4.0)
compressed = comp.process(input)
```

---

## Graphics Module

### Basic Operations

#### `Graphics.clear(color: Color)`
Clear screen with color.
```synthesis
Graphics.clear(Graphics.black)
Graphics.clear(Graphics.rgb(0.2, 0.2, 0.3))
```

#### `Graphics.set_blend_mode(mode: String)`
Set blending mode for subsequent draws.
- Modes: "normal", "add", "multiply", "screen", "overlay"
```synthesis
Graphics.set_blend_mode("add")
```

### Primitives

#### `Graphics.circle(x: Float, y: Float, radius: Float, color: Color, fill: Bool)`
Draw circle at position (0.0-1.0 coordinates).
```synthesis
Graphics.circle(x: 0.5, y: 0.5, radius: 0.1, color: Graphics.red, fill: true)
```

#### `Graphics.rectangle(x: Float, y: Float, width: Float, height: Float, color: Color)`
Draw rectangle.
```synthesis
Graphics.rectangle(x: 0.2, y: 0.3, width: 0.6, height: 0.4, color: Graphics.blue)
```

#### `Graphics.line(x1: Float, y1: Float, x2: Float, y2: Float, color: Color, width: Float)`
Draw line between two points.
```synthesis
Graphics.line(x1: 0.0, y1: 0.0, x2: 1.0, y2: 1.0, color: Graphics.white, width: 2.0)
```

### Effects

#### `Graphics.plasma(speed: Float, palette: String)`
Draw animated plasma effect.
- `palette`: "rainbow", "fire", "cool", "neon"
```synthesis
Graphics.plasma(speed: 2.0, palette: "rainbow")
```

#### `Graphics.starfield(count: Int, speed: Float)`
Animated starfield.
```synthesis
Graphics.starfield(count: 500, speed: 1.5)
```

#### `Graphics.particles(count: Int, gravity: Float)`
Particle system.
```synthesis
Graphics.particles(count: 1000, gravity: -0.1)
```

### Colors

#### `Graphics.rgb(r: Float, g: Float, b: Float) -> Color`
Create RGB color (0.0-1.0).
```synthesis
red = Graphics.rgb(1.0, 0.0, 0.0)
```

#### `Graphics.hsv(h: Float, s: Float, v: Float) -> Color`
Create HSV color (h: 0-360, s,v: 0.0-1.0).
```synthesis
blue = Graphics.hsv(240.0, 1.0, 1.0)
```

#### `Graphics.rgba(r: Float, g: Float, b: Float, a: Float) -> Color`
RGB with alpha transparency.
```synthesis
transparent_red = Graphics.rgba(1.0, 0.0, 0.0, 0.5)
```

---

## GUI Module

### Windows

#### `GUI.window(title: String, size: [Int, Int], theme: String)`
Create application window.
```synthesis
GUI.window("My App", size: [800, 600], theme: "dark") {
    content: {
        # Window contents
    }
}
```

### Controls

#### `GUI.slider(label: String, min: Float, max: Float, value: Float) -> Float`
Horizontal slider control.
```synthesis
volume = GUI.slider("Volume", 0.0, 1.0, 0.7)
```

#### `GUI.knob(label: String, min: Float, max: Float, value: Float) -> Float`
Rotary knob control.
```synthesis
frequency = GUI.knob("Frequency", 20.0, 20000.0, 440.0)
```

#### `GUI.button(text: String, style: String) -> Bool`
Push button, returns true when clicked.
```synthesis
if GUI.button("Play", style: "primary") {
    Audio.start_playback()
}
```

#### `GUI.toggle(label: String, value: Bool) -> Bool`
Toggle switch.
```synthesis
enabled = GUI.toggle("Enable Effect", enabled)
```

### Layouts

#### `GUI.horizontal_layout()`
Arrange children horizontally.
```synthesis
GUI.horizontal_layout() {
    left: GUI.controls_panel()
    right: GUI.display_panel()
}
```

#### `GUI.vertical_layout()`
Arrange children vertically.
```synthesis
GUI.vertical_layout() {
    top: GUI.menu_bar()
    bottom: GUI.status_bar()
}
```

---

## Hardware Module

### Controllers

#### `Hardware.controller_manager() -> ControllerManager`
Access game controllers.
```synthesis
controllers = Hardware.controller_manager()
controllers.update()
```

#### `ControllerManager.is_button_pressed(id: Int, button: String) -> Bool`
Check button state.
- `button`: "A", "B", "X", "Y", "left_bumper", "right_bumper"
```synthesis
if controllers.is_button_pressed(0, "A") {
    Audio.trigger_sample()
}
```

#### `ControllerManager.get_axis_value(id: Int, axis: Int) -> Float`
Get analog axis value (-1.0 to 1.0).
- `axis`: 0=left_x, 1=left_y, 2=right_x, 3=right_y, 4=left_trigger, 5=right_trigger
```synthesis
stick_x = controllers.get_axis_value(0, 0)
```

### Arduino

#### `Hardware.arduino_manager() -> ArduinoManager`
Manage Arduino connections.
```synthesis
arduino = Hardware.arduino_manager()
arduino.connect("/dev/ttyUSB0", 115200)
```

#### `ArduinoManager.read_sensors() -> Array<SensorData>`
Read all sensor data.
```synthesis
sensors = arduino.read_sensors()
for sensor in sensors {
    print("${sensor.sensor_id}: ${sensor.value}")
}
```

#### `ArduinoManager.get_sensor_value(id: String) -> Option<Float>`
Get specific sensor value.
```synthesis
temp = arduino.get_sensor_value("temperature")
if temp.is_some() {
    print("Temperature: ${temp.unwrap()}°C")
}
```

### Webcam

#### `Hardware.webcam_manager() -> WebcamManager`
Access webcam functionality.
```synthesis
webcam = Hardware.webcam_manager()
webcam.start_capture(device: 0)
```

#### `WebcamManager.analyze_motion() -> Option<MotionData>`
Detect motion in video feed.
```synthesis
motion = webcam.analyze_motion()
if motion.is_some() {
    amount = motion.unwrap().motion_amount
    print("Motion: ${amount}")
}
```

### OSC

#### `Hardware.osc_server() -> OscServer`
Create OSC server for receiving.
```synthesis
osc = Hardware.osc_server()
osc.bind("0.0.0.0:8000")
osc.start_listening()
```

#### `OscServer.get_float(address: String) -> Option<Float>`
Get float parameter by OSC address.
```synthesis
fader = osc.get_float("/mixer/fader1")
```

#### `Hardware.osc_client() -> OscClient`
Create OSC client for sending.
```synthesis
client = Hardware.osc_client()
client.connect("127.0.0.1:9000")
client.send_float("/synth/frequency", 440.0)
```

---

## Time Module

### Timeline

#### `Time.Timeline.new(bpm: Float) -> Timeline`
Create timeline for sequencing.
```synthesis
timeline = Time.Timeline.new(bpm: 120.0)
timeline.play()
```

#### `Timeline.get_beat_position() -> Float`
Get current beat position.
```synthesis
beat = timeline.get_beat_position()
```

### Sequencer

#### `Time.Sequencer.new(steps: Int, bpm: Float) -> Sequencer`
Create step sequencer.
```synthesis
seq = Time.Sequencer.new(steps: 16, bpm: 128.0)
seq.set_step(0, velocity: 127)
```

#### `Sequencer.is_step_active() -> Bool`
Check if current step should trigger.
```synthesis
if seq.is_step_active() {
    step = seq.get_current_step()
    Audio.play_drum(step)
}
```

### Animation

#### `Time.AnimationCurve.new() -> AnimationCurve`
Create animation curve for parameter automation.
```synthesis
curve = Time.AnimationCurve.new()
curve.add_keyframe(time: 0.0, value: 0.0, ease: "linear")
curve.add_keyframe(time: 2.0, value: 1.0, ease: "ease_out")
```

#### `AnimationCurve.sample(time: Float) -> Float`
Sample curve at specific time.
```synthesis
current_value = curve.sample(Time.get_time())
```

---

# API Referentie (Nederlands)

## Audio Module

### Input/Output

#### `Audio.microfoon_input() -> Array<Float>`
Audio vastleggen van standaard microfoon.

#### `Audio.lijn_input(kanaal: Int) -> Array<Float>`
Vastleggen van lijn input kanaal.

#### `Audio.output(data: Array<Float>)`
Audio sturen naar standaard output.

### Analyse

#### `Audio.analyseer_fft(data: Array<Float>, bins: Int) -> Array<Float>`
FFT analyse uitvoeren.
- `bins`: Aantal frequentie bins (macht van 2)
- Retourneert: Magnitude spectrum 0.0-1.0

#### `Audio.detecteer_beat(data: Array<Float>, drempel: Float) -> Bool`
Beats detecteren in audio signaal.

## Graphics Module

### Basis Operaties

#### `Graphics.leeg(kleur: Color)`
Scherm legen met kleur.

#### `Graphics.zet_blend_mode(mode: String)`
Blend modus instellen.
- Modi: "normal", "add", "multiply", "screen", "overlay"

### Primitieven

#### `Graphics.cirkel(x: Float, y: Float, straal: Float, kleur: Color, vulling: Bool)`
Cirkel tekenen op positie (0.0-1.0 coördinaten).

#### `Graphics.rechthoek(x: Float, y: Float, breedte: Float, hoogte: Float, kleur: Color)`
Rechthoek tekenen.

### Kleuren

#### `Graphics.rgb(r: Float, g: Float, b: Float) -> Color`
RGB kleur maken (0.0-1.0).

#### `Graphics.hsv(h: Float, s: Float, v: Float) -> Color`
HSV kleur maken (h: 0-360, s,v: 0.0-1.0).