use std::f32::consts::PI;
use std::mem;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock as SyncRwLock;
use rppal::gpio::{Gpio, InputPin, Level, Trigger};
use tokio::sync::RwLock;
use crate::feedback::{Rotation, ServoPwmFeedback, ServoPwmFeedbackConfig};

impl ServoPwmFeedback {
    pub fn init(pin: u8, freq: u16, lower_bound: f32, upper_bound: f32) -> ServoPwmFeedback {
        let mut in_pin: InputPin = Gpio::new()?.get(pin)?.into_input();

        ServoPwmFeedback {
            in_pin,
            rotation: Arc::new(RwLock::new(Rotation::Stopped)),
            config: ServoPwmFeedbackConfig {
                freq,
                lower_bound,
                upper_bound,
            },
        }
    }

    pub async fn enable_feedback(&mut self) {
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

                        (config.duration_to_radians(cur), config.duration_to_radians(last))
                    };

                    /// Very close: stopped / stalled
                    if (cur - last).abs() < 0.05 * PI {
                        *rotation.write() = Rotation::Stopped;
                    }

                    /// Full cycle: transition from <tau to >0
                    if last > 1.9 * PI && cur < 0.1 * PI || last < 0.1 * PI && cur > 1.9 * PI {
                        // TODO Emit event for full cycle
                    } else if cur > last {
                        *rotation.write() = Rotation::Clockwise;
                    } else if cur < last {
                        *rotation.write() = Rotation::Counterclockwise;
                    }
                }
            }
        }).expect("failed to attach interrupt timer");
    }

    pub async fn disable_feedback(&mut self) {
        self.in_pin.clear_async_interrupt().unwrap();
    }
}

impl ServoPwmFeedbackConfig {
    pub fn duration_to_radians(&self, duration: Duration) -> f32 {
        let mut period = 1.0 / self.freq as f32;
        let mut duration = duration.as_secs_f32();
        if duration < period * self.lower_bound {
            duration = period * self.lower_bound;
        } else if duration > period * self.upper_bound {
            duration = period * self.upper_bound;
        }
        period = period * (self.upper_bound - self.lower_bound);
        let radians = duration / period * 2.0 * PI;

        radians
    }
}