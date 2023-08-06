use hap::characteristic::OnUpdateFuture;
use hap::futures::FutureExt;
use crate::model::controller::Controller;

impl Controller {
    pub fn update_tilt_async(&'static self) -> impl OnUpdateFuture<i32> {
        move |_cur, new| {
            async move {
                self.set_tilt(new as i8).await;
                Ok(())
            }.boxed()
        }
    }

    pub fn update_pos_async(&'static self) -> impl OnUpdateFuture<i32> {
        move |_cur, new| {
            async move {
                self.set_position(new as u8).await;
                Ok(())
            }.boxed()
        }
    }
}
