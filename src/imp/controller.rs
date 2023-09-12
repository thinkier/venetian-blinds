use std::mem;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;
use crate::model::config::{ControllersConfig, VenetianBlind};
use crate::model::controller::{Controller, Controllers, InnerController};
use hap::Result;
use tokio::sync::RwLock;
#[cfg(feature = "raspi_pwm")]
use rppal::gpio::{InputPin, Gpio, Trigger, Level};
#[cfg(feature = "raspi_pwm")]
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
    #[cfg(feature = "raspi_pwm")]
    pub fn init(config: VenetianBlind) -> Result<Controller> {
        let channel = match config.channel {
            0 => Channel::Pwm0,
            1 => Channel::Pwm1,
            _ => panic!("Invalid channel"),
        };

        info!("Initializing PWM on channel {}", config.channel);
        let pwm = Pwm::with_period(channel, Duration::from_millis(20), Duration::from_micros(1500), Polarity::Normal, true).unwrap();
        let mut in_pin: InputPin = Gpio::new()?.get(config.in_pin)?.into_input();

        let mut last_time = AtomicU64::new(0);
        in_pin.set_async_interrupt(Trigger::Both, move |level| {
            match level {
                Level::High => {
                    // TODO Timer rise
                }
                Level::Low => {
                    // TODO Timer fall
                }
            }
        }).expect("failed to attach interrupt timer");

        Ok(Controller {
            inner: Arc::new(RwLock::new(InnerController {
                pwm,
                in_pin,
                tilt: -90.0,
                position: 0.0,
                config,
            }))
        })
    }

    #[cfg(not(feature = "raspi_pwm"))]
    pub fn init(config: VenetianBlind) -> Result<Controller> {
        warn!("Running in debug mode, not initializing hardware.");
        Ok(Controller {
            inner: Arc::new(RwLock::new(InnerController {
                tilt: -90f32,
                position: 0f32,
                config,
            }))
        })
    }


    pub fn get_tilt(&self) -> i8 {
        let (range, value) = {
            let inner = self.inner.read();

            (inner.config.rotations_to_fully_tilt, inner.tilt)
        };

        return map_f32_to_i8(value, 0f32, range, -90, 90);
    }

    pub async fn set_tilt(&self, tilt: i8) {
        let mut inner = self.inner.write().await;

        let want = map_i8_to_f32(tilt, -90, 90, 0f32, inner.config.rotations_to_fully_tilt);
        let have = mem::replace(&mut inner.tilt, want);

        self.move_exact(want - have).await;
    }

    pub fn get_position(&self) -> u8 {
        map_f32_to_u8(self.inner.read().position, 0f32, self.inner.read().config.rotations_to_fully_extend, 0, 100)
    }

    pub async fn set_position(&self, pos: u8) {
        let orig_tilt = self.get_tilt();
        if orig_tilt > -90 {
            self.set_tilt(-90).await;
        }

        let mut inner = self.inner.write().await;

        let want = map_u8_to_f32(pos, 0, 100, 0f32, inner.config.rotations_to_fully_extend);
        let have = mem::replace(&mut inner.position, want);

        self.move_exact(want - have).await;
    }

    pub async fn move_exact(&self, amount: f32) {}

    pub fn start_moving(&self, forward: bool) {
        #[cfg(feature = "raspi_pwm")]
        self.inner.read().pwm.set_pulse_width(Duration::from_micros(
            if forward {
                1700
            } else {
                1300
            }
        )).unwrap();
    }

    pub fn stop_moving(&self) {
        #[cfg(feature = "raspi_pwm")]
        self.inner.read().pwm.set_pulse_width(Duration::from_micros(1500)).unwrap();
    }
}

fn map_f32_to_u8(mut val: f32, ilow: f32, ihigh: f32, olow: u8, ohigh: u8) -> u8 {
    if val <= ilow { val = ilow; }
    if val >= ihigh { val = ihigh; }

    ((val - ilow) * (ohigh - olow) / (ihigh - ilow) + olow)
}

fn map_f32_to_i8(mut val: f32, ilow: f32, ihigh: f32, olow: i8, ohigh: i8) -> i8 {
    if val <= ilow { val = ilow; }
    if val >= ihigh { val = ihigh; }

    ((val - ilow) * (ohigh - olow) / (ihigh - ilow) + olow)
}

fn map_i8_to_f32(mut val: i8, ilow: i8, ihigh: i8, olow: f32, ohigh: f32) -> f32 {
    if val <= ilow { val = ilow; }
    if val >= ihigh { val = ihigh; }

    ((val - ilow) as f32 * (ohigh - olow) / (ihigh - ilow) as f32 + olow)
}

fn map_u8_to_f32(mut val: u8, ilow: u8, ihigh: u8, olow: f32, ohigh: f32) -> f32 {
    if val <= ilow { val = ilow; }
    if val >= ihigh { val = ihigh; }

    ((val - ilow) as f32 * (ohigh - olow) / (ihigh - ilow) as f32 + olow)
}
