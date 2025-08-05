# Synthesis Language - What Users Actually Need

## 🎯 User-Facing Files (Keep in Main Repo)

### **Essential Files**
- `README.md` - Main project description
- `install.syn` - One-line installer script
- `LICENSE` - License information

### **Source Code (Core Language)**
- `src/` - Complete source code
  - `compiler/` - Synthesis compiler
  - `parser/` - Language parser
  - `runtime/` - Execution engine
  - `errors.rs` - User-friendly error system
  - `modules/` - Built-in language modules
  - `bin/synthc.rs` - Compiler binary
  - All other core language files

### **Build System**
- `build.synt` - Rust build configuration
- `build.lock` - Dependency lock file

### **Examples** 
- `examples/` - Simple .syn example files
  - `hello.syn`
  - `audio_visualizer.syn`
  - `math_demo.syn`
  - Other basic examples

### **Package Management**
- `syn-pkg` - Package manager binary (if built)
- `package.syn` - Package configuration example

## 🗂️ Move to `_internal_dev/` (Development-Only Files)

### **Documentation Development**
- `docs/` - Internal markdown docs
- `docs_website/` - Astro documentation site
- `docs-yew/` - Yew documentation experiments

### **Analysis & Planning Documents**
- `CLAUDE.md` - Claude development instructions
- `COMPILER_DESIGN.md` - Internal compiler design
- `INDEPENDENCE_ROADMAP.md` - Development roadmap
- `STANDALONE_IMPLEMENTATION.md` - Implementation notes
- `Enhancing Synthesis Language*.md` - Analysis documents
- `new_features.md` - Feature planning

### **Development Utilities**
- `benchmarks/` - Performance benchmarks
- `tests/` - Rust test files
- `debug_*.rs` - Debug utilities
- `code-sandbox/` - Development sandbox
- `tailcast-website/` - Marketing website
- `test-visualizer/` - Test project

### **Build Artifacts**
- `target/` - Rust build output
- `node_modules/` - Node.js dependencies (in docs sites)
- Various `package-lock.json`, `pnpm-lock.yaml` files

## 📦 Final User Repository Structure

```
synthesis-lang/
├── README.md                 # "🎨 Synthesis Language - Universal Creative Programming"
├── LICENSE                   # MIT/Apache license
├── install.syn               # One-line installer
├── Cargo.toml               # Build configuration
├── Cargo.lock               # Dependencies
├── src/                     # Core language source
│   ├── main.rs
│   ├── lib.rs
│   ├── compiler/
│   ├── parser/
│   ├── runtime/
│   ├── modules/
│   └── bin/synthc.rs
├── examples/                # Simple examples for users
│   ├── hello.syn
│   ├── audio_visualizer.syn
│   └── math_demo.syn
├── package.syn              # Package example
└── _internal_dev/           # Development files (hidden)
    ├── docs/
    ├── docs_website/
    ├── analysis_documents/
    ├── benchmarks/
    └── tests/
```

## 🚀 What This Achieves

1. **Clean First Impression**: Users see a focused, professional language repo
2. **No Confusion**: No overwhelming documentation or dev files
3. **Easy Contribution**: Clear separation of user vs dev concerns
4. **Professional Image**: Looks like Python, Node.js, Go - not a research project
5. **Simple Onboarding**: `README.md` → `install.syn` → `examples/` → Done!

## 📋 Next Steps

1. Move development files to `_internal_dev/`
2. Create clean user-focused `README.md`
3. Ensure `install.syn` works perfectly
4. Test the user experience from scratch
5. Create release packaging scripts
