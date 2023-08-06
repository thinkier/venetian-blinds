use gethostname::gethostname;
use hap::accessory::AccessoryInformation;
use hap::accessory::bridge::BridgeAccessory;

pub fn get() -> BridgeAccessory {
    BridgeAccessory::new(1, AccessoryInformation {
        name: gethostname().into_string().unwrap(),
        manufacturer: "ACME Pty Ltd".into(),
        model: "Raspberry Pi".into(),
        ..Default::default()
    }).unwrap()
}
