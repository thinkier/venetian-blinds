use std::sync::Arc;

use hap::Result;
use serial2::SerialPort;
use tokio::sync::Mutex;
use crate::controller::config::ControllerConfig;

pub mod config;

pub struct Controller {
    pub port: Arc<Mutex<SerialPort>>,
    pub config: ControllerConfig,
}

impl Controller {
    pub fn from_config(config: ControllerConfig) -> Result<Controller> {
        Ok(Controller {
            port: Arc::new(Mutex::new(SerialPort::open(&config.serial_port, config.serial_baud_rate)?)),
            config,
        })
    }

    pub fn get_instance(&self, name: &str) -> ControllerInstance {
        ControllerInstance {
            id: self.config.blinds[name].address,
            controller: self,
        }
    }
}

impl Clone for Controller {
    fn clone(&self) -> Self {
        Controller {
            port: Arc::clone(&self.port),
            config: self.config.clone(),
        }
    }
}

pub struct ControllerInstance<'a> {
    pub id: u8,
    pub controller: &'a Controller,
}
