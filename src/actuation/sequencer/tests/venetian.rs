use std::time::Duration;
use crate::actuation::sequencer::HOLD_TIME;
use crate::model::conf::MotorConf;
use crate::model::sequencer::{WindowDressingSequencer, WindowDressingServoInstruction, WindowDressingState};

fn conf() -> MotorConf {
    MotorConf::Servo {
        pulse_width_retract: 1900,
        pulse_width_extend: 1100,
        pulse_width_retract_tilt: 1900,
        pulse_width_extend_tilt: 1100,
        pulse_width_center: 1500,
        full_cycle_time: 100f32,
        full_tilt_time: Some(1.8),
    }
}

#[test]
fn open_full_sequence() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 0;
    seq.current_state.tilt = 90;
    seq.set_position(100);

    for i in -89..=90 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_millis(10),
            completed_state: WindowDressingState { position: 0, tilt: -i },
        }));
    }

    for i in 1..=100 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: i, tilt: -90 },
        }));
    }

    // Blinds should teleport back to previous tilt when fully opened
    assert_eq!(seq.get_next_instruction().unwrap().completed_state.tilt, 90);

    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 100, tilt: 90 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_full_tiltless_sequence() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 0;
    seq.current_state.tilt = -90;
    seq.set_position(100);

    for i in 1..=100 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: i, tilt: -90 },
        }));
    }

    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 100, tilt: -90 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_partial_sequence() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 25;
    seq.current_state.tilt = 60;
    seq.set_position(75);

    for i in -59..=90 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_millis(10),
            completed_state: WindowDressingState { position: 25, tilt: -i },
        }));
    }

    for i in 26..=75 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: i, tilt: -90 },
        }));
    }

    for i in -89..=60 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1100,
            duration: Duration::from_millis(10),
            completed_state: WindowDressingState { position: 75, tilt: i },
        }));
    }
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 75, tilt: 60 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_partial_tiltless_sequence() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 25;
    seq.current_state.tilt = -90;
    seq.set_position(75);

    for i in 26..=75 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: i, tilt: -90 },
        }));
    }

    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 75, tilt: -90 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_trig_endstop() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 0;
    seq.current_state.tilt = -90;
    seq.set_position(90);

    for i in 1..=80 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: i, tilt: -90 },
        }));
    }

    seq.trig_endstop();

    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 100, tilt: 0 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_full_sequence() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 100;
    seq.current_state.tilt = -90;
    seq.set_position(0);

    // no tilt when position is 100
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1100,
        duration: Duration::from_secs(0),
        completed_state: WindowDressingState { position: 100, tilt: 90 },
    }));

    for i in -99..=0 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1100,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: -i as u8, tilt: 90 },
        }));
    }

    for i in -89..=90 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_millis(10),
            completed_state: WindowDressingState { position: 0, tilt: -i },
        }));
    }

    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 0, tilt: -90 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_full_tiltless_sequence() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 100;
    seq.current_state.tilt = 90;
    seq.set_position(0);

    for i in -99..=0 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1100,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: -i as u8, tilt: 90 },
        }));
    }
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 0, tilt: 90 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_partial_sequence() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 75;
    seq.current_state.tilt = -90;
    seq.set_position(25);

    for i in -89..=90 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1100,
            duration: Duration::from_millis(10),
            completed_state: WindowDressingState { position: 75, tilt: i },
        }));
    }

    for i in -74..=-25 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1100,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: -i as u8, tilt: 90 },
        }));
    }

    for i in -89..=90 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_millis(10),
            completed_state: WindowDressingState { position: 25, tilt: -i },
        }));
    }

    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 25, tilt: -90 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_partial_tiltless_sequence() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 75;
    seq.current_state.tilt = 90;
    seq.set_position(25);

    for i in -74..=-25 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1100,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: -i as u8, tilt: 90 },
        }));
    }
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 25, tilt: 90 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_trig_endstop() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 100;
    seq.current_state.tilt = -90;
    seq.set_position(0);

    // no tilt when position is 100
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1100,
        duration: Duration::from_secs(0),
        completed_state: WindowDressingState { position: 100, tilt: 90 },
    }));

    for i in -99..=-20 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1100,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: -i as u8, tilt: 90 },
        }));
    }

    seq.trig_endstop();

    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 0, tilt: 90 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}
