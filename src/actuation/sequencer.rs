use std::time::Duration;
use crate::model::conf::MotorConf;
use crate::model::state_machine::{WindowDressingServoInstruction, WindowDressingState, WindowDressingSequencer};

const HOLD_TIME: Duration = Duration::from_millis(500);

impl WindowDressingSequencer {
    /// Retrieve the next instruction to send to the hardware
    pub fn get_next_instruction(&mut self) -> WindowDressingServoInstruction {
        if let Some(next) = self.instructions.pop_front() {
            self.current_state = next.completed_state;
            next
        } else {
            match &self.motor_conf {
                MotorConf::Servo { pulse_width_center, .. } => WindowDressingServoInstruction {
                    pulse_width: *pulse_width_center,
                    duration: HOLD_TIME,
                    completed_state: self.current_state,
                },
            }
        }
    }

    /// Retrieve the position of the window dressing according to the state machine.
    pub fn get_position(&self) -> u8 {
        self.current_state.position
    }

    /// Retrieve the tilt of the window dressing according to the state machine.
    pub fn get_tilt(&self) -> i8 {
        self.current_state.tilt
    }

    /// Command from HAP to set the position of the window dressing.
    pub fn set_position(&mut self, opened: u8) {
        self.instructions.clear();
        let absolute_change = (opened as i8 - self.current_state.position as i8).abs();
        if absolute_change == 0 { return; }

        let opening = opened > self.current_state.position;
        let angle_while_moving = if opening { -90 } else { 90 };

        self.add_tilt(self.current_state.tilt, angle_while_moving);
        match &self.motor_conf {
            MotorConf::Servo {
                pulse_width_center, pulse_width_delta,
                full_cycle_time, ..
            } => for percentage_change in 1..=absolute_change {
                let mut pulse_width = *pulse_width_center;
                if opening {
                    pulse_width += *pulse_width_delta;
                } else {
                    pulse_width -= *pulse_width_delta;
                }

                self.instructions.push_back(WindowDressingServoInstruction {
                    pulse_width,
                    duration: Duration::from_secs(*full_cycle_time as u64) / 100,
                    completed_state: WindowDressingState {
                        position: self.current_state.position + percentage_change as u8,
                        tilt: self.current_state.tilt,
                    },
                });
            }
        }
        self.add_tilt(angle_while_moving, self.current_state.tilt);
    }

    /// Command from HAP to set the tilt of the window dressing.
    pub fn set_tilt(&mut self, angle: i8) {
        self.add_tilt(self.current_state.tilt, angle);
    }

    fn add_tilt(&mut self, from_angle: i8, to_angle: i8) {
        unimplemented!()
    }

    /// Feedback from hardware that the endstop has been triggered.
    pub fn trig_endstop(&mut self) {
        unimplemented!()
    }
}
