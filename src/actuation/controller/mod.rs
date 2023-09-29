use std::sync::Arc;
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
    pub activity: Arc<Semaphore>,
    inner: Arc<RwLock<InnerController>>,
}

#[derive(Debug)]
pub struct InnerController {
    #[cfg(feature = "raspi_pwm")]
    pwm: Pwm,
    // #[cfg(feature = "raspi_pwm")]
    // feedback: crate::feedback::ServoPwmFeedback,
    tilt: f32,
    position: f32,
    pub config: VenetianBlind,
}

