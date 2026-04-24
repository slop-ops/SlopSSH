use std::path::Path;

use anyhow::Result;
use wasmtime::*;

use super::api::PluginManifest;
use super::host::PluginHost;

pub struct WasmLoader {
    engine: Engine,
}

impl WasmLoader {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_multi_memory(true);
        config.consume_fuel(true);
        let engine = Engine::new(&config)?;
        Ok(Self { engine })
    }

    pub fn load_module(&self, wasm_path: &Path) -> Result<(Module, PluginManifest)> {
        let module = Module::from_file(&self.engine, wasm_path)?;

        let file_name = wasm_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let manifest = PluginManifest {
            id: file_name.to_string(),
            name: file_name.to_string(),
            version: "0.1.0".to_string(),
            description: None,
            author: None,
            capabilities: vec![super::api::PluginCapability::ExecuteCommand],
        };

        Ok((module, manifest))
    }

    pub fn create_store(&self, fuel: u64) -> Result<Store<PluginHost>> {
        let host = PluginHost::new(&[
            super::api::PluginCapability::ExecuteCommand,
            super::api::PluginCapability::ReadSetting,
        ]);
        let mut store = Store::new(&self.engine, host);
        store.set_fuel(fuel)?;
        Ok(store)
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

impl Default for WasmLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create WasmLoader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_loader_new() {
        let loader = WasmLoader::new();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_wasm_loader_default() {
        let _loader = WasmLoader::default();
    }

    #[test]
    fn test_wasm_loader_create_store() {
        let loader = WasmLoader::new().unwrap();
        let store = loader.create_store(1_000_000);
        assert!(store.is_ok());
    }

    #[test]
    fn test_load_module_nonexistent() {
        let loader = WasmLoader::new().unwrap();
        let result = loader.load_module(Path::new("/nonexistent/test.wasm"));
        assert!(result.is_err());
    }
}
