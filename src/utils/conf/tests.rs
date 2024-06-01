use crate::model::conf::{BlindConf, BridgeConf, HwMode, MotorConf};

#[test]
fn test_deserialize_debug_cfg() {
    let mut conf = BridgeConf::read();
    assert_eq!(conf.blinds.len(), 2);

    assert_eq!(conf.blinds.remove("DebugRoller").unwrap(), BlindConf {
        motor: MotorConf::Servo {
            pulse_width_delta: 400,
            pulse_width_center: 1500,
            full_cycle_time: 100,
            full_tilt_time: None,
        },
        hw_mode: HwMode::Mock {
            name: "DebugRoller".to_string()
        },
    });

    assert_eq!(conf.blinds.remove("DebugVenetian").unwrap(), BlindConf {
        motor: MotorConf::Servo {
            pulse_width_delta: 400,
            pulse_width_center: 1500,
            full_cycle_time: 100,
            full_tilt_time: Some(1),
        },
        hw_mode: HwMode::Mock {
            name: "DebugVenetian".to_string()
        },
    });
}