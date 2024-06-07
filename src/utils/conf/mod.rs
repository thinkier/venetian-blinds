use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use crate::model::conf::{BridgeConf, MotorConf};
#[cfg(any(feature = "hw_ble"))]
use crate::model::conf::HwMode;
use crate::model::sequencer::WindowDressingSequencer;

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

    pub(crate) fn read_with_name(conf_path: &str) -> Self {
        let conf_str = std::fs::read_to_string(conf_path).unwrap();

        #[cfg(any(feature = "hw_ble"))]
        {
            let mut parsed: Self = toml::from_str(&conf_str).unwrap();

            for blind in &mut parsed.blinds {
                #[cfg(feature = "hw_ble")]
                if let HwMode::Ble { name } = &mut blind.backend {
                    if name.is_empty() {
                        name.push_str(&blind.name);
                    }
                }
            }

            parsed
        }

        #[cfg(not(any(feature = "hw_ble")))]
        {
            toml::from_str(&conf_str).unwrap()
        }
    }
}

impl Default for MotorConf {
    fn default() -> Self {
        MotorConf::Servo {
            pulse_width_delta: 400,
            pulse_width_center: 1500,
            full_cycle_time: 100f32,
            full_tilt_time: None,
        }
    }
}

impl MotorConf {
    pub async fn to_sequencer(&self, name: &str) -> Arc<Mutex<WindowDressingSequencer>> {
        let name = name_to_state_config(name);
        let mut seq = WindowDressingSequencer::from_conf_and_name(*self, name.clone());

        let file = fs::OpenOptions::new()
            .write(false)
            .create(false)
            .read(true)
            .open(&name)
            .await;

        match file {
            Ok(file) => {
                if let Err(e) = seq.load(file).await {
                    warn!("Failed to load state for {}: {:?}", name, e);
                }
            }
            Err(e) => {
                warn!("Failed to open state file for {}: {:?}", name, e);
            }
        }

        Arc::new(Mutex::new(seq))
    }
}

fn name_to_state_config(name: &str) -> String {
    format!("data/{}.toml", safe_string(name))
}

fn safe_string(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() {
            c
        } else {
            '_'
        })
        .collect()
}
