use std::time::Duration;
use crate::actuation::sequencer::HOLD_TIME;
use crate::model::conf::MotorConf;
use crate::model::sequencer::{WindowDressingSequencer, WindowDressingServoInstruction, WindowDressingState};

fn conf() -> MotorConf {
    MotorConf::Servo {
        pulse_width_retract: 1900,
        pulse_width_extend: 1100,
        pulse_width_retract_tilt: 2500,
        pulse_width_extend_tilt: 500,
        pulse_width_center: 1500,
        full_cycle_time: 100f32,
        full_tilt_time: Some(1.8),
    }
}

#[test]
fn desired_state_updates() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.set_tilt(69);
    assert_eq!(seq.desired_state.tilt, 69);
}

#[test]
fn current_state_updates() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.tilt = 0;
    seq.set_tilt(69);
    for i in 1..=69 {
        seq.get_next_instruction();
        assert_eq!(seq.current_state.tilt, i);
    }
}

#[test]
fn close_full() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.tilt = -90;
    seq.set_tilt(90);
    for i in -89..=90 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 500,
            duration: Duration::from_millis(10),
            completed_state: WindowDressingState { position: 0, tilt: i },
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
fn open_full() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.tilt = 90;
    seq.set_tilt(-90);
    for i in -89..=90 {
        assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
            pulse_width: 2500,
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
fn close_trig_endstop() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.tilt = -90;
    seq.set_tilt(90);

    let _ = (0..90).map(|_| seq.get_next_instruction());
    seq.trig_endstop();
    assert_eq!(seq.current_state, WindowDressingState {
        position: 0,
        tilt: 90,
    });
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 0, tilt: 90 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_trig_endstop() {
    let conf = conf();
    let mut seq = WindowDressingSequencer::from_conf(conf);
    seq.current_state.tilt = 90;
    seq.set_tilt(-90);

    let _ = (0..90).map(|_| seq.get_next_instruction());
    seq.trig_endstop();
    assert_eq!(seq.current_state, WindowDressingState {
        position: 100,
        tilt: 0,
    });
    assert_eq!(seq.get_next_instruction(), Some(WindowDressingServoInstruction {
        pulse_width: 1500,
        duration: HOLD_TIME,
        completed_state: WindowDressingState { position: 100, tilt: 0 },
    }));
    assert_eq!(seq.get_next_instruction(), None);
}
