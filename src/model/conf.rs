#[derive(Debug, Deserialize)]
pub struct BridgeConf {
    /// Set of name-value configurations for a collection of window dressings controlled by this bridge
    pub blinds: Vec<BlindConf>,
}

/// Configuration for an individual window dressing
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BlindConf {
    /// Name of this window dressing
    pub name: String,
    /// Wrapper over physical motor configuration e.g. stepper vs servomotor
    #[serde(flatten)]
    pub motor: MotorConf,
    /// Wrapper over control protocol e.g. local PWM vs BLE
    #[serde(flatten)]
    pub hw_mode: HwMode,
}

/// Defines the variant of open-loop or closed-loop motor used for actuating the blinds,
/// as well as its configuration
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MotorConf {
    Servo {
        /// PWM width to add to the phase in order to open the blinds
        pulse_width_delta: i16,
        /// PWM phase width where the continuous servo will be engaged but stationary
        #[serde(default = "default_servo_pulse_width")]
        pulse_width_center: i16,
        /// Time to go from 0 to 100% opened
        full_cycle_time: u16,
        /// Time to go from -90 to 90 degrees tilt, if [`None`], the tilt feature is disabled
        full_tilt_time: Option<u16>,
    }
}

fn default_servo_pulse_width() -> i16 {
    1500
}

/// Defines hardware modes used for actuating the blinds
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HwMode {
    /// Use an external Bluetooth peripheral to handle this window dressing
    #[cfg(feature = "hw_ble")]
    Ble {
        /// The Bluetooth peripheral name to select
        name: String
    },
    /// Use a mock testing adaptor to analyse the window dressing's behaviour
    #[cfg(test)]
    Mock,
    /// Use a locally-connected PWM channel to handle this window dressing
    #[cfg(feature = "hw_raspi")]
    Pwm {
        /// The PWM channel to select locally (on the Raspberry Pi)
        channel: u8
    },
}
