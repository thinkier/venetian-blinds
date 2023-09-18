use std::error::Error;
use parking_lot::RwLock as SyncRwLock;
use std::f32::consts::PI;
use std::mem;
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::model::config::{ControllersConfig, VenetianBlind};
use crate::model::controller::{Controller, Controllers, InnerController};
use hap::Result as HapResult;
use tokio::sync::{RwLock, Semaphore};
#[cfg(feature = "raspi_pwm")]
use rppal::gpio::{InputPin, Gpio, Trigger, Level};
#[cfg(feature = "raspi_pwm")]
use rppal::pwm::{Channel, Polarity, Pwm};
use tokio::task::JoinError;

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
        let pwm = Pwm::with_period(channel, Duration::from_millis(20), Duration::from_micros(1500), Polarity::Normal, true).unwrap();
        let mut in_pin: InputPin = Gpio::new()?.get(config.feedback_pin)?.into_input();

        let pwm_duration = Arc::new(SyncRwLock::new(Duration::from_millis(0)));

        Ok(Controller {
            activity: Arc::new(Semaphore::new(1)),
            inner: Arc::new(RwLock::new(InnerController {
                pwm,
                in_pin,
                tilt: -90.0,
                position: 0.0,
                config,
            })),
            pwm_duration,
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
            pwm_duration: Arc::new(SyncRwLock::new(Duration::from_millis(0))),
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
        let mut inner = self.inner.write().await;

        let want = map_i8_to_f32(tilt, -90, 90, 0f32, inner.config.rotations_to_fully_tilt);
        let have = mem::replace(&mut inner.tilt, want);

        self.move_exact(want - have).await;
    }

    pub async fn get_position(&self) -> u8 {
        let inner = self.inner.read().await;
        map_f32_to_u8(inner.position, 0f32, inner.config.rotations_to_fully_extend, 0, 100)
    }

    pub async fn set_position(&self, pos: u8) {
        let orig_tilt = self.get_tilt().await;
        if orig_tilt > -90 {
            self.set_tilt(-90).await;
        }

        let mut inner = self.inner.write().await;

        let want = map_u8_to_f32(pos, 0, 100, 0f32, inner.config.rotations_to_fully_extend);
        let have = mem::replace(&mut inner.position, want);

        self.move_exact(want - have).await;
    }

    pub async fn move_exact(&self, amount: f32) {}

    pub async fn start_moving(&self, forward: bool) {
        #[cfg(feature = "raspi_pwm")]
        self.inner.read().await.pwm.set_pulse_width(Duration::from_micros(
            if forward {
                1700
            } else {
                1300
            }
        )).unwrap();
    }

    pub async fn stop_moving(&self) {
        #[cfg(feature = "raspi_pwm")]
        self.inner.read().await.pwm.set_pulse_width(Duration::from_micros(1500)).unwrap();
    }

    /// Radial position in terms of radians
    pub async fn get_radial_position(&self) -> Result<f32, JoinError> {
        let (freq, min, max) = {
            let config = &self.inner.read().await.config;
            (config.feedback_freq, config.feedback_duty_cycle_lower_bound, config.feedback_duty_cycle_upper_bound)
        };

        let pwm_duration = Arc::clone(&self.pwm_duration);

        tokio::task::spawn_blocking(move || {
            let mut phase_d = Duration::from_secs(1).div_f32(freq as f32).as_nanos() as f32;
            let mut d = pwm_duration.read().as_nanos() as f32;
            d -= min * phase_d;
            d /= max - min;
            phase_d *= max - min;
            return PI * 2.0 * d / phase_d;
        }).await
    }

    pub async fn enable_feedback(&self) {
        let pwm_duration = Arc::clone(&self.pwm_duration);
        let last_time = Arc::new(SyncRwLock::new(Instant::now()));
        #[cfg(feature = "raspi_pwm")]
        self.inner.write().await.in_pin.set_async_interrupt(Trigger::Both, move |level| {
            let now = Instant::now();
            match level {
                Level::High => {
                    *last_time.write() = now;
                }
                Level::Low => {
                    let last_time = { *last_time.read() };
                    let cur = now.duration_since(last_time);
                    let last = mem::replace(&mut *pwm_duration.write(), cur);
                }
            }
        }).expect("failed to attach interrupt timer");
    }

    pub async fn disable_feedback(&self) {
        #[cfg(feature = "raspi_pwm")]
        self.inner.write().await.in_pin.clear_async_interrupt().unwrap();
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

fn map_u8_to_f32(mut val: u8, ilow: u8, ihigh: u8, olow: f32, ohigh: f32) -> f32 {
    let mut val = val as f32;
    let ilow = ilow as f32;
    let ihigh = ihigh as f32;

    if val <= ilow { val = ilow; }
    if val >= ihigh { val = ihigh; }

    (val - ilow) * (ohigh - olow) / (ihigh - ilow) + olow
}
