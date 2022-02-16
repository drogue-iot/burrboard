#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![allow(unused)]

use drogue_device::{
    actors::button::{Button, ButtonPressed},
    actors::dfu::*,
    actors::flash::*,
    actors::led::Led,
    traits::led::Led as _,
    ActorContext, Address,
};
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_nrf::config::Config;
use embassy_nrf::gpio::{AnyPin, Input, Level, NoPin, Output, OutputDrive, Pin, Pull};
use embassy_nrf::interrupt::Priority;
use embassy_nrf::peripherals::{P0_02, P0_03, P0_04, P0_05, P0_06, P0_26, P0_27, P0_28, P0_30};
use embassy_nrf::uarte;
use embassy_nrf::{interrupt, Peripherals};
use nrf_softdevice::ble::{gatt_server, peripheral};
use nrf_softdevice::raw;
use nrf_softdevice::{Flash, Softdevice};

mod fmt;

#[cfg(feature = "panic-probe")]
use panic_probe as _;

#[cfg(feature = "defmt-rtt")]
use defmt_rtt as _;

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
mod mesh;
mod watchdog;

use accel::*;
use analog::*;
use app::*;
use board::*;
use counter::*;
use watchdog::*;

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
    let app = APP.put(App::enable(s));

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
    app.mount(s, ap.clone());

    // Launch watchdog
    static WATCHDOG: ActorContext<Watchdog> = ActorContext::new();
    WATCHDOG.mount(s, Watchdog(Duration::from_secs(2)));

    // Bootup animation signalling that everything is started
    let mut red = ap.leds.red;
    let mut green = ap.leds.green;
    let mut blue = ap.leds.blue;
    let mut yellow = ap.leds.yellow;

    red.on();
    Timer::after(Duration::from_secs(1)).await;
    green.on();
    Timer::after(Duration::from_secs(1)).await;
    blue.on();
    Timer::after(Duration::from_secs(1)).await;
    yellow.on();
    Timer::after(Duration::from_secs(1)).await;
    red.off();
    green.off();
    blue.off();
    yellow.off();

    info!("Application started");
}

#[allow(unused)]
#[allow(unused_variables)]
pub fn log_stack(file: &'static str) {
    let _u: u32 = 1;
    let _uptr: *const u32 = &_u;
    info!("[{}] SP: 0x{:?}", file, &_uptr);
}
