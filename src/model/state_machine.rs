use std::collections::VecDeque;
use std::time::Duration;
use crate::model::conf::MotorConf;

pub struct WindowDressingServoInstruction {
    pub pulse_width: i16,
    pub duration: Duration,
    pub completed_state: WindowDressingState,
}

pub struct WindowDressingSequencer {
    pub motor_conf: MotorConf,
    pub desired_state: WindowDressingState,
    pub current_state: WindowDressingState,
    pub instructions: VecDeque<WindowDressingServoInstruction>,
}

#[derive(Clone, Copy)]
pub struct WindowDressingState {
    pub position: u8,
    pub tilt: i8,
}
