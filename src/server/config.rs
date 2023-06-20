use hap::{
    accessory::AccessoryCategory,
    storage::{Storage},
    Config,
    // MacAddress,
    Pin,
    Result,
};

pub async fn get<S: Storage + Send + Sync + 'static>(storage: &mut S) -> Result<Config> {
    match storage.load_config().await {
        Ok(mut config) => {
            config.redetermine_local_ip();
            storage.save_config(&config).await?;
            Ok(config)
        }
        Err(_) => {
            let config = Config {
                pin: Pin::new([1, 5, 6, 7, 1, 6, 2, 7])?,
                name: "Venetian Blinds".into(),
                // device_id: MacAddress::parse_str("d8:3a:dd:01:1b:01")?,
                category: AccessoryCategory::WindowCovering,
                ..Default::default()
            };
            storage.save_config(&config).await?;
            Ok(config)
        }
    }
}
