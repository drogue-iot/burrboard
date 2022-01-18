#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use panic_probe as _;

use adxl343::accelerometer::RawAccelerometer;
use embassy::time::{Duration, Timer};
use embassy_nrf::config::Config;
use embassy_nrf::gpio::NoPin;
use embassy_nrf::interrupt::Priority;
use embassy_nrf::saadc;
use embassy_nrf::twim;
use embassy_nrf::uarte;
use embassy_nrf::{interrupt, Peripherals};

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
    config.scl_pullup = true;
    config.sda_pullup = true;
    config.frequency = twim::Frequency::K100;
    let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    let i2c = twim::Twim::new(p.TWISPI0, irq, p.P0_12, p.P0_11, config);

    // Ensure accel is ready
    Timer::after(Duration::from_millis(100)).await;
    let mut adxl = adxl343::Adxl343::new(i2c).unwrap();

    let config = saadc::Config::default();
    let temp_channel = saadc::ChannelConfig::single_ended(&mut p.P0_05);
    let light_channel = saadc::ChannelConfig::single_ended(&mut p.P0_03);
    let bat_channel = saadc::ChannelConfig::single_ended(&mut p.P0_04);
    let mut saadc = saadc::Saadc::new(
        p.SAADC,
        interrupt::take!(SAADC),
        config,
        [temp_channel, light_channel, bat_channel],
    );

    loop {
        let mut buf = [0; 3];
        saadc.sample(&mut buf).await;

        info!("temp sample: {}", &buf[0]);
        info!("light sample: {}", &buf[1]);
        info!("bat sample: {}", &buf[2]);

        let voltage = buf[0] as f32 * 3.3;
        let voltage = voltage / 4095 as f32;
        info!("Voltage: {}", voltage);
        let tempc = (voltage - 0.5) * 100.0;
        info!("Temperature: {}", tempc);

        let accel = adxl.accel_raw().unwrap();
        info!("Accel (X, Y, Z): ({}, {}, {})", accel.x, accel.y, accel.z);

        Timer::after(Duration::from_millis(1000)).await;
    }
}
