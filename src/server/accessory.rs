use hap::accessory::{AccessoryInformation, HapAccessory};
use hap::accessory::window_covering::WindowCoveringAccessory;
use hap::characteristic::{CharacteristicCallbacks};
use hap::service::window_covering::WindowCoveringService;
use crate::controller::Controller;

pub fn get(controller: Controller, index: u8) -> impl HapAccessory {
    let mut venetian = WindowCoveringAccessory::new(2 + index as u64, AccessoryInformation {
        name: format!("Venetian Blinds {}", index),
        manufacturer: "ACME Pty Ltd".into(),
        model: "Raspberry Pi 4B 2GB + BigTreeTech SKR Pico v1.0".into(),
        ..Default::default()
    }).unwrap();

    initialize_characteristics(&mut venetian.window_covering, controller, index);

    return venetian;
}

fn initialize_characteristics(window_covering: &mut WindowCoveringService, controller: Controller, index: u8) {
    window_covering.current_horizontal_tilt_angle = None;
    window_covering.target_horizontal_tilt_angle = None;
    window_covering.hold_position = None;
    window_covering.obstruction_detected = None;


    if let Some(tvta) = &mut window_covering.target_vertical_tilt_angle {
        tvta.on_read(Some(|| {
            Ok(Some(0))
        }));
    }

    if let Some(cvta) = &mut window_covering.current_vertical_tilt_angle {
        cvta.on_read(Some(|| {
            Ok(Some(0))
        }));
    }

    {
        let tp = &mut window_covering.target_position;

        tp.on_read(Some(|| {
            Ok(Some(100))
        }));
    }

    {
        let cp = &mut window_covering.current_position;

        cp.on_read(Some(|| {
            Ok(Some(100))
        }));
    }

    {
        let ps = &mut window_covering.position_state;

        ps.on_read(Some(|| {
            Ok(Some(2))
        }));
    }
}
