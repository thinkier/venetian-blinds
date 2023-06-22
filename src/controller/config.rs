use hap::Result;
use std::collections::HashMap;
use tokio::fs;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControllerConfig {
    pub serial_port: String,
    #[serde(default = "default_serial_baud_rate")]
    pub serial_baud_rate: u32,
    pub blinds: HashMap<String, VenetianBlind>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VenetianBlind {
    pub address: u8,
    pub steps_to_extend: u32,
    #[serde(default = "default_sgthrs")]
    pub extend_sgthrs: u8,
    pub steps_to_tilt: u32,
    pub tilt_sgthrs: Option<u8>,
}

fn default_serial_baud_rate() -> u32 {
    115200
}

fn default_sgthrs() -> u8 {
    100
}

impl ControllerConfig {
    pub async fn load() -> Result<ControllerConfig> {
        let path = "Accessory.toml";

        return match fs::try_exists(path).await {
            Ok(true) => {
                let toml = fs::read_to_string(path).await?;

                Ok(toml::from_str(&toml).unwrap())
            }
            _ => {
                let config = ControllerConfig {
                    serial_port: "/dev/ttyUSB0".to_string(),
                    serial_baud_rate: default_serial_baud_rate(),
                    blinds: HashMap::new(),
                };

                let toml = toml::to_string(&config).unwrap();

                fs::write(path, toml).await?;

                Ok(config)
            }
        };
    }
}
