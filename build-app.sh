#!/bin/bash
# Build script for Artist Oil Paints macOS app

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_DIR"

echo "Building Artist Oil Paints..."
echo "Project directory: $PROJECT_DIR"

# Build release binary
echo ""
echo "Step 1: Building release binary..."
cargo leptos build --release

# Check build succeeded
if [ ! -f "target/release/aop" ]; then
    echo "Error: Build failed - binary not found"
    exit 1
fi

# App bundle paths
APP_DIR="ArtistOilPaints.app/Contents"
RESOURCES="$APP_DIR/Resources"
MACOS="$APP_DIR/MacOS"

echo ""
echo "Step 2: Creating app bundle..."

# Ensure directories exist
mkdir -p "$RESOURCES"
mkdir -p "$MACOS"

# Copy binary
echo "  - Copying binary..."
cp target/release/aop "$RESOURCES/"
chmod +x "$RESOURCES/aop"

# Copy site assets
echo "  - Copying site assets..."
rm -rf "$RESOURCES/site"
cp -r target/site "$RESOURCES/"

# Copy database
echo "  - Copying database..."
cp data.db "$RESOURCES/"

# Copy .env if exists
if [ -f ".env" ]; then
    echo "  - Copying .env..."
    cp .env "$RESOURCES/"
fi

# Copy launcher script (should already exist from initial setup)
if [ ! -f "$MACOS/launch" ]; then
    echo "Warning: Launcher script missing. Please recreate ArtistOilPaints.app structure."
fi

# Copy Info.plist (should already exist)
if [ ! -f "$APP_DIR/Info.plist" ]; then
    echo "Warning: Info.plist missing. Please recreate ArtistOilPaints.app structure."
fi

echo ""
echo "Build complete!"
echo ""
echo "App bundle: $PROJECT_DIR/ArtistOilPaints.app"
echo ""
echo "Contents:"
ls -la "$RESOURCES/"
echo ""
echo "To use:"
echo "  1. Double-click ArtistOilPaints.app to launch"
echo "  2. Or copy to Applications: cp -r ArtistOilPaints.app /Applications/"
echo "  3. The app will start the server and open http://127.0.0.1:3000"
echo ""
echo "To add a custom icon:"
echo "  sips -s format icns your-icon.png --out $RESOURCES/AppIcon.icns"
