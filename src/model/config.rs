use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControllersConfig {
    /// The declaration of all blinds names and attributes to run on this Raspberry Pi
    pub blinds: HashMap<String, VenetianBlind>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VenetianBlind {
    /// The PWM channel to use for this blind
    pub channel: u8,
    /// The fractional amount of rotations needed to move the blind
    /// from the fully extended state to the fully retracted state
    pub rotations_to_fully_extend: f32,
    /// The fractional amount of rotation needed to rotate through the range [-90, 90]
    pub rotations_to_fully_tilt: f32,
    /// Frequency of the feedback PWM signal
    #[serde(default = "default_feedback_freq")]
    pub feedback_freq: u32,
    /// The GPIO pin to use for the feedback PWM signal for this blind's servo
    pub feedback_pin: u8,
    /// The duty cycle value of the feedback PWM signal when the servo is at 0 degrees
    #[serde(default = "default_feedback_duty_cycle_lower_bound")]
    pub feedback_duty_cycle_lower_bound: f32
    /// The duty cycle value of the feedback PWM signal when the servo is at 359.99 degrees
    #[serde(default = "default_feedback_duty_cycle_upper_bound")]
    pub feedback_duty_cycle_upper_bound: f32,
}

// Default values in datasheet: https://www.digikey.com.au/en/htmldatasheets/production/2483575/0/0/1/900-00360

fn default_feedback_freq() -> u32 {
    910
}

fn default_feedback_duty_cycle_lower_bound() -> f32 {
    0.027
}

fn default_feedback_duty_cycle_upper_bound() -> f32 {
    0.971
}
