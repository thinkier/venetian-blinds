use hap::characteristic::OnUpdateFuture;
use hap::futures::FutureExt;
use crate::actuation::controller::Controller;

impl Controller {
    pub fn update_tilt_async(&'static self) -> impl OnUpdateFuture<i32> {
        move |cur, new| {
            async move {
                info!("Setting tilt from {} to {}", cur, new);
                self.set_tilt(new as i8).await;
                Ok(())
            }.boxed()
        }
    }

    pub fn update_pos_async(&'static self) -> impl OnUpdateFuture<u8> {
        move |cur, new| {
            async move {
                info!("Setting position from {} to {}", cur, new);
                let _ = tokio::spawn(async move {
                    self.set_position(new).await;
                });
                Ok(())
            }.boxed()
        }
    }
}
