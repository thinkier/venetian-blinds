use std::sync::{Arc};
use hap::accessory::{AccessoryInformation, HapAccessory};
use hap::accessory::window_covering::WindowCoveringAccessory;
use hap::characteristic::{AsyncCharacteristicCallbacks, CharacteristicCallbacks};
use hap::characteristic::position_state::PositionStateCharacteristic;
use hap::futures::FutureExt;
use hap::service::window_covering::WindowCoveringService;
use parking_lot::RwLock;
use crate::model::controller::Controller;

pub fn get(ctr: Controller, name: &str, index: usize) -> impl HapAccessory {
    let mut venetian = WindowCoveringAccessory::new((2 + index) as u64, AccessoryInformation {
        name: format!("Venetian Blinds {}", name),
        manufacturer: "ACME Pty Ltd".into(),
        model: "Raspberry Pi".into(),
        ..Default::default()
    }).unwrap();

    let ctr = Box::leak(Box::new(ctr));
    initialize_characteristics(&mut venetian.window_covering, ctr);

    return venetian;
}

fn initialize_characteristics(window_covering: &mut WindowCoveringService, ctr: &'static Controller) {
    window_covering.current_horizontal_tilt_angle = None;
    window_covering.target_horizontal_tilt_angle = None;
    window_covering.hold_position = None;
    window_covering.obstruction_detected = None;

    if let Some(tvta) = &mut window_covering.target_vertical_tilt_angle {
        tvta.on_update_async(Some(ctr.update_tilt_async()));
    }

    if let Some(cvta) = &mut window_covering.current_vertical_tilt_angle {
        cvta.on_read(Some(move || {
            Ok(Some(ctr.get_tilt() as i32 * 90 / 128))
        }));
    }

    {
        let tp = &mut window_covering.target_position;
        tp.on_update_async(Some(ctr.update_pos_async()));
    }

    {
        let cp = &mut window_covering.current_position;
        cp.on_read(Some(|| {
            Ok(Some(ctr.get_position() as i32 * 100 / 255))
        }));
    }

    {
        let ps = &mut window_covering.position_state;
        let ctr = ctr.clone();
        ps.on_read(Some(|| {
            Ok(Some(if ctr.active.try_lock().is_ok() {
                // 1 // Increasing
                0 // Decreasing
            } else {
                2 // Stopped
            }))
        }));
    }
}
