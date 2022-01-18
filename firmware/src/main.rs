#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use panic_probe as _;

use adxl343::accelerometer::RawAccelerometer;
use embassy::time::{Duration, Timer};
use embassy::traits::i2c::I2c;
use embassy::traits::spi::FullDuplex;
use embassy_nrf::gpio::{Level, NoPin, Output, OutputDrive};
use embassy_nrf::saadc::{ChannelConfig, Config, Saadc};
use embassy_nrf::twim;
use embassy_nrf::uarte;
use embassy_nrf::{interrupt, Peripherals};

#[macro_use]
mod logger;

mod fmt;

#[embassy::main]
async fn main(spawner: embassy::executor::Spawner, mut p: Peripherals) {
    /*let cs = Output::new(p.P0_12, Level::High, OutputDrive::Standard);
    Timer::after(Duration::from_millis(1000)).await;

    let irq = interrupt::take!(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::K125;
    config.mode = spim::MODE_3;
    let spim = spim::Spim::new(p.TWISPI1, irq, p.P0_17, p.P0_01, p.P0_13, config);
    defmt::info!("Initializing");
    defmt::info!("Done");
    */
    let mut config = twim::Config::default();
    config.scl_pullup = true;
    config.sda_pullup = true;
    config.frequency = twim::Frequency::K100;
    let irq = interrupt::take!(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
    let i2c = twim::Twim::new(p.TWISPI1, irq, p.P0_12, p.P0_11, config);
    let mut adxl = adxl343::Adxl343::new(i2c).unwrap();

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

    let config = Config::default();
    let temp_channel = ChannelConfig::single_ended(&mut p.P0_05);
    let light_channel = ChannelConfig::single_ended(&mut p.P0_03);
    let bat_channel = ChannelConfig::single_ended(&mut p.P0_04);
    let mut saadc = Saadc::new(
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

        /*
        let accel = adxl.accel_raw().unwrap();
        defmt::info!("Accel (X, Y, Z): ({}, {}, {})", accel.x, accel.y, accel.z);
        */

        Timer::after(Duration::from_millis(1000)).await;
    }
}
