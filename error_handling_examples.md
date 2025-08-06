# Error Handling Examples

This document showcases the enhanced error translation system in Synthesis, demonstrating how Rust errors are converted to user-friendly messages.

## New Error Pattern Coverage

### 1. Function Argument Errors (E0061)

**Before (Raw Rust):**
```
error[E0061]: this function takes 2 arguments but 1 argument was supplied
  --> src/main.rs:5:5
   |
5  |     Audio.analyze_fft(audio_data);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected 2 arguments, found 1
```

**After (Synthesis Translation):**
```
🎵 Synthesis Error: Function 'analyze_fft' called with wrong number of arguments

💡 Suggestions:
   • Check the function documentation for correct parameters
   • Make sure you're passing the right number of arguments
   • Some parameters might be optional or have default values

📚 Learn more: https://synthesis-lang.org/docs/functions
```

### 2. Trait Bound Errors (E0277)

**Before (Raw Rust):**
```
error[E0277]: the trait bound `Text: AudioProcessor` is not satisfied
  --> src/main.rs:8:5
   |
8  |     text_data.process_audio();
   |     ^^^^^^^^^ the trait `AudioProcessor` is not implemented for `Text`
```

**After (Synthesis Translation):**
```
🔗 Synthesis Error: This operation isn't supported for this type of data

💡 Suggestions:
   • Check if you're using compatible data types
   • Some operations only work with specific types like Numbers or Audio
   • Try converting your data to the right type first

📚 Learn more: https://synthesis-lang.org/docs/types#operations
```

### 3. Use After Move Errors (E0382)

**Before (Raw Rust):**
```
error[E0382]: use of moved value: `audio_stream`
  --> src/main.rs:10:5
   |
9  |     let processed = audio_stream.apply_reverb();
   |                     ------------ value moved here
10 |     let analyzed = audio_stream.analyze_fft();
   |                    ^^^^^^^^^^^^ value used here after move
```

**After (Synthesis Translation):**
```
🎵 Synthesis Error: Variable 'audio_stream' was already used and can't be used again

💡 Suggestions:
   • In Synthesis, some operations consume their inputs
   • Try using .clone() if you need to use the same data multiple times
   • Consider restructuring your code to avoid reusing consumed values

📚 Learn more: https://synthesis-lang.org/docs/variables#ownership
```

### 4. Immutable Variable Assignment (E0384)

**Before (Raw Rust):**
```
error[E0384]: cannot assign twice to immutable variable `frequency`
  --> src/main.rs:8:5
   |
6  |     let frequency = 440.0;
   |         --------- first assignment to `frequency`
8  |     frequency = 880.0;
   |     ^^^^^^^^^^^^^^^^^ cannot assign twice to immutable variable
```

**After (Synthesis Translation):**
```
🎵 Synthesis Error: Variable 'frequency' cannot be changed after it's set

💡 Suggestions:
   • Use 'mut' keyword when creating the variable: `let mut variable = ...`
   • Variables in Synthesis are unchangeable by default for safety
   • Consider using a new variable name if you don't need mutability

📚 Learn more: https://synthesis-lang.org/docs/variables#mutability
```

### 5. Runtime Panic Handling

**Before (Raw Rust):**
```
thread 'main' panicked at 'index out of bounds: the len is 8 but the index is 10'
```

**After (Synthesis Translation):**
```
🎵 Synthesis Error: Trying to access data that doesn't exist

💡 Suggestions:
   • Check that your list or array has enough items
   • Array indices start at 0, not 1
   • Use .len() to check the size before accessing elements

📚 Learn more: https://synthesis-lang.org/docs/data#arrays
```

### 6. Division by Zero

**Before (Raw Rust):**
```
attempt to divide by zero
```

**After (Synthesis Translation):**
```
🎵 Synthesis Error: Cannot divide by zero

💡 Suggestions:
   • Check that your divisor is not zero before dividing
   • Use conditional logic: `if divisor != 0 { result = a / divisor }`
   • Consider using small non-zero values instead of exact zero
```

## New Stream-Specific Errors

### Stream Connection Error
```
🔌 Synthesis Error: Cannot connect stream 'audio_input' to 'graphics_output'

💡 Suggestions:
   • Check that the stream types are compatible
   • Audio streams connect to Audio, Graphics to Graphics, etc.
   • Use conversion functions if you need to change stream types

📚 Learn more: https://synthesis-lang.org/docs/streams#connections
```

### Buffer Underrun Error
```
🌊 Synthesis Error: Audio stream 'main_audio' buffer underrun - not enough data to maintain real-time

💡 Suggestions:
   • Increase buffer size with --buffer-size option
   • Optimize your processing code for better performance
   • Reduce sample rate if high quality isn't needed

📚 Learn more: https://synthesis-lang.org/docs/performance#audio-buffers
```

### Real-time Violation
```
⚡ Synthesis Error: Operation 'audio_processing' took 2.50ms which violates real-time constraints

💡 Suggestions:
   • Use lighter processing or increase buffer size
   • Enable optimizations with -O creative flag
   • Consider using background processing for heavy operations

📚 Learn more: https://synthesis-lang.org/docs/performance#real-time
```

## Context-Aware Suggestions

The error system now provides context-aware suggestions based on the error content:

### Import Errors
When an error mentions "import", the system adds:
- "Import statements should be at the top of your file"
- "Use: `import ModuleName` or `import ModuleName.{function1, function2}`"

### Loop Errors  
When an error mentions "loop", the system adds:
- "Loop blocks need curly braces: `loop { ... }`"
- "Make sure your loop has a body with actual code"

### Type Conversion Errors
When Number/Text type mismatch occurs, the system adds:
- "Use Number.parse(text) to convert text to numbers"
- "Or use text interpolation: `\"The value is: ${number}\"`"

### Audio Device Errors
When audio device errors occur, the system adds:
- "Try plugging/unplugging your audio device"
- "Check audio device settings in your system"
- "Close other applications that might be using audio"

## Integration Utilities

### Error Translation Macro
```rust
use synthesis_error_from;

// Automatically translates any error with context
let result = synthesis_error_from!(rust_error, "parsing audio file");
```

### Safe Execution
```rust
use crate::errors::integration::execute_with_translation;

let result = execute_with_translation(|| {
    // Some Rust operation that might fail
    risky_operation()
}, "audio processing")?;
```

### Panic Catching
```rust
use catch_rust_panic;

let result = catch_rust_panic!(
    potentially_panicking_operation(),
    "graphics rendering"
);
```

## Benefits Achieved

1. **Zero Rust Exposure**: Users never see internal Rust error messages
2. **Educational**: Every error teaches users about Synthesis concepts
3. **Actionable**: Specific suggestions help users fix problems quickly
4. **Contextual**: Suggestions adapt based on what the user was trying to do
5. **Professional**: Consistent formatting with emojis and documentation links
6. **Comprehensive**: Covers compilation, runtime, and panic scenarios

The error translation system makes Synthesis feel like a purpose-built creative programming language rather than a Rust wrapper, fulfilling the key goal from the V1 roadmap.