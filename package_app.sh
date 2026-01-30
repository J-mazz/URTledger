#!/bin/bash
set -e

APP_NAME="URTledger"
BINARY_NAME="dynamic_inventory_engine"
VERSION="1.0.0"
DIST_DIR="dist/$APP_NAME"

echo "[*] Building Release Binary..."
cargo build --release

echo "[*] Creating Distribution Directory..."
rm -rf dist
mkdir -p "$DIST_DIR"

# 1. Copy the Binary
cp "target/release/$BINARY_NAME" "$DIST_DIR/"

# 2. Copy the Icon (if it exists, otherwise skip)
if [ -f "app_icon.png" ]; then
    cp "app_icon.png" "$DIST_DIR/"
fi

# 3. Create the Installation Script
cat << 'EOF' > "$DIST_DIR/install.sh"
#!/bin/bash

APP_NAME="URTledger"
BINARY_NAME="dynamic_inventory_engine"
INSTALL_DIR="$HOME/.local/share/$APP_NAME"
BIN_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"

echo "Installing $APP_NAME..."

# Create Directories
mkdir -p "$INSTALL_DIR"
mkdir -p "$BIN_DIR"
mkdir -p "$DESKTOP_DIR"

# Move Files
cp "$BINARY_NAME" "$INSTALL_DIR/"
[ -f "app_icon.png" ] && cp "app_icon.png" "$INSTALL_DIR/"

# Create a Wrapper Script (Ensures DB is saved in the correct folder)
# This is CRITICAL. Without this, the DB appears in random home folders.
cat << EOW > "$INSTALL_DIR/launch.sh"
#!/bin/bash
cd "$INSTALL_DIR"
./$BINARY_NAME
EOW

chmod +x "$INSTALL_DIR/launch.sh"

# Create Desktop Entry
ICON_PATH="utilities-terminal"
[ -f "$INSTALL_DIR/app_icon.png" ] && ICON_PATH="$INSTALL_DIR/app_icon.png"

cat << EOD > "$DESKTOP_DIR/$APP_NAME.desktop"
[Desktop Entry]
Version=1.0
Type=Application
Name=URT Ledger
Comment=Inventory Management System
Exec=$INSTALL_DIR/launch.sh
Icon=$ICON_PATH
Terminal=false
Categories=Office;Utility;
EOD

# Refresh Menu
update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true

echo "------------------------------------------"
echo "Installation Complete!"
echo "You can now find 'URT Ledger' in your Applications menu."
echo "------------------------------------------"
EOF

chmod +x "$DIST_DIR/install.sh"

# 4. Compress the package
echo "[*] Compressing Package..."
cd dist
tar -czf "${APP_NAME}_v${VERSION}_Installer.tar.gz" "$APP_NAME"

echo "[+] Package created at: dist/${APP_NAME}_v${VERSION}_Installer.tar.gz"