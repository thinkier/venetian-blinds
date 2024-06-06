use std::cmp::Ordering;
use std::collections::VecDeque;
use std::time::Duration;
use crate::model::conf::MotorConf;
use crate::model::sequencer::{WindowDressingServoInstruction, WindowDressingState, WindowDressingSequencer};

#[cfg(test)]
mod tests;

const HOLD_TIME: Duration = Duration::from_millis(500);

impl WindowDressingSequencer {
    pub fn from_conf(conf: MotorConf) -> Self {
        Self {
            motor_conf: conf,
            desired_state: WindowDressingState::default(),
            current_state: WindowDressingState::default(),
            instructions: VecDeque::new(),
        }
    }

    /// Retrieve the next instruction to send to the hardware, if present.
    pub fn get_next_instruction(&mut self) -> Option<WindowDressingServoInstruction> {
        if let Some(mut next) = self.instructions.pop_front() {
            self.current_state = next.completed_state;
            // Round the duration to the nearest microsecond to avoid floating point errors during testing
            next.duration = Duration::from_micros((next.duration.as_secs_f32() * 1e6).round() as u64);

            match &self.motor_conf {
                MotorConf::Servo { pulse_width_center, .. } =>
                // If the instructions queue is empty & it's not commanded to hold, buffer a hold command
                    if self.instructions.is_empty() && next.pulse_width != *pulse_width_center {
                        self.instructions.push_back(WindowDressingServoInstruction {
                            pulse_width: *pulse_width_center,
                            duration: HOLD_TIME,
                            completed_state: self.current_state,
                        });
                    },
            }

            Some(next)
        } else { None }
    }

    /// Command from HAP to set the position of the window dressing.
    pub fn set_position(&mut self, opened: u8) {
        self.desired_state.position = opened;
        self.instructions.clear();
        let absolute_change = (opened as i8 - self.current_state.position as i8).abs();
        if absolute_change == 0 { return; }

        let opening = opened > self.current_state.position;
        let mut angle_while_moving = if opening { -90 } else { 90 };

        self.add_tilt(self.current_state.tilt, angle_while_moving);
        match &self.motor_conf {
            MotorConf::Servo {
                pulse_width_center, pulse_width_delta,
                full_cycle_time, full_tilt_time,
            } => for percentage_change in 1..=absolute_change {
                if full_tilt_time.is_none() {
                    angle_while_moving = 0;
                }

                let mut pulse_width = *pulse_width_center;
                let mut relative_change = percentage_change as i8;
                if opening {
                    pulse_width += *pulse_width_delta;
                } else {
                    pulse_width -= *pulse_width_delta;
                    relative_change *= -1;
                }

                let position = (self.current_state.position as i8 + relative_change) as u8;
                self.instructions.push_back(WindowDressingServoInstruction {
                    pulse_width,
                    duration: Duration::from_nanos((full_cycle_time * 1e9) as u64) / 100,
                    completed_state: WindowDressingState {
                        position,
                        tilt: if position == 100 { 0 } else { angle_while_moving },
                    },
                });
            }
        }
        if opened < 100 {
            self.add_tilt(angle_while_moving, self.current_state.tilt);
        }
    }

    /// Command from HAP to set the tilt of the window dressing.
    pub fn set_tilt(&mut self, angle: i8) {
        self.add_tilt(self.get_tail_state().tilt, angle);
    }

    /// Get the desired state of the window dressing, as defined by the last command.
    fn get_tail_state(&self) -> WindowDressingState {
        self.instructions.back()
            .map_or(self.current_state, |i| i.completed_state)
    }

    /// Schedules the command necessary to tilt the window dressing.
    fn add_tilt(&mut self, from_angle: i8, to_angle: i8) {
        let MotorConf::Servo {
            pulse_width_center, pulse_width_delta,
            full_tilt_time, ..
        } = &self.motor_conf;
        if let Some(full_tilt_time) = full_tilt_time {
            self.desired_state.tilt = to_angle;
            let opening = to_angle < from_angle;
            let absolute_change = (to_angle as i16 - from_angle as i16).abs();
            if absolute_change == 0 { return; }

            for angle_change in 1..=absolute_change {
                let tilt = if opening {
                    from_angle as i16 - angle_change
                } else {
                    from_angle as i16 + angle_change
                } as i8;

                let position = self.get_tail_state().position;

                if position == 100 {
                    self.instructions.push_back(WindowDressingServoInstruction {
                        pulse_width: *pulse_width_center,
                        duration: HOLD_TIME,
                        completed_state: WindowDressingState {
                            position,
                            tilt,
                        },
                    });
                } else {
                    self.instructions.push_back(WindowDressingServoInstruction {
                        pulse_width: pulse_width_center + if opening { *pulse_width_delta } else { -pulse_width_delta },
                        duration: Duration::from_nanos((full_tilt_time * 1e9) as u64) / 180,
                        completed_state: WindowDressingState {
                            position,
                            tilt,
                        },
                    });
                }
            }
        }
    }

    /// Feedback from hardware that the endstop has been triggered.
    pub fn trig_endstop(&mut self) {
        self.instructions.clear();

        let opening = if self.current_state.position == self.desired_state.position {
            self.current_state.tilt > self.desired_state.tilt
        } else {
            self.current_state.position < self.desired_state.position || self.current_state.position == 100
        };
        let tilt = if let MotorConf::Servo { full_tilt_time: Some(_), .. } = &self.motor_conf {
            if opening { 0 } else { 90 }
        } else { 0 };
        let end_state = WindowDressingState {
            position: if opening { 100 } else { 0 },
            tilt,
        };

        self.current_state = end_state;
        self.desired_state = end_state;
        self.instructions.push_back(WindowDressingServoInstruction {
            pulse_width: 1500,
            duration: HOLD_TIME,
            completed_state: end_state,
        });
    }
}

impl PartialOrd for WindowDressingState {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WindowDressingState {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if self.position == other.position {
            other.tilt.cmp(&&self.tilt)
        } else {
            self.position.cmp(&other.position)
        }
    }
}
