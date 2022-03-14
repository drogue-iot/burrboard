use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
use embassy::time::{Duration, Ticker};
use futures::StreamExt;

pub struct Watchdog(pub Duration);

impl Actor for Watchdog {
    type Message<'m> = ();
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
            let mut ticker = Ticker::every(self.0);
            let mut handle = unsafe { embassy_nrf::wdt::WatchdogHandle::steal(0) };
            loop {
                handle.pet();
                ticker.next().await;
            }
        }
    }
}
