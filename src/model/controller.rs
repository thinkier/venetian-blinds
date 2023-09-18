use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock as SyncRwLock;
#[cfg(feature = "raspi_pwm")]
use rppal::gpio::InputPin;
#[cfg(feature = "raspi_pwm")]
use rppal::pwm::Pwm;
use tokio::sync::{RwLock, Semaphore};
use crate::model::config::{ControllersConfig, VenetianBlind};

#[derive(Debug)]
pub struct Controllers {
    pub config: ControllersConfig,
}

#[derive(Clone, Debug)]
pub struct Controller {
    pub(crate) activity: Arc<Semaphore>,
    pub(crate) inner: Arc<RwLock<InnerController>>,
    pub(crate) pwm_duration: Arc<SyncRwLock<Duration>>,
}

#[derive(Debug)]
pub struct InnerController {
    #[cfg(feature = "raspi_pwm")]
    pub(crate) pwm: Pwm,
    #[cfg(feature = "raspi_pwm")]
    pub(crate) in_pin: InputPin,
    pub(crate) tilt: f32,
    pub(crate) position: f32,
    pub config: VenetianBlind,
}

