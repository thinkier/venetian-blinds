use std::sync::Arc;
use parking_lot::RwLock;
#[cfg(feature = "raspi")]
use rppal::pwm::Pwm;
use tokio::sync::Mutex;
use crate::model::config::{ControllersConfig, VenetianBlind};

#[derive(Debug)]
pub struct Controllers {
    pub config: ControllersConfig,
}

#[derive(Clone, Debug)]
pub struct Controller {
    pub(crate) inner: Arc<RwLock<InnerController>>,
    pub(crate) active: Arc<Mutex<()>>,
}

#[derive(Debug)]
pub struct InnerController {
    #[cfg(feature = "raspi")]
    pub(crate) pwm: Pwm,
    pub(crate) tilt: i8,
    pub(crate) position: u8,
    pub config: VenetianBlind,
}

