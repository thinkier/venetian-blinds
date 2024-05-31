use std::collections::HashMap;

pub mod imp;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControllersConfig {
    /// The declaration of all blinds names and attributes to run on this Raspberry Pi
    pub blinds: HashMap<String, VenetianBlind>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VenetianBlind {
    /// The PWM channel to use for this blind
    pub channel: u8,
    /// The time needed to move the blind all the way down,
    /// accounting for speed-torque differences
    pub extend: ServoParams,
    /// The time needed to move the blind all the way up,
    /// accounting for speed-torque differences
    pub retract: ServoParams,
    /// The amount of time needed to use the extend/retract phase widths to rotate through the range [-90, 90]
    pub tilt_time: f32,
    // /// Frequency of the feedback PWM signal
    // #[serde(default = "default_feedback_freq")]
    // pub feedback_freq: u16,
    // /// The GPIO pin to use for the feedback PWM signal for this blind's servo
    // pub feedback_pin: u8,
    // /// The duty cycle value of the feedback PWM signal when the servo is at 0 degrees
    // #[serde(default = "default_feedback_duty_cycle_lower_bound")]
    // pub feedback_duty_cycle_lower_bound: f32,
    // /// The duty cycle value of the feedback PWM signal when the servo is at 359.99 degrees
    // #[serde(default = "default_feedback_duty_cycle_upper_bound")]
    // pub feedback_duty_cycle_upper_bound: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServoParams {
    /// The PWM ON phase width to use for actuating the servo
    pub phase_width: f32,
    /// The time to actuate the servo before stopping
    pub time: f32,
}

// Default values in datasheet: https://www.digikey.com.au/en/htmldatasheets/production/2483575/0/0/1/900-00360

fn default_feedback_freq() -> u16 {
    910
}

fn default_feedback_duty_cycle_lower_bound() -> f32 {
    0.027
}

fn default_feedback_duty_cycle_upper_bound() -> f32 {
    0.971
}
