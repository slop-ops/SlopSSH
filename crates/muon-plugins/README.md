# Muon SSH Example Plugins

This directory contains example WASM plugins for Muon SSH.

## Plugins

### hello-world
A simple example plugin that demonstrates the plugin API. Shows a markdown panel in the UI.

### k8s-context
Displays the current Kubernetes context in the status bar. Runs `kubectl config current-context` on the connected remote host.

## Building

```bash
# Build all plugins (from project root)
./scripts/build-plugins.sh

# Build individual plugin
cd crates/muon-plugins/hello
cargo build --release --target wasm32-unknown-unknown
```

## Installing

Place the compiled `.wasm` files in `~/.config/muon-ssh/plugins/`:

```bash
mkdir -p ~/.config/muon-ssh/plugins/
cp target/wasm32-unknown-unknown/release/hello_world.wasm ~/.config/muon-ssh/plugins/
```

Then restart Muon SSH and enable the plugin in Settings > Plugins.

## Plugin API

Each plugin exports these C ABI functions:

- `plugin_manifest()` - Returns JSON with plugin metadata
- `on_session_connect(session_id, len)` - Called when SSH session connects
- `on_session_disconnect(session_id, len)` - Called when SSH session disconnects
- `render_panel()` - Returns JSON with panel content (if `render_panel` capability)

### Manifest Format

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "0.1.0",
  "description": "Plugin description",
  "author": "Author Name",
  "capabilities": ["execute_command", "show_notification", "render_panel"]
}
```
