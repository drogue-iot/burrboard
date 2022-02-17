use crate::accel::*;
use crate::analog::*;
use crate::app::*;
use crate::control::*;
use crate::counter::*;
use crate::led::*;
use drogue_device::{
    actors::button::{Button, ButtonPressed},
    actors::dfu::*,
    actors::flash::*,
    actors::led::Led,
    ActorContext, Address,
};
use embassy::executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::peripherals::{P0_02, P0_06, P0_26, P0_27, P0_28, P0_30};
use embassy_nrf::Peripherals;
use nrf_softdevice::Flash;

pub type RedLed = Led<Output<'static, P0_06>>;
pub type GreenLed = Led<Output<'static, P0_30>>;
pub type BlueLed = Led<Output<'static, P0_28>>;
pub type YellowLed = Led<Output<'static, P0_02>>;

pub type ButtonA = Input<'static, P0_27>;
pub type ButtonB = Input<'static, P0_26>;

pub struct BurrBoard {
    accel: ActorContext<Accelerometer>,
    analog: ActorContext<AnalogSensors>,

    red: ActorContext<RedLed>,
    green: ActorContext<GreenLed>,
    blue: ActorContext<BlueLed>,
    yellow: ActorContext<YellowLed>,

    counter_a: ActorContext<Counter>,
    button_a: ActorContext<Button<ButtonA, ButtonPressed<Counter>>>,

    counter_b: ActorContext<Counter>,
    button_b: ActorContext<Button<ButtonB, ButtonPressed<Counter>>>,

    flash: ActorContext<SharedFlash<Flash>>,
    dfu: ActorContext<FirmwareManager<SharedFlashHandle<Flash>>>,

    control: ActorContext<ControlButton>,
}

#[derive(Clone)]
pub struct BoardPeripherals {
    pub leds: Leds,

    pub counter_a: Address<Counter>,
    pub counter_b: Address<Counter>,

    pub analog: Address<AnalogSensors>,

    pub accel: Option<Address<Accelerometer>>,

    pub flash: Address<SharedFlash<Flash>>,

    pub dfu: Address<FirmwareManager<SharedFlashHandle<Flash>>>,
}

#[derive(Clone)]
pub struct Leds {
    pub red: StatefulLed<RedLed>,
    pub green: StatefulLed<GreenLed>,
    pub blue: StatefulLed<BlueLed>,
    pub yellow: StatefulLed<YellowLed>,
}

impl BurrBoard {
    pub const fn new() -> Self {
        Self {
            accel: ActorContext::new(),
            analog: ActorContext::new(),

            red: ActorContext::new(),
            green: ActorContext::new(),
            blue: ActorContext::new(),
            yellow: ActorContext::new(),

            counter_a: ActorContext::new(),
            button_a: ActorContext::new(),

            counter_b: ActorContext::new(),
            button_b: ActorContext::new(),

            flash: ActorContext::new(),
            dfu: ActorContext::new(),

            control: ActorContext::new(),
        }
    }

    pub fn mount(&'static self, s: Spawner, app: &'static App, p: Peripherals) -> BoardPeripherals {
        #[cfg(feature = "lsm")]
        let accel: Option<Address<Accelerometer>> =
            if let Ok(accel) = Accelerometer::new(p.TWISPI0, p.P0_12, p.P0_11) {
                Some(self.accel.mount(s, accel))
            } else {
                None
            };
        #[cfg(not(feature = "lsm"))]
        let accel: Option<Address<Accelerometer>> = None;

        // Actor for all analog sensors
        let analog = self
            .analog
            .mount(s, AnalogSensors::new(p.SAADC, p.P0_05, p.P0_03, p.P0_04));

        // LEDs
        let red = self.red.mount(
            s,
            RedLed::new(Output::new(p.P0_06, Level::Low, OutputDrive::Standard)),
        );
        let green = self.green.mount(
            s,
            GreenLed::new(Output::new(p.P0_30, Level::Low, OutputDrive::Standard)),
        );
        let blue = self.blue.mount(
            s,
            BlueLed::new(Output::new(p.P0_28, Level::Low, OutputDrive::Standard)),
        );

        let yellow = self.yellow.mount(
            s,
            YellowLed::new(Output::new(p.P0_02, Level::Low, OutputDrive::Standard)),
        );

        // Actor for button A and press counter
        let counter_a = self.counter_a.mount(s, Counter::new());
        self.button_a.mount(
            s,
            Button::new(
                Input::new(p.P0_27, Pull::None),
                ButtonPressed(counter_a, CounterMessage::Increment),
            ),
        );

        // Actor for button B and press counter
        let counter_b = self.counter_b.mount(s, Counter::new());
        self.button_b.mount(
            s,
            Button::new(
                Input::new(p.P0_26, Pull::None),
                ButtonPressed(counter_b, CounterMessage::Increment),
            ),
        );

        // Actor for shared access to flash
        let flash = self.flash.mount(s, SharedFlash::new(app.flash()));

        // Actor for DFU
        let dfu = self.dfu.mount(
            s,
            FirmwareManager::new(flash.into(), embassy_boot_nrf::updater::new()),
        );

        self.control.mount(
            s,
            ControlButton::new(app, Input::new(p.P1_02.degrade(), Pull::Up)),
        );

        BoardPeripherals {
            leds: Leds {
                red: StatefulLed::new(red, false),
                green: StatefulLed::new(green, false),
                blue: StatefulLed::new(blue, false),
                yellow: StatefulLed::new(yellow, false),
            },
            counter_a,
            counter_b,

            analog,
            accel,
            flash,
            dfu,
        }
    }
}
