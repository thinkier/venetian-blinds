use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Conf {
    pub blinds: HashMap<String, BlindConf>,
}

impl Conf {
    pub fn read() -> Self {
        let conf_str = std::fs::read_to_string("Bridge.toml").unwrap();
        toml::from_str(&conf_str).unwrap()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlindConf {
    #[serde(flatten)]
    pub motor: MotorConf,
    #[serde(flatten)]
    pub hw_mode: HwMode,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MotorConf {
    Servo {
        /// PWM width to add to the phase in order to open the blinds
        phase_width_delta: i16,
        /// PWM phase width where the continuous servo will be stationary
        phase_width_center: i16,
        /// Time to go from 0 to 100% opened
        full_cycle_time: u16,
        /// Time to go from -90 to 90 degrees tilt
        full_tilt_time: Option<u16>,
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HwMode {
    // #[cfg(feature = "hw_raspi")]
    Pwm {
        channel: u8
    },
    // #[cfg(feature = "hw_ble")]
    Ble {
        periph_name: String
    },
}
