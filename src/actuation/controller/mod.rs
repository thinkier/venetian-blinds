use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock as SyncRwLock;
#[cfg(feature = "raspi_pwm")]
use rppal::gpio::InputPin;
#[cfg(feature = "raspi_pwm")]
use rppal::pwm::Pwm;
use tokio::sync::{RwLock, Semaphore};
use crate::actuation::config::{ControllersConfig, VenetianBlind};

pub mod imp;

#[derive(Debug)]
pub struct Controllers {
    pub config: ControllersConfig,
}

#[derive(Clone, Debug)]
pub struct Controller {
    activity: Arc<Semaphore>,
    inner: Arc<RwLock<InnerController>>,
}

#[derive(Debug)]
pub struct InnerController {
    #[cfg(feature = "raspi_pwm")]
    pwm: Pwm,
    #[cfg(feature = "raspi_pwm")]
    tilt: f32,
    position: f32,
    pub config: VenetianBlind,
}

