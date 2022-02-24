#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use drogue_device::ActorContext;
use embassy::time::Duration;
use embassy::util::Forever;
use embassy_nrf::config::Config;
use embassy_nrf::interrupt::Priority;
use embassy_nrf::Peripherals;

#[cfg(feature = "log")]
use embassy_nrf::{gpio::NoPin, interrupt, uarte};

mod fmt;

#[cfg(feature = "panic-probe")]
use panic_probe as _;

#[cfg(feature = "nrf-softdevice-defmt-rtt")]
use nrf_softdevice_defmt_rtt as _;

#[cfg(feature = "log")]
mod logger;

#[cfg(not(feature = "defmt"))]
use panic_reset as _;

mod accel;
mod analog;
mod app;
mod board;
mod control;
mod counter;
mod gatt;
mod led;
mod mesh;
mod watchdog;

use app::*;
use board::*;
use watchdog::*;

#[cfg(not(any(feature = "rev2", feature = "rev3", feature = "rev3.5",)))]
compile_error!("No board revision selected. You must activate exactly one of the following: rev2, rev3 or rev3.5");

const FIRMWARE_VERSION: &str = env!("CARGO_PKG_VERSION");
const FIRMWARE_REVISION: Option<&str> = option_env!("REVISION");

// Application must run at a lower priority than softdevice
fn config() -> Config {
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    config
}

#[embassy::main(config = "config()")]
async fn main(s: embassy::executor::Spawner, mut p: Peripherals) {
    // First enable application
    static APP: Forever<App> = Forever::new();
    let app = APP.put(App::enable(s, "BurrBoard"));

    #[cfg(feature = "log")]
    {
        logger::init(uarte::Uarte::new(
            p.UARTE0,
            interrupt::take!(UARTE0_UART0),
            p.P0_24,
            p.P0_25,
            NoPin,
            NoPin,
            Default::default(),
        ));
    }

    // Setup burrboard peripherals
    static BOARD: BurrBoard = BurrBoard::new();
    let mut ap = BOARD.mount(s, app, p);

    // Launch the selected application
    app.mount(s, &ap);

    // Launch watchdog
    static WATCHDOG: ActorContext<Watchdog> = ActorContext::new();
    WATCHDOG.mount(s, Watchdog(Duration::from_secs(2)));

    // Bootup animation signalling that everything is started
    app.post(&mut ap.leds).await;

    info!("Application started");
}

#[allow(unused)]
#[allow(unused_variables)]
pub fn log_stack(file: &'static str) {
    let _u: u32 = 1;
    let _uptr: *const u32 = &_u;
    info!("[{}] SP: 0x{:?}", file, &_uptr);
}
