use std::collections::HashMap;
use crate::actuation::config::ControllersConfig;
use hap::Result;
use tokio::fs;

impl ControllersConfig {
    pub async fn load() -> Result<ControllersConfig> {
        let path = "Accessory.toml";

        return match fs::try_exists(path).await {
            Ok(true) => {
                let toml = fs::read_to_string(path).await?;

                Ok(toml::from_str(&toml).unwrap())
            }
            _ => {
                let config = ControllersConfig {
                    blinds: HashMap::new(),
                };

                let toml = toml::to_string(&config).unwrap();

                fs::write(path, toml).await?;

                Ok(config)
            }
        };
    }
}
