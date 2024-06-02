use std::time::Duration;
use crate::actuation::sequencer::HOLD_TIME;
use crate::model::conf::{MotorConf};
use crate::model::sequencer::{WindowDressingSequencer, WindowDressingServoInstruction, WindowDressingState};

fn conf() -> MotorConf {
    MotorConf::Servo {
        pulse_width_delta: 400,
        pulse_width_center: 1500,
        full_cycle_time: 100f32,
        full_tilt_time: None,
    }
}

#[test]
fn desired_state_updates() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.set_position(69);
    assert_eq!(seq.desired_state, WindowDressingState { position: 69, tilt: 0 });
}

#[test]
fn current_state_updates() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 0;
    seq.set_position(69);
    for i in 1..=69 {
        seq.get_next_instruction();
        assert_eq!(seq.current_state, WindowDressingState { position: i, tilt: 0 });
    }
}

#[test]
fn open_fully() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 0;
    seq.set_position(100);

    for i in 1..=100 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: i, tilt: 0 },
        }));
    }
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 100, tilt: 0 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_partially() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 25;
    seq.set_position(75);

    for i in 1..=50 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: i + 25, tilt: 0 },
        }));
    }
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 75, tilt: 0 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_fully() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 100;
    seq.set_position(0);

    for j in 1..=100 {
        let i = 100 - j;
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1100,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: i, tilt: 0 },
        }));
    }
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 0, tilt: 0 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_partially() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 75;
    seq.set_position(25);

    for j in 1..=50 {
        let i = 75 - j;
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1100,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: i, tilt: 0 },
        }));
    }
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 25, tilt: 0 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_trig_endstop() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 0;
    seq.set_position(100);

    for i in 1..=50 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1900,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: i, tilt: 0 },
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
fn close_trig_endstop() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.position = 100;
    seq.set_position(0);

    for i in 1..=50 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 1100,
            duration: Duration::from_secs(1),
            completed_state: WindowDressingState { position: 100 - i, tilt: 0 },
        }));
    }

    seq.trig_endstop();
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 0, tilt: 0 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}
