use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControllersConfig {
    pub blinds: HashMap<String, VenetianBlind>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VenetianBlind {
    pub channel: u8,
    pub period_extend: u32,
    pub period_tilt: u32,
}
