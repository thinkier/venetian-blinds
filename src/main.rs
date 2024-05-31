#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod conf;

use tokio;
use hap::Result;
use crate::conf::Conf;

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
    let conf = Conf::read();

    debug!("Read configuration {:?}", conf);

    Ok(())
}
