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
use embassy_nrf::spim;
use embassy_nrf::twim;
use embassy_nrf::uarte;
use embassy_nrf::{interrupt, Peripherals};
use embedded_hal::blocking::spi::Transfer;

#[embassy::main]
async fn main(spawner: embassy::executor::Spawner, mut p: Peripherals) {
    let mut config = twim::Config::default();
    config.scl_pullup = true;
    config.sda_pullup = true;
    config.frequency = twim::Frequency::K100;
    let irq = interrupt::take!(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
    let mut i2c = twim::Twim::new(p.TWISPI1, irq, p.P1_00, p.P0_26, config);

    Timer::after(Duration::from_millis(100)).await;
    defmt::info!("Initializing");
    let w: [u8; 1] = [0; 1];
    let mut buffer: [u8; 1] = [0; 1];
    const ADDRESS: u8 = 0x53;
    match i2c.write_read(ADDRESS, &w, &mut buffer).await {
        Ok(_) => {
            defmt::info!("Whoami: {}", buffer[0]);
        }
        Err(e) => {
            defmt::info!("Error i2c : {:?}", e);
        }
    }

    let config = Config::default();
    let temp_channel = ChannelConfig::single_ended(&mut p.P0_02);
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

        defmt::info!("temp sample: {}", &buf[0]);
        defmt::info!("light sample: {}", &buf[1]);
        defmt::info!("bat sample: {}", &buf[2]);

        let voltage = buf[0] as f32 * 3.3;
        let voltage = voltage / 4095 as f32;
        defmt::info!("Voltage: {}", voltage);
        let tempc = (voltage - 0.5) * 100.0;
        defmt::info!("Temperature: {}", tempc);

        //let accel = adxl.accel_raw().unwrap();
        //defmt::info!("Accel (X, Y, Z): ({}, {}, {})", accel.x, accel.y, accel.z);

        Timer::after(Duration::from_millis(1000)).await;
    }
}
