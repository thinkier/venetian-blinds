use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use crate::model::sequencer::{WindowDressingSequencer, WindowDressingServoInstruction};

pub async fn mock_backend(name: String, seq: &Arc<Mutex<WindowDressingSequencer>>) -> JoinHandle<()> {
    let seq = seq.clone();

    tokio::spawn(async move {
        // let mut start = Instant::now();
        let mut stacked: Option<WindowDressingServoInstruction> = None;

        loop {
            if let Some(i) = seq.lock().await.get_next_instruction() {
                if let Some(stacked_i) = &mut stacked {
                    if stacked_i.pulse_width == i.pulse_width {
                        *stacked_i += &i;
                    } else {
                        info!("{}: {:?}", name, stacked_i);
                        *stacked_i = i.clone();
                    }
                } else {
                    stacked = Some(i.clone());
                }

                // start += i.duration;
            } else {
                if let Some(stacked_i) = &stacked {
                    info!("{}: {:?}", name, stacked_i);

                    let state_file_name = { seq.lock().await.name.clone() };
                    let file = fs::OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(&state_file_name)
                        .await;
                    match file {
                        Ok(file) => {
                            if let Err(e) = seq.lock().await.save(file).await {
                                error!("Failed to save state for {}: {:?}", state_file_name, e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to open state file for {}: {:?}",state_file_name, e);
                        }
                    }
                }
                stacked = None;
                // start += Duration::from_millis(100);
            }
            // tokio::time::sleep_until(start).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
}
