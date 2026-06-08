#!/bin/bash

# 1. Build the project in release mode
echo "📦 Building Release Binary..."
cargo build --release

# 2. Create local bin directory if it doesn't exist
mkdir -p ~/.local/bin

# 3. Move the binary
echo "🚀 Installing to ~/.local/bin/..."
cp target/release/phantimer ~/.local/bin/

# 4. Create a Desktop Entry (for Rofi/Wofi/Launcher support)
# ...
echo "🖥️  Installing Desktop Entry..."
mkdir -p ~/.local/share/applications

# We have to edit the Exec line for local install, or rely on PATH
sed 's|Exec=phantimer|Exec='"$HOME"'/.local/bin/phantimer|' phantimer.desktop >~/.local/share/applications/phantimer.desktop

# 5. Success
echo "✅ Phantimer installed! You can now run 'phantimer' from anywhere."
