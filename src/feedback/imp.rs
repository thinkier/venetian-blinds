use std::error::Error;
use std::f32::consts::PI;
use std::mem;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock as SyncRwLock;
use rppal::gpio::{Gpio, InputPin, Level, Trigger};
use crate::feedback::{Rotation, ServoPwmFeedback, ServoPwmFeedbackConfig};

const TAU: f32 = 2f32 * PI;
const LOWER_ERROR: f32 = 0.001;
const UPPER_ERROR: f32 = 0.02;

impl ServoPwmFeedback {
    pub fn init(pin: u8, freq: u16, lower_bound: f32, upper_bound: f32) -> Result<ServoPwmFeedback, Box<dyn Error>> {
        let mut in_pin: InputPin = Gpio::new()?.get(pin)?.into_input();

        Ok(ServoPwmFeedback {
            in_pin,
            rotation: Arc::new(SyncRwLock::new(Rotation::Stopped)),
            config: ServoPwmFeedbackConfig {
                freq,
                lower_bound,
                upper_bound,
            },
        })
    }

    pub fn enable_feedback(&mut self) {
        let rotation = Arc::clone(&self.rotation);
        let config = self.config.clone();
        let pwm_duration = Arc::new(SyncRwLock::new(Duration::from_secs(0)));
        let last_time = Arc::new(SyncRwLock::new(Instant::now()));

        self.in_pin.set_async_interrupt(Trigger::Both, move |level| {
            let now = Instant::now();
            match level {
                Level::High => {
                    *last_time.write() = now;
                }
                Level::Low => {
                    let (cur, last) = {
                        let last_time = { *last_time.read() };
                        let cur = now.duration_since(last_time);
                        let last = mem::replace(&mut *pwm_duration.write(), cur);

                        if let (Some(c), Some(l)) = (config.duration_to_radians(cur), config.duration_to_radians(last)) {
                            (c, l)
                        } else {
                            warn!("Invalid duration signal detected from servo");
                            return;
                        }
                    };

                    // Very close: stopped / stalled
                    if (cur - last).abs() < LOWER_ERROR * TAU {
                        *rotation.write() = Rotation::Stopped;
                    }

                    // Full cycle: transition from <tau to >0
                    if last > (1f32 - UPPER_ERROR) * TAU && cur < UPPER_ERROR * TAU || last < UPPER_ERROR * TAU && cur > (1f32 - UPPER_ERROR) * TAU {
                        info!("Full rotation {:?}", *rotation.read())
                    } else if cur > last {
                        *rotation.write() = Rotation::Clockwise;
                    } else if cur < last {
                        *rotation.write() = Rotation::Counterclockwise;
                    }
                }
            }
        }).expect("failed to attach interrupt timer");
    }

    pub fn disable_feedback(&mut self) {
        self.in_pin.clear_async_interrupt().unwrap();
    }
}

impl ServoPwmFeedbackConfig {
    pub fn duration_to_radians(&self, duration: Duration) -> Option<f32> {
        let mut period = 1. / self.freq as f32;
        let mut duration = duration.as_secs_f32();

        if duration < 0. || period < duration {
            return None;
        }

        if duration < period * self.lower_bound {
            duration = period * self.lower_bound;
        } else if duration > period * self.upper_bound {
            duration = period * self.upper_bound;
        }

        period = period * (self.upper_bound - self.lower_bound);
        let radians = (duration / period) * TAU;

        Some(radians)
    }
}
