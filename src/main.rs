#![feature(async_closure)]

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
use crate::model::config::ControllersConfig;
use crate::model::controller::Controllers;

mod imp;
mod model;
mod proxy;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "proxy=debug,libmdns=error,info");
    }
    env_logger::init();

    let mut storage = FileStorage::current_dir().await?;
    let config = proxy::config::get(&mut storage).await?;

    let controllers = Controllers::from_config(ControllersConfig::load().await?)?;

    let server = IpServer::new(config, storage).await?;
    server.add_accessory(proxy::bridge::get()).await?;

    for (i, name) in controllers.config.blinds.keys().enumerate() {
        server.add_accessory(proxy::accessory::get(controllers.get_instance(name), name.as_ref(), i)).await?;
    }

    server.run_handle().await?;
    Ok(())
}
