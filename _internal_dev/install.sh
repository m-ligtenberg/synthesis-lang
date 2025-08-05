#!/bin/bash
# Synthesis Language Installation Script
# Installs Synthesis as a standalone creative programming language

set -e

SYNTHESIS_VERSION="0.1.0"
INSTALL_DIR="$HOME/.synthesis"
BIN_DIR="$HOME/.local/bin"

echo "🎵 Installing Synthesis Language v$SYNTHESIS_VERSION"
echo "   Universal Creative Programming Language"
echo

# Create directories
mkdir -p "$INSTALL_DIR"
mkdir -p "$BIN_DIR"

# Check if we're installing from source or downloading
if [[ -f "syn-pkg" && -f "Cargo.toml" ]]; then
    echo "📦 Installing from source..."
    
    # Check for Rust (required for building)
    if ! command -v cargo &> /dev/null; then
        echo "⚠️  Rust is required to build Synthesis from source."
        echo "   Visit https://rustup.rs to install Rust, then run this script again."
        exit 1
    fi
    
    # Build release binaries
    echo "🔨 Building Synthesis binaries..."
    cargo build --release --quiet
    
    # Copy binaries
    cp target/release/synthesis "$INSTALL_DIR/"
    cp target/release/syn-pkg "$INSTALL_DIR/"
    
    # Copy wrapper script
    cp syn-pkg "$BIN_DIR/syn-pkg"
    chmod +x "$BIN_DIR/syn-pkg"
    
    # Update wrapper to point to installed location
    sed -i "s|SYNTHESIS_ROOT=.*|SYNTHESIS_ROOT=\"$INSTALL_DIR\"|" "$BIN_DIR/syn-pkg"
    sed -i "s|target/release/syn-pkg|syn-pkg|" "$BIN_DIR/syn-pkg"
    
else
    echo "📥 Downloading Synthesis binaries..."
    echo "   (Download functionality will be implemented when binaries are available)"
    echo "   For now, please clone the repository and run this script from the source directory."
    exit 1
fi

# Create examples directory
echo "📚 Setting up examples and documentation..."
mkdir -p "$INSTALL_DIR/examples"
if [[ -d "examples" ]]; then
    cp -r examples/* "$INSTALL_DIR/examples/"
fi

# Create standard library
mkdir -p "$INSTALL_DIR/stdlib"
if [[ -d "src/modules" ]]; then
    cp -r src/modules "$INSTALL_DIR/stdlib/"
fi

# Add to PATH if not already there
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "🔧 Adding Synthesis to PATH..."
    
    # Determine shell config file
    if [[ -n "$ZSH_VERSION" ]]; then
        SHELL_CONFIG="$HOME/.zshrc"
    elif [[ -n "$BASH_VERSION" ]]; then
        SHELL_CONFIG="$HOME/.bashrc"
    else
        SHELL_CONFIG="$HOME/.profile"
    fi
    
    # Add to shell config
    echo "" >> "$SHELL_CONFIG"
    echo "# Synthesis Language" >> "$SHELL_CONFIG"
    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$SHELL_CONFIG"
    
    echo "   Added $BIN_DIR to PATH in $SHELL_CONFIG"
    echo "   Run 'source $SHELL_CONFIG' or restart your terminal"
fi

# Create global config
echo "⚙️  Creating global configuration..."
cat > "$INSTALL_DIR/config.toml" << EOF
[synthesis]
version = "$SYNTHESIS_VERSION"
stdlib_path = "$INSTALL_DIR/stdlib"
examples_path = "$INSTALL_DIR/examples"

[registry]
url = "https://packages.synthesis-lang.org"
cache_dir = "$INSTALL_DIR/cache"

[editor]
syntax_highlighting = true
auto_complete = true
theme = "synthesis-dark"
EOF

# Success message
echo
echo "✅ Synthesis Language installed successfully!"
echo
echo "📍 Installation location: $INSTALL_DIR"
echo "🔗 Binary location: $BIN_DIR/syn-pkg"
echo
echo "🚀 Quick start:"
echo "   syn-pkg new my-first-project"
echo "   cd my-first-project"
echo "   syn-pkg run"
echo
echo "📖 Learn more:"
echo "   syn-pkg --help          # Show all commands"
echo "   ls $INSTALL_DIR/examples # Browse examples"
echo "   https://synthesis-lang.org  # Official documentation"
echo
echo "🎨 Happy creative coding!"