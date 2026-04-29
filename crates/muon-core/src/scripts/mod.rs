//! Built-in shell scripts bundled at compile time.

pub struct ScriptLoader;

impl ScriptLoader {
    /// Returns the process-listing script content.
    pub fn ps_script() -> &'static str {
        include_str!("../../../../scripts/ps.sh")
    }

    /// Returns the file search script content.
    pub fn search_script() -> &'static str {
        include_str!("../../../../scripts/search.sh")
    }

    /// Returns the system information script content.
    pub fn sysinfo_script() -> &'static str {
        include_str!("../../../../scripts/linux-sysinfo.sh")
    }

    /// Returns all built-in scripts as `(name, content)` pairs.
    pub fn load_all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("ps", Self::ps_script()),
            ("search", Self::search_script()),
            ("sysinfo", Self::sysinfo_script()),
        ]
    }
}
