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
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::saadc::{ChannelConfig, Config, Saadc};
use embassy_nrf::spim;
use embassy_nrf::twim;
use embassy_nrf::{interrupt, Peripherals};
use embedded_hal::blocking::spi::Transfer;

#[embassy::main]
async fn main(spawner: embassy::executor::Spawner, mut p: Peripherals) {
    let cs = Output::new(p.P0_12, Level::High, OutputDrive::Standard);
    Timer::after(Duration::from_millis(1000)).await;

    let irq = interrupt::take!(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::K125;
    config.mode = spim::MODE_3;
    let spim = spim::Spim::new(p.TWISPI1, irq, p.P0_17, p.P0_01, p.P0_13, config);
    defmt::info!("Initializing");
    let mut adxl = adxl343::Adxl343::new(adxl343::SpiTransport::new(spim, cs)).unwrap();
    defmt::info!("Done");

    let config = Config::default();
    let temp_channel = ChannelConfig::single_ended(&mut p.P0_02);
    let light_channel = ChannelConfig::single_ended(&mut p.P0_03);
    let mut saadc = Saadc::new(
        p.SAADC,
        interrupt::take!(SAADC),
        config,
        [temp_channel, light_channel],
    );

    loop {
        let mut buf = [0; 2];
        saadc.sample(&mut buf).await;

        defmt::info!("temp sample: {=i16}", &buf[0]);
        defmt::info!("light sample: {=i16}", &buf[1]);

        let voltage = buf[0] as f32 * 3.3;
        let voltage = voltage / 4095 as f32;
        defmt::info!("Voltage: {}", voltage);
        let tempc = (voltage - 0.5) * 100.0;
        defmt::info!("Temperature: {}", tempc);

        let accel = adxl.accel_raw().unwrap();
        defmt::info!("Accel (X, Y, Z): ({}, {}, {})", accel.x, accel.y, accel.z);

        Timer::after(Duration::from_millis(1000)).await;
    }
}
