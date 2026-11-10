#!/bin/bash
set -euo pipefail

# Build Alfred Claude Workflow
# Creates the claude.alfredworkflow package; --install also opens it in Alfred.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build/workflow"
OUTPUT_FILE="$PROJECT_ROOT/claude.alfredworkflow"
BINARY="$PROJECT_ROOT/target/release/alfred-claude"

echo "Building Alfred Claude Workflow..."

cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml"

# Clean previous build
rm -rf "$BUILD_DIR"
rm -f "$OUTPUT_FILE"
mkdir -p "$BUILD_DIR"

echo "Copying workflow files..."

if [[ -f "$PROJECT_ROOT/info.plist" ]]; then
    cp "$PROJECT_ROOT/info.plist" "$BUILD_DIR/"
else
    echo "Error: info.plist not found" >&2
    exit 1
fi

if [[ -f "$PROJECT_ROOT/icons/icon.png" ]]; then
    cp "$PROJECT_ROOT/icons/icon.png" "$BUILD_DIR/icon.png"
else
    echo "Warning: icons/icon.png not found, workflow will have no icon" >&2
fi

cp "$BINARY" "$BUILD_DIR/alfred-claude"
chmod 755 "$BUILD_DIR/alfred-claude"

echo "Creating claude.alfredworkflow..."
cd "$BUILD_DIR"
zip -r "$OUTPUT_FILE" . -x "*.DS_Store" -x "__MACOSX/*"

echo "Successfully created: $OUTPUT_FILE"

if [[ "${1:-}" == "--install" ]]; then
    open "$OUTPUT_FILE"
fi
