use std::sync::Arc;
use std::time::Duration;
use rppal::pwm::{Channel, Polarity, Pwm};
use tokio::fs;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use crate::model::sequencer::{WindowDressingSequencer};

pub async fn pi_pwm_backend(channel: u8, seq: &Arc<Mutex<WindowDressingSequencer>>) -> JoinHandle<()> {
    let seq = seq.clone();
    let pwm = Pwm::with_period(
        match channel {
            0 => Channel::Pwm0,
            1 => Channel::Pwm1,
            _ => panic!("invalid pwm channel {}", channel),
        },
        Duration::from_millis(20),
        Duration::from_micros(1500),
        Polarity::Normal,
        false,
    ).unwrap();

    tokio::spawn(async move {
        let mut start = Instant::now();

        loop {
            let i = { seq.lock().await.get_next_instruction() };
            if let Some(i) = i {
                pwm.set_pulse_width(Duration::from_micros(i.pulse_width as u64)).unwrap();
                if pwm.is_enabled().map_or(false, |e| !e) {
                    let _ = pwm.enable().unwrap();
                }

                start += i.duration;
            } else {
                if pwm.is_enabled().unwrap_or(true) {
                    pwm.disable().unwrap();
                }
                let state_file_name = { seq.lock().await.name.clone() };
                let file = fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(&state_file_name)
                    .await;
                match file {
                    Ok(file) => {
                        if let Err(e) = { seq.lock().await.save(file).await } {
                            error!("Failed to save state for {}: {:?}", state_file_name, e);
                        } else {
                            info!("Saved state for {}", state_file_name);
                        }
                    }
                    Err(e) => {
                        error!("Failed to open state file for {}: {:?}",state_file_name, e);
                    }
                }
                start += Duration::from_millis(100);
            }
            tokio::time::sleep_until(start).await;
        }
    })
}
