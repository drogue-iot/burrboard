use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
use embassy_nrf::{
    interrupt,
    peripherals::{P0_11, P0_12, TWISPI0},
    twim,
};

use lsm6ds33::Lsm6ds33;

pub struct Accelerometer {
    lsm: Lsm6ds33<twim::Twim<'static, TWISPI0>>,
}

impl Accelerometer {
    pub fn new(twi: TWISPI0, sda: P0_12, scl: P0_11) -> Self {
        let mut config = twim::Config::default();
        config.frequency = twim::Frequency::K100;
        let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
        let i2c = twim::Twim::new(twi, irq, sda, scl, config);

        //let mut adxl = adxl343::Adxl343::new(i2c).unwrap();
        let mut lsm = lsm6ds33::Lsm6ds33::new(i2c, 0x6A).unwrap();
        lsm.set_accelerometer_output(lsm6ds33::AccelerometerOutput::Rate13)
            .unwrap();
        lsm.set_accelerometer_scale(lsm6ds33::AccelerometerScale::G04)
            .unwrap();
        Self { lsm }
    }
}

impl Actor for Accelerometer {
    type OnMountFuture<'m, M>
    where
        Self: 'm,
        M: 'm,
    = impl Future<Output = ()> + 'm;
    fn on_mount<'m, M>(&'m mut self, _: Address<Self>, _: &'m mut M) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self> + 'm,
        Self: 'm,
    {
        async move {
            loop {
                if let Ok((x, y, z)) = self.lsm.read_accelerometer() {
                    info!("Result: x: {}, y: {}, z: {}", x, y, z);
                }
            }
        }
    }
}
