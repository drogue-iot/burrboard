use drogue_device::*;
use embassy_nrf::uarte;

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

                }
            }
        }

macro_rules! info {
    ($l:ident, $s:literal $(, $x:expr)* $(,)?) => {
        {
            $l.notify($s)
            #[cfg(feature = "log")]
            ::log::info!($s $(, $x)*);
            #[cfg(feature = "defmt")]
            ::defmt::info!($s $(, $x)*);
            #[cfg(not(any(feature = "log", feature="defmt")))]
            let _ = ($( & $x ),*);
        }
    };
}
