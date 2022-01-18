use core::fmt::Write;
use drogue_device::*;
use embassy::traits::uart::Write as _;
use embassy_nrf::{peripherals::UARTE0, uarte};

pub struct UartLogger<T: uarte::Instance> {
    uart: uarte::Uarte<'static, T>,
}

impl<T: uarte::Instance> UartLogger<T> {
    pub fn new(uart: uarte::Uarte<'static, T>) -> Self {
        Self { uart }
    }
}

pub static LOGGER: ActorContext<UartLogger<UARTE0>, 4> = ActorContext::new();

impl<T: uarte::Instance> Actor for UartLogger<T> {
    type Message<'m> = heapless::String<64>;

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
                if let Some(mut m) = inbox.next().await {
                    let m = m.message();
                    let _ = self.uart.write(m.as_bytes()).await;
                }
            }
        }
    }
}

pub fn init(s: embassy::executor::Spawner, uart: uarte::Uarte<'static, UARTE0>) {
    LOGGER.mount(s, UartLogger::new(uart));
}

#[macro_export]
macro_rules! print {
    ($s:literal $(, $x:expr)* $(,)?) => {
        let mut b = heapless::String::new();
        use core::fmt::Write;
        let _ = write!(b, $s $(, $x)*);
        let _ = write!(b, "\r\n");
        let _ = logger::LOGGER.address().request(b).unwrap().await;
    };
}
