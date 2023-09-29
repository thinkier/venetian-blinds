use std::error::Error;
use std::mem;
use std::sync::Arc;
use std::time::{Duration};
use crate::actuation::config::{ControllersConfig, VenetianBlind};
use crate::actuation::controller::{Controller, Controllers, InnerController};
use hap::Result as HapResult;
use tokio::sync::{RwLock, Semaphore};
#[cfg(feature = "raspi_pwm")]
use rppal::pwm::{Channel, Polarity, Pwm};

impl Controllers {
    pub fn from_config(config: ControllersConfig) -> HapResult<Controllers> {
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
    pub fn init(config: VenetianBlind) -> Result<Controller, Box<dyn Error>> {
        let channel = match config.channel {
            0 => Channel::Pwm0,
            1 => Channel::Pwm1,
            _ => panic!("Invalid channel"),
        };

        info!("Initializing PWM on channel {}", config.channel);
        let pwm = Pwm::with_period(
            channel,
            Duration::from_millis(20),
            Duration::from_micros(1500),
            Polarity::Normal,
            false,
        ).unwrap();

        Ok(Controller {
            activity: Arc::new(Semaphore::new(1)),
            inner: Arc::new(RwLock::new(InnerController {
                pwm,
                tilt: -90.0,
                position: 0.0,
                config,
            })),
        })
    }

    #[cfg(not(feature = "raspi_pwm"))]
    pub fn init(config: VenetianBlind) -> Result<Controller, Box<dyn Error>> {
        warn!("Running in debug mode, not initializing hardware.");
        Ok(Controller {
            activity: Arc::new(Semaphore::new(1)),
            inner: Arc::new(RwLock::new(InnerController {
                tilt: -90f32,
                position: 0f32,
                config,
            })),
        })
    }


    pub async fn get_tilt(&self) -> i8 {
        let (range, value) = {
            let inner = self.inner.read().await;

            (inner.config.rotations_to_fully_tilt, inner.tilt)
        };

        return map_f32_to_i8(value, 0f32, range, -90, 90);
    }

    pub async fn set_tilt(&self, tilt: i8) {
        let _ = self.activity.acquire().await.unwrap();

        self.set_tilt_nonacquiring(tilt).await;
    }

    async fn set_tilt_nonacquiring(&self, tilt: i8) {
        let (want, have) = {
            let mut inner = self.inner.write().await;

            let want = map_i8_to_f32(tilt, -90, 90, 0f32, inner.config.rotations_to_fully_tilt);
            let have = mem::replace(&mut inner.tilt, want);
            (want, have)
        };

        self.move_exact(want - have).await;
    }

    pub async fn get_position(&self) -> u8 {
        let inner = self.inner.read().await;
        map_f32_to_u8(inner.position, 0f32, inner.config.rotations_to_fully_extend, 0, 100)
    }

    pub async fn set_position(&self, pos: u8) {
        let _ = self.activity.acquire().await.unwrap();

        let orig_tilt = self.get_tilt().await;
        if orig_tilt > -90 {
            self.set_tilt(-90).await;
        }

        let (want, have) = {
            let mut inner = self.inner.write().await;

            let want = map_u8_to_f32(pos, 0, 100, 0f32, inner.config.rotations_to_fully_extend);
            let have = mem::replace(&mut inner.position, want);
            (want, have)
        };

        self.move_exact(want - have).await;
        self.set_tilt_nonacquiring(orig_tilt).await;
    }

    pub async fn move_exact(&self, amount: f32) {
        info!("Attempting to move exactly {} turns.", amount);

        #[cfg(feature = "raspi_pwm")]
        self.start_moving(amount.is_sign_positive()).await;

        // WONTFIX: Substitute for accurate rotation measurement from the PWM feedback
        tokio::time::sleep(Duration::from_secs_f32(amount.abs())).await;

        #[cfg(feature = "raspi_pwm")]
        self.stop_moving().await;

        info!("Move finished");
    }

    #[cfg(feature = "raspi_pwm")]
    pub async fn start_moving(&self, forward: bool) {
        let pwm = &self.inner.read().await.pwm;
        pwm.set_pulse_width(Duration::from_micros(
            if forward {
                1700
            } else {
                1300
            }
        )).unwrap();
        pwm.enable().unwrap()
    }

    #[cfg(feature = "raspi_pwm")]
    pub async fn stop_moving(&self) {
        self.inner.read().await.pwm.disable().unwrap();
    }
}

fn map_f32_to_u8(mut val: f32, ilow: f32, ihigh: f32, olow: u8, ohigh: u8) -> u8 {
    let olow = olow as f32;
    let ohigh = ohigh as f32;

    if val <= ilow { val = ilow; }
    if val >= ihigh { val = ihigh; }

    ((val - ilow) * (ohigh - olow) / (ihigh - ilow) + olow) as u8
}

fn map_f32_to_i8(mut val: f32, ilow: f32, ihigh: f32, olow: i8, ohigh: i8) -> i8 {
    let olow = olow as f32;
    let ohigh = ohigh as f32;

    if val <= ilow { val = ilow; }
    if val >= ihigh { val = ihigh; }

    ((val - ilow) * (ohigh - olow) / (ihigh - ilow) + olow) as i8
}

fn map_i8_to_f32(val: i8, ilow: i8, ihigh: i8, olow: f32, ohigh: f32) -> f32 {
    let mut val = val as f32;
    let ilow = ilow as f32;
    let ihigh = ihigh as f32;

    if val <= ilow { val = ilow; }
    if val >= ihigh { val = ihigh; }

    (val - ilow) * (ohigh - olow) / (ihigh - ilow) + olow
}

fn map_u8_to_f32(val: u8, ilow: u8, ihigh: u8, olow: f32, ohigh: f32) -> f32 {
    let mut val = val as f32;
    let ilow = ilow as f32;
    let ihigh = ihigh as f32;

    if val <= ilow { val = ilow; }
    if val >= ihigh { val = ihigh; }

    (val - ilow) * (ohigh - olow) / (ihigh - ilow) + olow
}
