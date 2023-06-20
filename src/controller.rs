use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};
use tokio::fs;

use hap::Result;

#[derive(Clone)]
pub struct Controller {
    pub config: ControllerConfig,
}

impl Controller {
    pub fn from_config(config: ControllerConfig) -> Controller {
        Controller {
            config,
        }
    }

    pub fn get_instance(&self, id: u8) -> ControllerInstance {
        ControllerInstance {
            id,
            controller: self,
        }
    }
}

pub struct ControllerInstance<'a> {
    pub id: u8,
    pub controller: &'a Controller,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControllerConfig {
    pub serial_port: String,
    #[serde(default = "default_serial_baud_rate")]
    pub serial_baud_rate: u32,
    pub blinds: Blinds,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Blinds {
    #[serde(flatten)]
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
                    blinds: Blinds {
                        uart_tx: 0,
                        uart_rx: 1,
                        blinds: HashMap::new(),
                    },
                };

                let toml = toml::to_string(&config).unwrap();

                fs::write(path, toml).await?;

                Ok(config)
            }
        };
    }
}