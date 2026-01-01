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
echo "🖥️  Creating Desktop Entry..."
mkdir -p ~/.local/share/applications

cat >~/.local/share/applications/phantimer.desktop <<EOF
[Desktop Entry]
Type=Application
Name=Phantimer
Comment=Floating Ghost Timer
Exec=$HOME/.local/bin/phantimer
Terminal=true
Categories=Utility;
Keywords=timer;countdown;
EOF

# 5. Success
echo "✅ Phantimer installed! You can now run 'phantimer' from anywhere."
