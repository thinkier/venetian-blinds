use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use crate::model::conf::{BlindConf, BridgeConf};
use crate::model::sequencer::WindowDressingSequencer;

pub struct BlindInstance<'a> {
    pub conf: &'a BlindConf,
    pub seq: Arc<Mutex<WindowDressingSequencer>>,
    pub backend: JoinHandle<()>,
}

pub struct Bridge<'a> {
    pub conf: &'a BridgeConf,
    pub blinds: Vec<BlindInstance<'a>>,
}
