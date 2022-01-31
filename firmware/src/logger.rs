use embassy::util::Forever;

use embassy_nrf::{peripherals::UARTE0, uarte};
use log::{Level, Log, Metadata, Record};

pub struct UartLogger<T: uarte::Instance> {
    uart: uarte::Uarte<'static, T>,
}

impl<T: uarte::Instance> UartLogger<T> {
    pub fn new(uart: uarte::Uarte<'static, T>) -> Self {
        Self { uart }
    }
}

pub static LOGGER: Forever<UartLogger<UARTE0>> = Forever::new();

pub fn init(uart: uarte::Uarte<'static, UARTE0>) {
    LOGGER.put(UartLogger::new(uart));
    log::set_max_level(log::LevelFilter::Debug);
    log::set_logger(unsafe { LOGGER.steal() }).unwrap();
}

impl Log for UartLogger<UARTE0> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut b: heapless::String<256> = heapless::String::new();
            use core::fmt::Write;
            let _ = write!(b, "{}\r\n", record.args());
            let _ = unsafe { LOGGER.steal() }
                .uart
                .blocking_write(b.as_bytes())
                .unwrap();
        }
    }

    fn flush(&self) {}
}
