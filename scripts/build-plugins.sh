#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLUGINS_DIR="$SCRIPT_DIR/../crates/slopssh-plugins"
OUTPUT_DIR="$HOME/.config/slopssh/plugins"

mkdir -p "$OUTPUT_DIR"

for plugin in "$PLUGINS_DIR"/*/; do
    name=$(basename "$plugin")
    echo "Building plugin: $name"
    (cd "$plugin" && cargo build --release --target wasm32-unknown-unknown 2>/dev/null) || \
    (cd "$plugin" && cargo build --release 2>/dev/null && \
     cp "target/release/lib${name//-/_}.so" "$OUTPUT_DIR/${name}.wasm" 2>/dev/null) || \
    echo "  Skipped $name (build failed or target not supported)"
done

echo "Plugins built to: $OUTPUT_DIR"
