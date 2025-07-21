# Hardware Integration

## MIDI Controllers

### Setup
```synthesis
import Hardware.{midi_input, midi_output}

midi_in = Hardware.midi_input()
midi_in.open_all_devices()

midi_out = Hardware.midi_output()
midi_out.open_device("IAC Driver")
```

### Message Processing
```synthesis
messages = midi_in.get_messages()
for msg in messages {
    match msg.type {
        "note_on" => Audio.play_note(msg.note, msg.velocity),
        "note_off" => Audio.stop_note(msg.note),
        "control_change" => process_cc(msg.controller, msg.value),
        "pitch_bend" => Audio.set_pitch_bend(msg.value)
    }
}
```

### Supported Devices
- **Keyboard Controllers**: Akai MPK, Novation Launchkey
- **Pad Controllers**: Akai MPC, Native Instruments Maschine  
- **DJ Controllers**: Pioneer DDJ, Numark Party Mix
- **Modular**: Expert Sleepers, Doepfer

## Game Controllers

### Xbox/PlayStation
```synthesis
controllers = Hardware.controller_manager()
controllers.update()

# Face buttons
if controllers.is_button_pressed(0, "A") {
    Audio.trigger_sample("kick.wav")
}

# Analog sticks  
left_x = controllers.get_axis_value(0, 0)
left_y = controllers.get_axis_value(0, 1)

Graphics.set_brush_position(left_x, left_y)
```

### Custom Mappings
```synthesis
mapping = Hardware.creative_controller_mapping(0)
mapping.map_axis("volume", axis: 5, range: [0.0, 1.0])
mapping.map_button("record", button: 0, mode: "toggle")

volume = mapping.evaluate("volume", controllers)
```

## Arduino Integration

### Serial Communication
```synthesis
arduino = Hardware.arduino_manager()
arduino.connect("/dev/ttyUSB0", 115200)

# Read sensor data
sensors = arduino.read_sensors()
temperature = arduino.get_sensor_value("temperature")
```

### Data Format
Arduino should send CSV format:
```
temperature,23.5,temperature
humidity,65.0,humidity  
light,512,analog
button,1,digital
```

### Example Arduino Code
```cpp
void setup() {
    Serial.begin(115200);
}

void loop() {
    float temp = 25.0; // Read from sensor
    int light = analogRead(A0);
    
    Serial.print("temperature,");
    Serial.print(temp);
    Serial.println(",temperature");
    
    Serial.print("light,");
    Serial.print(light / 1023.0);
    Serial.println(",analog");
    
    delay(50);
}
```

## Webcam

### Setup
```synthesis
webcam = Hardware.webcam_manager()
webcam.start_capture(device: 0)
```

### Motion Detection
```synthesis
loop {
    webcam.update()
    
    motion = webcam.analyze_motion()
    if motion.motion_amount > 0.1 {
        Audio.trigger_sample("clap.wav")
        
        # Map motion center to sound position
        Audio.set_pan(motion.motion_center.x * 2.0 - 1.0)
    }
}
```

### Color Analysis
```synthesis
color_data = webcam.analyze_color()
hue = Hardware.color_to_hue(color_data.dominant_color)
Audio.set_filter_cutoff(hue * 10.0 + 200.0)
```

## OSC Protocol

### Server Setup
```synthesis
osc_server = Hardware.osc_server()
osc_server.bind("0.0.0.0:8000")
osc_server.start_listening()
```

### TouchOSC Integration
```synthesis
# Receive from TouchOSC app
fader1 = osc_server.get_float("/1/fader1")
xy_pad = osc_server.get_parameter("/1/xy1")

if xy_pad {
    x_pos = xy_pad.value[0]
    y_pos = xy_pad.value[1]
    Graphics.set_particle_center(x_pos, y_pos)
}
```

### Send to External Software
```synthesis
osc_client = Hardware.osc_client()
osc_client.connect("127.0.0.1:9000")

# Send to Ableton Live
osc_client.send_float("/live/song/set/tempo", current_bpm)
osc_client.send_message("/live/song/start_playing", [])
```

## Supported Hardware

### MIDI Keyboards
- Akai MPK Mini/249/261
- Novation Launchkey 25/49/61
- M-Audio Keystation
- Arturia MiniLab

