use hap::accessory::{AccessoryCategory, AccessoryInformation};
use hap::{Config, MacAddress, Pin};
use hap::accessory::window_covering::WindowCoveringAccessory;
use hap::characteristic::{AsyncCharacteristicCallbacks};
use hap::futures::FutureExt;
use hap::Result;
use hap::server::{IpServer, Server};
use hap::storage::{FileStorage, Storage};
use crate::actuation::backend::mock::mock_backend;
#[cfg(feature = "hw_raspi")]
use crate::actuation::backend::pi_pwm::pi_pwm_backend;
use crate::model::conf::{BridgeConf, HwMode, MotorConf};
use crate::model::gateway::{BlindInstance, Bridge};
use crate::model::sequencer::WindowDressingSequencer;

macro_rules! chardef {
    ($seq:ident, $ch:expr, $read_fn:expr $(,$($write_fn:expr)?)?) => {
        if let Some(ch) = &mut $ch {
            let seq = $seq.clone();
            ch.on_read_async(Some(move || {
                let seq = seq.clone();
                async move {
                    let mut seq = seq.lock().await;
                    Ok(Some($read_fn(&mut seq)))
                }.boxed()
            }));

            $($(
                let seq = $seq.clone();
                ch.on_update_async(Some(move |_old, new| {
                    let seq = seq.clone();
                    async move {
                        let mut seq = seq.lock().await;
                        Ok($write_fn(&mut seq, new))
                    }.boxed()
                }));
            )?)?
        }
    }
}

impl<'a> Bridge<'a> {
    pub async fn new(bridge_conf: &'a BridgeConf) -> Self {
        let mut bridge = Bridge {
            conf: bridge_conf,
            blinds: vec![],
        };

        for conf in &bridge.conf.blinds {
            let seq = conf.motor.to_sequencer(&conf.name).await;

            let backend = match &conf.backend {
                HwMode::Mock => {
                    mock_backend(conf.name.clone(), &seq).await
                }
                #[cfg(feature = "hw_raspi")]
                HwMode::Pwm { channel } => {
                    pi_pwm_backend(*channel, &seq).await
                }
                #[cfg(any(feature = "hw_ble", all(feature = "hw_pwm", not(feature = "hw_raspi"))))]
                m => unimplemented!("{:?} is not implemented", m)
            };

            bridge.blinds.push(BlindInstance {
                conf,
                seq,
                backend,
            });
        }

        return bridge;
    }

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

    fn configure_accessory(&self, accessory: &mut WindowCoveringAccessory, inst: &'a BlindInstance<'a>) {
        accessory.window_covering.current_vertical_tilt_angle = None;
        accessory.window_covering.target_vertical_tilt_angle = None;
        accessory.window_covering.obstruction_detected = None;
        accessory.window_covering.hold_position = None;

        match inst.conf.motor {
            MotorConf::Servo { full_tilt_time, .. } => {
                let seq = inst.seq.clone();
                if full_tilt_time.is_none() {
                    accessory.window_covering.current_vertical_tilt_angle = None;
                    accessory.window_covering.target_vertical_tilt_angle = None;
                }
                chardef!(seq, Some(&mut accessory.window_covering.current_position),
                    |seq: &mut WindowDressingSequencer| { seq.current_state.position }
                );

                chardef!(seq, Some(&mut accessory.window_covering.position_state),
                    |seq: &mut WindowDressingSequencer| {
                        // https://developers.homebridge.io/#/characteristic/PositionState
                        match seq.current_state.position.cmp(&seq.desired_state.position) {
                            std::cmp::Ordering::Less => 0, // Decreasing
                            std::cmp::Ordering::Greater => 1, // Increasing
                            std::cmp::Ordering::Equal => 2, // Stopped
                        }
                    }
                );

                chardef!(seq, Some(&mut accessory.window_covering.target_position),
                    |seq: &mut WindowDressingSequencer| { seq.desired_state.position },
                    |seq: &mut WindowDressingSequencer, new| { seq.set_position(new) }
                );

                chardef!(seq, accessory.window_covering.current_horizontal_tilt_angle,
                    |seq: &mut WindowDressingSequencer| { seq.current_state.tilt as i32 }
                );

                chardef!(seq, accessory.window_covering.target_horizontal_tilt_angle,
                    |seq: &mut WindowDressingSequencer| { seq.desired_state.tilt  as i32},
                    |seq: &mut WindowDressingSequencer, new| { seq.set_tilt(new as i8) }
                );
            }
        }
    }

    fn accessories(&mut self) -> Result<Vec<WindowCoveringAccessory>> {
        let mut accessories_buf = vec![];

        let accessories = self.blinds.iter().enumerate().map(|(id, inst)| {
            (WindowCoveringAccessory::new((id as u64 * 10) + 1, AccessoryInformation {
                name: inst.conf.name.clone(),
                ..Default::default()
            }), inst)
        });

        for (accessory, inst) in accessories {
            let mut accessory = accessory?;

            self.configure_accessory(&mut accessory, inst);
            accessories_buf.push(accessory);
        }

        Ok(accessories_buf)
    }

    pub async fn start(&mut self) -> Result<()> {
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

impl<'a> Drop for Bridge<'a> {
    fn drop(&mut self) {
        for inst in &self.blinds {
            inst.backend.abort();
        }
    }
}
