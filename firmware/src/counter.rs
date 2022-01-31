use core::future::Future;
use drogue_device::{Actor, Address, Inbox};

pub struct Counter {
    presses: u32,
}

impl Counter {
    pub fn new() -> Self {
        Self { presses: 0 }
    }
}

#[derive(Clone, Copy)]
pub struct Increment;

impl Actor for Counter {
    type Message<'m> = Increment;

    type OnMountFuture<'m, M>
    where
        Self: 'm,
        M: 'm,
    = impl Future<Output = ()> + 'm;

    fn on_mount<'m, M>(
        &'m mut self,
        _: Address<Self>,
        inbox: &'m mut M,
    ) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self> + 'm,
    {
        async move {
            loop {
                if let Some(_) = inbox.next().await {
                    self.presses += 1;
                    info!("Presses: {}", self.presses);
                }
            }
        }
    }
}
