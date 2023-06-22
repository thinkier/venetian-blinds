#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use tokio;

use hap::{
    server::{IpServer, Server},
    storage::FileStorage,
    Result,
};
use crate::controller::{Controller};
use crate::controller::config::ControllerConfig;

mod server;
mod controller;

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "hap=debug,libmdns=error,info");
    env_logger::init();

    let mut storage = FileStorage::current_dir().await?;
    let config = server::config::get(&mut storage).await?;

    let controller = Controller::from_config(ControllerConfig::load().await?)?;

    let server = IpServer::new(config, storage).await?;
    server.add_accessory(server::bridge::get()).await?;

    let mut index = 0;
    for name in controller.config.blinds.keys() {
        server.add_accessory(server::accessory::get(controller.get_instance(name), index)).await?;
    }

    server.run_handle().await?;
    Ok(())
}
