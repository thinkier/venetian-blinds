use std::sync::Arc;
use rppal::gpio::InputPin;
use tokio::sync::RwLock;

pub mod imp;

pub struct ServoPwmFeedback {
    pub in_pin: InputPin,
    pub rotation: Arc<RwLock<Rotation>>,
    pub config: ServoPwmFeedbackConfig,
}

#[derive(Clone)]
pub struct ServoPwmFeedbackConfig {
    pub freq: u16,
    pub lower_bound: f32,
    pub upper_bound: f32,
}

pub enum Rotation {
    Clockwise,
    Counterclockwise,
    Stopped,
}