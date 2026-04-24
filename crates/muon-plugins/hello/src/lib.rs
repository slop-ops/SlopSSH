use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PluginManifest {
    id: &'static str,
    name: &'static str,
    version: &'static str,
    description: &'static str,
    author: &'static str,
    capabilities: Vec<&'static str>,
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_manifest() -> *mut u8 {
    let manifest = PluginManifest {
        id: "hello-world",
        name: "Hello World",
        version: "0.1.0",
        description: "A simple example plugin that greets the user",
        author: "Muon SSH",
        capabilities: vec!["show_notification", "render_panel"],
    };
    let json = serde_json::to_string(&manifest).unwrap_or_default();
    let mut bytes = json.into_bytes();
    bytes.push(0);
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn on_session_connect(_session_id: *const u8, _session_id_len: usize) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn on_session_disconnect(_session_id: *const u8, _session_id_len: usize) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn render_panel() -> *mut u8 {
    let panel = serde_json::json!({
        "title": "Hello World",
        "content_type": "markdown",
        "content": "# Hello from Muon Plugin!\n\nThis is a sample plugin panel."
    });
    let json = panel.to_string();
    let mut bytes = json.into_bytes();
    bytes.push(0);
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    ptr
}
