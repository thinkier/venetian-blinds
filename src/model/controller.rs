use std::sync::Arc;
#[cfg(feature = "raspi_pwm")]
use rppal::gpio::InputPin;
#[cfg(feature = "raspi_pwm")]
use rppal::pwm::Pwm;
use tokio::sync::RwLock;
use crate::model::config::{ControllersConfig, VenetianBlind};

#[derive(Debug)]
pub struct Controllers {
    pub config: ControllersConfig,
}

#[derive(Clone, Debug)]
pub struct Controller {
    pub(crate) inner: Arc<RwLock<InnerController>>,
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

