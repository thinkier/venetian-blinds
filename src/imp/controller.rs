use std::sync::Arc;
use std::time::Duration;
use crate::model::config::{ControllersConfig, VenetianBlind};
use crate::model::controller::{Controller, Controllers, InnerController};
use hap::Result;
use parking_lot::RwLock;
#[cfg(feature = "raspi")]
use rppal::pwm::{Channel, Polarity, Pwm};

impl Controllers {
    pub fn from_config(config: ControllersConfig) -> Result<Controllers> {
        Ok(Controllers {
            config,
        })
    }

    pub fn get_instance(&self, name: &String) -> Controller {
        Controller::init(self.config.blinds.get(name).unwrap().clone()).unwrap()
    }
}

impl Controller {
    #[cfg(feature = "raspi")]
    pub fn init(config: VenetianBlind) -> Result<Controller> {
        let channel = match config.channel {
            0 => Channel::Pwm0,
            1 => Channel::Pwm1,
            _ => panic!("Invalid channel"),
        };

        info!("Initializing PWM on channel {}", config.channel);
        let pwm = Pwm::with_period(channel, Duration::from_millis(20), Duration::from_micros(1500), Polarity::Normal, true).unwrap();

        Ok(Controller {
            inner: Arc::new(RwLock::new(InnerController {
                pwm,
                tilt: -128,
                position: 255,
                config,
            })),
            active: Default::default(),
        })
    }

    #[cfg(not(feature = "raspi"))]
    pub fn init(config: VenetianBlind) -> Result<Controller> {
        warn!("Running in debug mode, not initializing hardware.");
        Ok(Controller {
            inner: Arc::new(RwLock::new(InnerController {
                tilt: -128,
                position: 255,
                config,
            })),
            active: Default::default(),
        })
    }


    pub fn get_tilt(&self) -> i8 {
        self.inner.read().tilt
    }

    pub async fn set_tilt(&self, tilt: i8) {
        let _ = self.active.lock().await;
        let mut inn = Arc::clone(&self.inner);
        let delta = tilt - self.get_tilt();

        if delta == 0 { return; }
    }

    pub fn get_position(&self) -> u8 {
        self.inner.read().position
    }

    pub async fn set_position(&self, pos: u8) {
        if self.get_tilt() != -128 {
            self.set_tilt(-128).await;
        }
        let _ = self.active.lock().await;

        let mut inn = Arc::clone(&self.inner);
        let delta = pos as i16 - self.get_position() as i16;

        if delta == 0 { return; }
    }

    pub fn start_moving(&self, forward: bool) {
        #[cfg(feature = "raspi")]
        self.inner.read().pwm.set_pulse_width(Duration::from_micros(
            if forward {
                2000
            } else {
                1000
            }
        )).unwrap();
    }

    pub fn stop_moving(&self) {
        #[cfg(feature = "raspi")]
        self.inner.read().pwm.set_pulse_width(Duration::from_micros(1500)).unwrap();
    }
}