### Controllers  
- Xbox One/Series Controllers
- PlayStation 4/5 DualShock/DualSense
- Nintendo Switch Pro Controller
- Generic USB HID controllers

### Arduino Sensors
- **Temperature**: DHT22, DS18B20
- **Humidity**: DHT22, SHT30
- **Light**: Photoresistor, TSL2561
- **Motion**: PIR, MPU6050
- **Distance**: HC-SR04, VL53L0X
- **Pressure**: BMP280, BMP180

### Webcams
- USB UVC compatible cameras
- Logitech C920/C922/C930
- Built-in laptop cameras
- Raspberry Pi Camera Module

### OSC Applications
- **TouchOSC**: Mobile control surfaces
- **Lemur**: Advanced touch interfaces  
- **Max/MSP**: Visual programming
- **Pure Data**: Audio processing
- **Ableton Live**: DAW integration
- **Reaper**: DAW control

## Linux Device Setup

### MIDI Permissions
```bash
# Add user to audio group
sudo usermod -a -G audio $USER

# Check MIDI devices
aconnect -l
amidi -l
```

### Serial Permissions
```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER

# Check serial ports
ls -la /dev/ttyUSB* /dev/ttyACM*
```

### Webcam Permissions
```bash
# Check video devices
ls -la /dev/video*

# Test camera
ffplay /dev/video0
```

### USB Controller Detection
```bash
# List USB devices
lsusb

# Test controller input
evtest /dev/input/event*
```

---

# Hardware Integratie (Nederlands)

## MIDI Controllers

### Setup
```synthesis
import Hardware.{midi_input, midi_output}

midi_in = Hardware.midi_input()
midi_in.open_alle_apparaten()

midi_out = Hardware.midi_output()
midi_out.open_apparaat("IAC Driver")
```

### Bericht Verwerking
```synthesis
berichten = midi_in.krijg_berichten()
for bericht in berichten {
    match bericht.type {
        "note_on" => Audio.speel_noot(bericht.noot, bericht.velocity),
        "note_off" => Audio.stop_noot(bericht.noot),
        "control_change" => verwerk_cc(bericht.controller, bericht.waarde)
    }
}
```

## Game Controllers

### Xbox/PlayStation
```synthesis
controllers = Hardware.controller_manager()
controllers.update()

# Gezichtsknoppen
if controllers.is_knop_ingedrukt(0, "A") {
    Audio.trigger_sample("kick.wav")
}

# Analoge sticks
links_x = controllers.krijg_as_waarde(0, 0)
links_y = controllers.krijg_as_waarde(0, 1)
```

## Arduino Integratie

### Seriële Communicatie
```synthesis
arduino = Hardware.arduino_manager()
arduino.verbind("/dev/ttyUSB0", 115200)

# Sensor data lezen
sensors = arduino.lees_sensoren()
temperatuur = arduino.krijg_sensor_waarde("temperatuur")
```

### Data Formaat
Arduino moet CSV formaat sturen:
```
temperatuur,23.5,temperature
luchtvochtigheid,65.0,humidity
licht,512,analog
```

## Webcam

### Setup
```synthesis
webcam = Hardware.webcam_manager()
webcam.start_opname(apparaat: 0)
```

### Bewegingsdetectie
```synthesis
loop {
    webcam.update()
    
    beweging = webcam.analyseer_beweging()
    if beweging.bewegings_hoeveelheid > 0.1 {
        Audio.trigger_sample("clap.wav")
    }
}
```

## OSC Protocol

### Server Setup
```synthesis
osc_server = Hardware.osc_server()
osc_server.bind("0.0.0.0:8000")
osc_server.start_luisteren()
```

### TouchOSC Integratie
```synthesis
# Ontvangen van TouchOSC app
fader1 = osc_server.krijg_float("/1/fader1")
xy_pad = osc_server.krijg_parameter("/1/xy1")
```

## Ondersteunde Hardware

### MIDI Keyboards
- Akai MPK Mini/249/261
- Novation Launchkey 25/49/61
- M-Audio Keystation
- Arturia MiniLab

### Arduino Sensoren
- **Temperatuur**: DHT22, DS18B20
- **Luchtvochtigheid**: DHT22, SHT30  
- **Licht**: Fotoweerstand, TSL2561
- **Beweging**: PIR, MPU6050
- **Afstand**: HC-SR04, VL53L0X