#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use drogue_device::{
    actors::{
        button::{Button, ButtonPressed},
        led::Led,
    },
    drivers::button::Button as ButtonDriver,
    drivers::led::Led as LedDriver,
    ActorContext,
};
use panic_probe as _;

use embassy::time::{Duration, Timer};
use embassy_nrf::config::Config;
use embassy_nrf::gpio::{Input, Level, NoPin, Output, OutputDrive, Pull};
use embassy_nrf::interrupt::Priority;
use embassy_nrf::peripherals::{P0_02, P0_03, P0_04, P0_05, P0_06, P0_26, P0_27, P0_28, P0_30};
use embassy_nrf::uarte;
use embassy_nrf::{interrupt, Peripherals};
use nrf_softdevice::Softdevice;

#[macro_use]
mod logger;

mod fmt;

mod accel;
mod analog;
mod counter;
mod dfu;
mod flash;

use accel::*;
use analog::*;
use counter::*;
use dfu::*;
use flash::*;

pub type RedLed = LedDriver<Output<'static, P0_06>>;
pub type GreenLed = LedDriver<Output<'static, P0_30>>;
pub type BlueLed = LedDriver<Output<'static, P0_28>>;
pub type YellowLed = LedDriver<Output<'static, P0_02>>;

pub type ButtonA = ButtonDriver<Input<'static, P0_27>>;
pub type ButtonB = ButtonDriver<Input<'static, P0_26>>;

pub type BatteryPin = P0_04;
pub type TemperaturePin = P0_05;
pub type LightPin = P0_03;

// Application must run at a lower priority than softdevice
fn config() -> Config {
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    config
}

#[embassy::main(config = "config()")]
async fn main(s: embassy::executor::Spawner, p: Peripherals) {
    logger::init(
        s,
        uarte::Uarte::new(
            p.UARTE0,
            interrupt::take!(UARTE0_UART0),
            p.P0_24,
            p.P0_25,
            NoPin,
            NoPin,
            Default::default(),
        ),
    );

    let sd = Softdevice::enable(&Default::default());

    // Ensure accel is ready
    Timer::after(Duration::from_millis(500)).await;

    // Actor for accelerometer
    static ACCEL: ActorContext<Accelerometer> = ActorContext::new();
    let _accel = ACCEL.mount(s, Accelerometer::new(p.TWISPI0, p.P0_12, p.P0_11));

    // Actor for all analog sensors
    static ANALOG: ActorContext<AnalogSensors> = ActorContext::new();
    let _analog = ANALOG.mount(s, AnalogSensors::new(p.SAADC, p.P0_05, p.P0_03, p.P0_04));

    // Actor for red LED
    static RED: ActorContext<Led<RedLed>> = ActorContext::new();
    RED.mount(
        s,
        Led::new(RedLed::new(Output::new(
            p.P0_06,
            Level::Low,
            OutputDrive::Standard,
        ))),
    );

    // Actor for green LED
    static GREEN: ActorContext<Led<GreenLed>> = ActorContext::new();
    GREEN.mount(
        s,
        Led::new(GreenLed::new(Output::new(
            p.P0_30,
            Level::Low,
            OutputDrive::Standard,
        ))),
    );

    // Actor for blue LED
    static BLUE: ActorContext<Led<BlueLed>> = ActorContext::new();
    BLUE.mount(
        s,
        Led::new(BlueLed::new(Output::new(
            p.P0_28,
            Level::Low,
            OutputDrive::Standard,
        ))),
    );

    // Actor for yellow LED
    static YELLOW: ActorContext<Led<YellowLed>> = ActorContext::new();
    YELLOW.mount(
        s,
        Led::new(YellowLed::new(Output::new(
            p.P0_02,
            Level::Low,
            OutputDrive::Standard,
        ))),
    );

    // Actor for button A and press counter
    static COUNTER_A: ActorContext<Counter> = ActorContext::new();
    static BUTTON_A: ActorContext<
        Button<ButtonDriver<Input<'static, P0_27>>, ButtonPressed<Counter>>,
    > = ActorContext::new();
    BUTTON_A.mount(
        s,
        Button::new(
            ButtonDriver::new(Input::new(p.P0_27, Pull::None)),
            ButtonPressed(COUNTER_A.mount(s, Counter::new()), Increment),
        ),
    );

    // Actor for button B and press counter
    static COUNTER_B: ActorContext<Counter> = ActorContext::new();
    static BUTTON_B: ActorContext<
        Button<ButtonDriver<Input<'static, P0_26>>, ButtonPressed<Counter>>,
    > = ActorContext::new();
    BUTTON_B.mount(
        s,
        Button::new(
            ButtonDriver::new(Input::new(p.P0_26, Pull::None)),
            ButtonPressed(COUNTER_B.mount(s, Counter::new()), Increment),
        ),
    );

    // Actor for Flash
    static FLASH: ActorContext<SharedFlash> = ActorContext::new();
    let flash = FLASH.mount(s, SharedFlash::new(sd));

    // Actor for DFU
    static DFU: ActorContext<FirmwareManager<SharedFlashHandle>> = ActorContext::new();
    DFU.mount(s, FirmwareManager::new(SharedFlashHandle(flash)));
}
