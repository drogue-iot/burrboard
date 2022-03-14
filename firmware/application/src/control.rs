use crate::app::App;
use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
use embassy::time::{Duration, Instant};
use embassy_nrf::gpio::{AnyPin, Input};

pub struct ControlButton {
    app: &'static App,
    button: Input<'static, AnyPin>,
}

impl ControlButton {
    pub fn new(app: &'static App, button: Input<'static, AnyPin>) -> Self {
        Self { app, button }
    }
}

impl Actor for ControlButton {
    type OnMountFuture<'m, M> = impl Future<Output = ()> + 'm
    where
        Self: 'm,
        M: 'm + Inbox<Self>;
    fn on_mount<'m, M>(&'m mut self, _: Address<Self>, _: &'m mut M) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self> + 'm,
        Self: 'm,
    {
        async move {
            loop {
                self.button.wait_for_low().await;
                let now = Instant::now();
                self.button.wait_for_high().await;
                self.button.wait_for_low().await;
                if Instant::now() - now < Duration::from_millis(300) {
                    self.app.switch();
                }
            }
        }
    }
}
