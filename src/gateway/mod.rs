use std::sync::Arc;
use hap::accessory::{AccessoryCategory, AccessoryInformation};
use hap::{Config, MacAddress, Pin};
use hap::accessory::window_covering::WindowCoveringAccessory;
use hap::characteristic::{AsyncCharacteristicCallbacks};
use hap::futures::future::BoxFuture;
use hap::futures::FutureExt;
use hap::Result;
use hap::server::{IpServer, Server};
use hap::storage::{FileStorage, Storage};
use tokio::sync::Mutex;
use crate::model::conf::{BlindConf, BridgeConf, MotorConf};
use crate::model::gateway::Bridge;
use crate::model::sequencer::WindowDressingSequencer;

impl From<BridgeConf> for Bridge {
    fn from(conf: BridgeConf) -> Self {
        Bridge {
            conf
        }
    }
}

impl Bridge {
    fn parse_pin(&self) -> Result<Pin> {
        let mut pin = [0u8; 8];

        self.conf.pairing_pin.chars()
            .filter(|c| c.is_digit(10))
            .map(|c| c.to_digit(10).unwrap() as u8)
            .take(8)
            .enumerate()
            .for_each(|(i, d)| pin[i] = d);

        Pin::new(pin)
    }

    fn configure_accessory(&self, accessory: &mut WindowCoveringAccessory, config: &BlindConf) {
        accessory.window_covering.current_vertical_tilt_angle = None;
        accessory.window_covering.target_vertical_tilt_angle = None;
        accessory.window_covering.obstruction_detected = None;
        accessory.window_covering.hold_position = None;

        match config.motor {
            MotorConf::Servo { full_tilt_time, .. } => {
                let seq = Arc::new(Mutex::new(WindowDressingSequencer::from_conf(config.motor)));

                if full_tilt_time.is_none() {
                    accessory.window_covering.current_vertical_tilt_angle = None;
                    accessory.window_covering.target_vertical_tilt_angle = None;
                }
                if let Some(c) = &mut accessory.window_covering.current_horizontal_tilt_angle {
                    let seq = seq.clone();
                    c.on_read_async(Some(move || {
                        let seq = seq.clone();
                        async move {
                            let tilt = seq.lock().await.current_state.tilt as i32;
                            Ok(Some(tilt))
                        }.boxed()
                    }));
                    // TODO write
                }
                if let Some(c) = &mut accessory.window_covering.target_horizontal_tilt_angle {
                    let seq = seq.clone();
                    c.on_read_async(Some(move || {
                        let seq = seq.clone();
                        async move {
                            let tilt = seq.lock().await.desired_state.tilt as i32;
                            Ok(Some(tilt))
                        }.boxed()
                    }));
                    // TODO write
                }
                todo!()
            }
        }
    }

    fn accessories(&self) -> Result<Vec<WindowCoveringAccessory>> {
        let mut buf = vec![];

        let accessories = self.conf.blinds.iter().enumerate().map(|(id, blind)| {
            (WindowCoveringAccessory::new(id as u64 * 10, AccessoryInformation {
                name: blind.name.clone(),
                ..Default::default()
            }), blind)
        });

        for (mut accessory, config) in accessories {
            let mut accessory = accessory?;

            self.configure_accessory(&mut accessory, config);
            buf.push(accessory);
        }

        Ok(buf)
    }

    pub async fn start(&self) -> Result<()> {
        let mut storage = FileStorage::current_dir().await?;

        let config = match storage.load_config().await {
            Ok(mut config) => {
                config.redetermine_local_ip();
                storage.save_config(&config).await?;
                config
            }
            Err(_) => {
                let config = Config {
                    pin: self.parse_pin()?,
                    name: "Blinds Bridge".into(),
                    device_id: MacAddress::new([0xB8, 0x27, 0xEB, 69, 69, 69]),
                    category: AccessoryCategory::Bridge,
                    ..Default::default()
                };
                storage.save_config(&config).await?;
                config
            }
        };

        let server = IpServer::new(config, storage).await?;
        for accessory in self.accessories()? {
            server.add_accessory(accessory).await?;
        }

        let handle = server.run_handle();

        handle.await
    }
}
