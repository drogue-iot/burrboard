#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use drogue_device::{
    actors::{
        self,
        button::{Button, ButtonEvent, ButtonEventHandler},
    },
    drivers, ActorContext,
};
use panic_probe as _;

use adxl343::accelerometer::Accelerometer;
use adxl343::accelerometer::RawAccelerometer;
use embassy::time::{Duration, Timer};
use embassy::traits::gpio::WaitForLow;
use embassy_nrf::config::Config;
use embassy_nrf::gpio::{AnyPin, Input, Level, NoPin, Output, OutputDrive, Pin, Pull};
use embassy_nrf::interrupt::Priority;
use embassy_nrf::peripherals::P1_02;
use embassy_nrf::saadc;
use embassy_nrf::twim;
use embassy_nrf::uarte;
use embassy_nrf::{interrupt, Peripherals};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use futures::future::{select, Either};

#[macro_use]
mod logger;

mod fmt;

// Application must run at a lower priority than softdevice
fn config() -> Config {
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    config
}

#[embassy::main(config = "config()")]
async fn main(spawner: embassy::executor::Spawner, mut p: Peripherals) {
    logger::init(
        spawner,
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

    let mut config = twim::Config::default();
    config.scl_pullup = false;
    config.sda_pullup = false;
    config.frequency = twim::Frequency::K100;
    let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    let i2c = twim::Twim::new(p.TWISPI0, irq, p.P0_12, p.P0_11, config);

    // Ensure accel is ready
    Timer::after(Duration::from_millis(500)).await;
    //let mut adxl = adxl343::Adxl343::new(i2c).unwrap();
    let mut lsm = lsm6ds33::Lsm6ds33::new(i2c, 0x6A).unwrap();
    lsm.set_accelerometer_output(lsm6ds33::AccelerometerOutput::Rate13)
        .unwrap();
    lsm.set_accelerometer_scale(lsm6ds33::AccelerometerScale::G04)
        .unwrap();

    let config = saadc::Config::default();
    let temp_channel = saadc::ChannelConfig::single_ended(&mut p.P0_05);
    let light_channel = saadc::ChannelConfig::single_ended(&mut p.P0_03);
    let mut bat_channel = saadc::ChannelConfig::single_ended(&mut p.P0_04);
    bat_channel.time = saadc::Time::_40US;
    bat_channel.gain = saadc::Gain::GAIN1_5;
    bat_channel.resistor = saadc::Resistor::BYPASS;
    let mut saadc = saadc::Saadc::new(
        p.SAADC,
        interrupt::take!(SAADC),
        config,
        [temp_channel, light_channel], //, bat_channel],
    );

    /*
    DFU_BUTTON.mount(
        spawner,
        Button::new(
            drivers::button::Button::new(Input::new(p.P1_02, Pull::Up)),
            DfuActivator,
        ),
    );

    */

    let mut button_a = Input::new(p.P0_27, Pull::None);
    let mut button_b = Input::new(p.P0_26, Pull::None);

    let mut leds: [Output<'static, AnyPin>; 4] = [
        Output::new(p.P0_06.degrade(), Level::Low, OutputDrive::Standard),
        Output::new(p.P0_30.degrade(), Level::Low, OutputDrive::Standard),
        Output::new(p.P0_28.degrade(), Level::Low, OutputDrive::Standard),
        Output::new(p.P0_02.degrade(), Level::Low, OutputDrive::Standard),
    ];
    let mut led_idx = 0;
    loop {
        let mut buf = [0; 2];
        saadc.sample(&mut buf).await;

        info!("temp sample: {}", &buf[0]);
        info!("light sample: {}", &buf[1]);

        let voltage = buf[0] as f32 * 3.3;
        let voltage = voltage / 4095 as f32;
        //info!("Voltage: {}", voltage);
        let tempc = (voltage - 0.5) * 100.0;
        //info!("Temperature: {}", tempc);

        /*
        info!("bat sample: {}", &buf[2]);
        let bat_voltage = buf[2] as f32 * 3 as f32;
        let bat_voltage = bat_voltage * 1.5 / 4064 as f32;
        info!("Bat voltage: {} V", bat_voltage);
        leds[led_idx].set_high();
        */
        //        let accel = adxl.accel_norm().unwrap();
        //       info!("Accel (X, Y, Z): ({}, {}, {})", accel.x, accel.y, accel.z);

        /*
        match select(button_a.wait_for_low(), button_b.wait_for_low()).await {
            Either::Left((_, _)) => {
                info!("Button 'A' pressed");
                leds[led_idx].set_low();
                if led_idx == 0 {
                    led_idx = 3;
                } else {
                    led_idx -= 1;
                }
            }
            Either::Right((_, _)) => {
                info!("Button 'B' pressed");
                leds[led_idx].set_low();
                if led_idx == 3 {
                    led_idx = 0;
                } else {
                    led_idx += 1;
                }
            }
        }*/
        let result = lsm.read_accelerometer().unwrap();
        info!("Result: x: {}, y: {}, z: {}", result.0, result.1, result.2);
        Timer::after(Duration::from_millis(1000)).await;
    }
}

pub struct DfuActivator;

impl ButtonEventHandler for DfuActivator {
    fn handle(&mut self, event: ButtonEvent) {
        use embassy_nrf::pac;
        if let ButtonEvent::Released = event {
            unsafe {
                let cp = pac::Peripherals::steal();
                // 0x57 - Regular bootloader
                // 0xA8 - OTA bootloader
                // 0x4E - Serial bootloader
                cp.POWER.gpregret.write(|w| w.bits(0xA8));
                pac::SCB::sys_reset();
            }
        }
    }
}

type UserButton = drivers::button::Button<Input<'static, P1_02>, drivers::ActiveLow>;
static DFU_BUTTON: ActorContext<Button<UserButton, DfuActivator>> = ActorContext::new();
