pub struct ScriptLoader;

impl ScriptLoader {
    pub fn ps_script() -> &'static str {
        include_str!("../../../../scripts/ps.sh")
    }

    pub fn search_script() -> &'static str {
        include_str!("../../../../scripts/search.sh")
    }

    pub fn sysinfo_script() -> &'static str {
        include_str!("../../../../scripts/linux-sysinfo.sh")
    }

    pub fn load_all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("ps", Self::ps_script()),
            ("search", Self::search_script()),
            ("sysinfo", Self::sysinfo_script()),
        ]
    }
}
