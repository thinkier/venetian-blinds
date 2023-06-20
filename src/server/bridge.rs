use hap::accessory::AccessoryInformation;
use hap::accessory::bridge::BridgeAccessory;

pub fn get() -> BridgeAccessory {
    BridgeAccessory::new(1, AccessoryInformation {
        name: "Venetian Blinds Bridge".into(),
        manufacturer: "ACME Pty Ltd".into(),
        model: "Raspberry Pi 4B 2GB + BigTreeTech SKR Pico v1.0".into(),
        ..Default::default()
    }).unwrap()
}
