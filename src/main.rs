#![feature(async_closure)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod actuation;
mod gateway;
mod model;
mod utils;

use tokio;
use hap::Result;
use crate::model::conf::BridgeConf;
use crate::model::gateway::Bridge;

fn check_env() {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "hap=debug,info");
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    check_env();
    env_logger::init();
    let conf = BridgeConf::read();

    let mut bridge = Bridge::new(&conf).await;

    bridge.start().await
}
