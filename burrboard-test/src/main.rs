#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use panic_probe as _;

use embassy::time::{Duration, Timer};
use embassy::traits::spi::FullDuplex;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::saadc::{ChannelConfig, Config, Saadc};
use embassy_nrf::spim;
use embassy_nrf::{interrupt, Peripherals};

mod register;
use register::*;

#[embassy::main]
async fn main(spawner: embassy::executor::Spawner, mut p: Peripherals) {
    let mut cs = Output::new(p.P0_09, Level::High, OutputDrive::Standard);
    let irq = interrupt::take!(SPIM3);
    let mut spim = spim::Spim::new(
        p.SPI3,
        irq,
        p.P0_17,
        p.P0_01,
        p.P0_13,
        spim::Config::default(),
    );

    let mut devid = [0u8; 2];

    cs.set_high();
    Timer::after(Duration::from_millis(1000)).await;
    spim.read_write(
        &mut devid,
        &[Register::DEVID.addr(), Register::DEVID.addr()],
    )
    .await;
    defmt::info!("Accel dev id: {:x}, {:x}", devid[0], devid[1]);
    cs.set_low();

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

        Timer::after(Duration::from_millis(1000)).await;
    }
}
