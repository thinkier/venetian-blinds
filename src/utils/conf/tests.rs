use crate::model::conf::{BlindConf, BridgeConf, HwMode, MotorConf};

#[test]
fn mock() {
    let mut conf = BridgeConf::read_with_name("examples/Mock.toml");
    assert_eq!(conf.blinds.len(), 2);

    assert_eq!(conf.blinds.remove(0), BlindConf {
        name: "MockRoller".to_string(),
        motor: MotorConf::Servo {
            pulse_width_retract: 1000,
            pulse_width_extend: 1700,
            pulse_width_center: 1500,
            full_cycle_time: 100f32,
            full_tilt_time: None,
        },
        backend: HwMode::Mock,
    });

    assert_eq!(conf.blinds.remove(0), BlindConf {
        name: "MockVenetian".to_string(),
        motor: MotorConf::Servo {
            pulse_width_retract: 1000,
            pulse_width_extend: 1700,
            pulse_width_center: 1500,
            full_cycle_time: 100f32,
            full_tilt_time: Some(1f32),
        },
        backend: HwMode::Mock,
    });
}

#[cfg(feature = "hw_ble")]
#[test]
fn ble() {
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
        backend: HwMode::Ble {
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
        backend: HwMode::Ble {
            name: "BleVenetian DEAD BEEF".to_string(),
        },
    });
}

#[cfg(feature = "hw_pwm")]
#[test]
fn pwm() {
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
        backend: HwMode::Pwm {
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
        backend: HwMode::Pwm {
            channel: 1,
        },
    });
}
