use std::collections::VecDeque;
use std::time::Duration;
use crate::model::conf::MotorConf;

#[derive(Debug, PartialEq)]
pub struct WindowDressingServoInstruction {
    pub pulse_width: i16,
    pub duration: Duration,
    pub completed_state: WindowDressingState,
}

#[derive(Debug, PartialEq)]
pub struct WindowDressingSequencer {
    pub motor_conf: MotorConf,
    pub desired_state: WindowDressingState,
    pub current_state: WindowDressingState,
    pub instructions: VecDeque<WindowDressingServoInstruction>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WindowDressingState {
    pub position: u8,
    pub tilt: i8,
}
