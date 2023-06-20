#[macro_use]
extern crate log;

use tokio;

use hap::{
    server::{IpServer, Server},
    storage::FileStorage,
    Result,
};
use crate::controller::{Controller, ControllerConfig};

mod server;
mod controller;

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "hap=debug,libmdns=error,info");
    env_logger::init();

    let mut storage = FileStorage::current_dir().await?;
    let config = server::config::get(&mut storage).await?;

    let controller = Controller::from_config(ControllerConfig::load().await?);

    let server = IpServer::new(config, storage).await?;
    server.add_accessory(server::bridge::get()).await?;

    for i in 0..controller.config.accessory_count {
        server.add_accessory(server::accessory::get(controller.clone(), i)).await?;
    }

    server.run_handle().await?;
    Ok(())
}
