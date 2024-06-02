use crate::model::conf::{BlindConf, BridgeConf, HwMode, MotorConf};

#[test]
fn test_deserialize_mock_cfg() {
    let mut conf = BridgeConf::read_with_name("examples/Mock.toml");
    assert_eq!(conf.blinds.len(), 2);

    assert_eq!(conf.blinds.remove(0), BlindConf {
        name: "MockRoller".to_string(),
        motor: MotorConf::Servo {
            pulse_width_delta: 400,
            pulse_width_center: 1500,
            full_cycle_time: 100f32,
            full_tilt_time: None,
        },
        hw_mode: HwMode::Mock,
    });

    assert_eq!(conf.blinds.remove(0), BlindConf {
        name: "MockVenetian".to_string(),
        motor: MotorConf::Servo {
            pulse_width_delta: 400,
            pulse_width_center: 1500,
            full_cycle_time: 100f32,
            full_tilt_time: Some(1f32),
        },
        hw_mode: HwMode::Mock,
    });
}

#[cfg(feature = "hw_ble")]
#[test]
fn test_deserialize_ble_cfg() {
    let mut conf = BridgeConf::read_with_name("examples/Ble.toml");
    assert_eq!(conf.blinds.len(), 2);

    assert_eq!(conf.blinds.remove(0), BlindConf {
        name: "BleRoller".to_string(),
        motor: MotorConf::Servo {
            pulse_width_delta: 400,
            pulse_width_center: 1500,
            full_cycle_time: 100f32,
            full_tilt_time: None,
        },
        hw_mode: HwMode::Ble {
            name: "BleRoller".to_string(),
        },
    });

    assert_eq!(conf.blinds.remove(0), BlindConf {
        name: "BleVenetian".to_string(),
        motor: MotorConf::Servo {
            pulse_width_delta: 400,
            pulse_width_center: 1500,
            full_cycle_time: 100f32,
            full_tilt_time: Some(1f32),
        },
        hw_mode: HwMode::Ble {
            name: "BleVenetian DEAD BEEF".to_string(),
        },
    });
}

#[cfg(feature = "hw_pwm")]
#[test]
fn test_deserialize_pwm_cfg() {
    let mut conf = BridgeConf::read_with_name("examples/Pwm.toml");
    assert_eq!(conf.blinds.len(), 2);

    assert_eq!(conf.blinds.remove(0), BlindConf {
        name: "PwmRoller".to_string(),
        motor: MotorConf::Servo {
            pulse_width_delta: 400,
            pulse_width_center: 1500,
            full_cycle_time: 100f32,
            full_tilt_time: None,
        },
        hw_mode: HwMode::Pwm {
            channel: 0,
        },
    });

    assert_eq!(conf.blinds.remove(0), BlindConf {
        name: "PwmVenetian".to_string(),
        motor: MotorConf::Servo {
            pulse_width_delta: 400,
            pulse_width_center: 1500,
            full_cycle_time: 100f32,
            full_tilt_time: Some(1f32),
        },
        hw_mode: HwMode::Pwm {
            channel: 1,
        },
    });
}
