use crate::accel::*;
use crate::analog::*;
use crate::app::*;
use crate::control::*;
use crate::counter::*;
use crate::led::*;
use cfg_if::cfg_if;
use drogue_device::{
    actors::button::{Button, ButtonEventDispatcher},
    actors::dfu::*,
    actors::flash::*,
    actors::led::Led,
    ActorContext, Address,
};
use embassy::executor::Spawner;
use embassy::util::Forever;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::Peripherals;
use nrf_softdevice::Flash;

pub type RedLed = Led<Output<'static, AnyPin>>;
pub type GreenLed = Led<Output<'static, AnyPin>>;
pub type BlueLed = Led<Output<'static, AnyPin>>;
pub type YellowLed = Led<Output<'static, AnyPin>>;

pub type ButtonA = Input<'static, AnyPin>;
pub type ButtonB = Input<'static, AnyPin>;

pub struct BurrBoard {
    accel: ActorContext<Accelerometer>,
    analog: ActorContext<AnalogSensors>,

    red: ActorContext<RedLed>,
    green: ActorContext<GreenLed>,
    blue: ActorContext<BlueLed>,
    yellow: ActorContext<YellowLed>,

    counter_a: ActorContext<Counter>,
    button_a: ActorContext<Button<ButtonA, ButtonEventDispatcher<Counter>>>,

    counter_b: ActorContext<Counter>,
    button_b: ActorContext<Button<ButtonB, ButtonEventDispatcher<Counter>>>,

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

    pub accel: Address<Accelerometer>,

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
        cfg_if! {
            if #[cfg(feature = "rev2")] {
                let red_led_pin = p.P0_06.degrade();
                let green_led_pin = p.P0_30.degrade();
                let blue_led_pin = p.P0_28.degrade();
                let yellow_led_pin = p.P0_02.degrade();

                let button_a_pin = p.P0_27.degrade();
                let button_b_pin = p.P0_26.degrade();

                let temp_pin = p.P0_05;
                let light_pin = p.P0_03;
                let batt_pin = p.P0_04;
            } else if #[cfg(feature = "rev3")] {
                static TEMP_EN: Forever<Output<'static, AnyPin>> = Forever::new();
                static LIGHT_EN: Forever<Output<'static, AnyPin>> = Forever::new();
                TEMP_EN.put(Output::new(
                    p.P1_08.degrade(),
                    Level::High,
                    OutputDrive::Standard,
                ));
                LIGHT_EN.put(Output::new(
                    p.P0_07.degrade(),
                    Level::High,
                    OutputDrive::Standard,
                ));

                let red_led_pin = p.P0_30.degrade();
                let green_led_pin = p.P0_28.degrade();
                let blue_led_pin = p.P0_02.degrade();
                let yellow_led_pin = p.P0_26.degrade();

                let button_a_pin = p.P0_06.degrade();
                let button_b_pin = p.P0_27.degrade();

                let temp_pin = p.P0_05;
                let light_pin = p.P0_03;
                let batt_pin = p.P0_04;
            } else if #[cfg(feature = "rev3.5")] {
                static EN: Forever<Output<'static, AnyPin>> = Forever::new();
                EN.put(Output::new(
                    p.P1_08.degrade(),
                    Level::High,
                    OutputDrive::Standard,
                ));

                // Needed to ensure accelerometer can be initialized
                embassy::time::block_for(embassy::time::Duration::from_millis(500));

                let red_led_pin = p.P0_30.degrade();
                let green_led_pin = p.P0_28.degrade();
                let blue_led_pin = p.P0_02.degrade();
                let yellow_led_pin = p.P0_27.degrade();

                let button_a_pin = p.P0_08.degrade();
                let button_b_pin = p.P0_06.degrade();

                let temp_pin = p.P0_05;
                let light_pin = p.P0_03;
                let batt_pin = p.P0_04;
            }
        }

        let accel = self.accel.mount(
            s,
            Accelerometer::new(p.TWISPI0, p.P0_12, p.P0_11)
                .ok()
                .unwrap(),
        );

        // Actor for all analog sensors
        let analog = self.analog.mount(
            s,
            AnalogSensors::new(p.SAADC, temp_pin, light_pin, batt_pin),
        );

        // LEDs
        let red = self.red.mount(
            s,
            RedLed::new(Output::new(red_led_pin, Level::Low, OutputDrive::Standard)),
        );
        let green = self.green.mount(
            s,
            GreenLed::new(Output::new(
                green_led_pin,
                Level::Low,
                OutputDrive::Standard,
            )),
        );
        let blue = self.blue.mount(
            s,
            BlueLed::new(Output::new(blue_led_pin, Level::Low, OutputDrive::Standard)),
        );

        let yellow = self.yellow.mount(
            s,
            YellowLed::new(Output::new(
                yellow_led_pin,
                Level::Low,
                OutputDrive::Standard,
            )),
        );

        // Actor for button A and press counter
        let counter_a = self.counter_a.mount(s, Counter::new());
        self.button_a.mount(
            s,
            Button::new(Input::new(button_a_pin, Pull::None), counter_a.into()),
        );

        // Actor for button B and press counter
        let counter_b = self.counter_b.mount(s, Counter::new());
        self.button_b.mount(
            s,
            Button::new(Input::new(button_b_pin, Pull::None), counter_b.into()),
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
