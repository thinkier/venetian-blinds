use crate::model::conf::{BridgeConf};

#[cfg(test)]
mod tests;

impl BridgeConf {
    /// Attempt to read the configuration of the bridge from the file system. The configuration file is expected to be in the TOML format.
    ///
    /// When the environment variable `BRIDGE_CONF` is set, the configuration will attempt to read from the path specified by the envar.
    /// Otherwise, if the environment variable is not set, the configuration will attempt to read from the file `Bridge.toml`, or `DebugConf.toml` in a unit testing environment.
    pub fn read() -> Self {
        let conf_path = std::env::var("BRIDGE_CONF")
            .unwrap_or_else(|_| {
                #[cfg(test)]
                { "DebugConf.toml" }
                #[cfg(not(test))]
                { "Bridge.toml" }
            }.to_string());
        let conf_str = std::fs::read_to_string(conf_path).unwrap();

        toml::from_str(&conf_str).unwrap()
    }
}
