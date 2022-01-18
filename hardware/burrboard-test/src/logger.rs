use drogue_device::*;
use embassy_nrf::uarte;
use log::{Level, Metadata, Record};

pub struct UartLogger<T: uarte::Instance> {
    uart: uarte::Uarte<'static, T>,
}

impl<T: uarte::Instance> UartLogger<T> {
    pub fn new(uart: uarte::Uarte<'static, T>) -> Self {
        Self { uart }
    }
}

static LOGGER: ActorContext<UartLogger<UARTE0>> = ActorContext::new();

impl<T: uarte::Instance> Actor for UartLogger<T> {
    type Message<'m> = &'static str;

    // Workaround until async traits
    type OnMountFuture<'m, M>
    where
        M: 'm,
    = impl core::future::Future<Output = ()> + 'm;

    // Actor entry point
    fn on_mount<'m, M>(
        &'m mut self,
        _: Address<Self>,
        inbox: &'m mut M,
    ) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self> + 'm,
        Self: 'm,
    {
        async move {
            loop {
                if let Some(m) = inbox.next().await {
                    let m = *m.message();
                    self.uart.blocking_write(m.as_bytes());
                }
            }
        }
    }
}

macro_rules! print {
    ($s:literal $(, $x:expr)* $(,)?) => {
        LOGGER::address().notify($s).unwrap()
    };
}
