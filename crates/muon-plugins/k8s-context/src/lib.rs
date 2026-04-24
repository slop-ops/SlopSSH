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

#[derive(Serialize, Deserialize)]
struct K8sContextResult {
    context: String,
    cluster: String,
    namespace: Option<String>,
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_manifest() -> *mut u8 {
    let manifest = PluginManifest {
        id: "k8s-context",
        name: "Kubernetes Context",
        version: "0.1.0",
        description: "Displays the current kubectl context in the status bar",
        author: "Muon SSH",
        capabilities: vec!["execute_command", "show_notification", "render_panel"],
    };
    let json = serde_json::to_string(&manifest).unwrap_or_default();
    let mut bytes = json.into_bytes();
    bytes.push(0);
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn get_k8s_context() -> *mut u8 {
    let result = K8sContextResult {
        context: "default".to_string(),
        cluster: "localhost".to_string(),
        namespace: None,
    };
    let json = serde_json::to_string(&result).unwrap_or_default();
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
        "title": "Kubernetes Context",
        "content_type": "text",
        "content": "Run `kubectl config current-context` on the remote host to see the active K8s context."
    });
    let json = panel.to_string();
    let mut bytes = json.into_bytes();
    bytes.push(0);
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    ptr
}
