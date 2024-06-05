use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use crate::model::sequencer::WindowDressingSequencer;

pub async fn mock_backend(name: String, seq: &Arc<Mutex<WindowDressingSequencer>>) -> JoinHandle<()> {
    let seq = seq.clone();

    tokio::spawn(async move {
        let mut start = Instant::now();
        loop {
            if let Some(i) = seq.lock().await.get_next_instruction() {
                info!("{}: {:?}", name, i);

                start += i.duration;
            } else {
                start += Duration::from_millis(100);
            }
            tokio::time::sleep_until(start).await;
        }
    })
}
