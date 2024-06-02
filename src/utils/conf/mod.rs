use crate::model::conf::{BridgeConf, HwMode};

#[cfg(test)]
mod tests;

impl BridgeConf {
    /// Attempt to read the configuration of the bridge from the file system. The configuration file is expected to be in the TOML format.
    ///
    /// When the environment variable `BRIDGE_CONF` is set, the configuration will attempt to read from the path specified by the envar.
    /// Otherwise, if the environment variable is not set, the configuration will attempt to read from the file `Bridge.toml`, or `DebugConf.toml` in a unit testing environment.
    pub fn read() -> Self {
        let conf_path = std::env::var("BRIDGE_CONF")
            .unwrap_or_else(|_| "Bridge.toml".to_string());

        Self::read_with_name(&conf_path)
    }

    pub(crate) fn read_with_name(conf_path:&str) -> Self {
        let conf_str = std::fs::read_to_string(conf_path).unwrap();

        let mut parsed: Self = toml::from_str(&conf_str).unwrap();

        for blind in &mut parsed.blinds {
            #[cfg(feature = "hw_ble")]
            if let HwMode::Ble { name } = &mut blind.hw_mode {
                if name.is_empty() {
                    name.push_str(&blind.name);
                }
            }
        }

        parsed
    }
}
