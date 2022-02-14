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

pub enum AccelError {
    Init,
}

impl Accelerometer {
    pub fn new(twi: TWISPI0, sda: P0_12, scl: P0_11) -> Result<Self, AccelError> {
        let mut config = twim::Config::default();
        config.frequency = twim::Frequency::K100;
        let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
        let i2c = twim::Twim::new(twi, irq, sda, scl, config);

        //let mut adxl = adxl343::Adxl343::new(i2c).unwrap();
        let mut lsm = lsm6ds33::Lsm6ds33::new(i2c, 0x6A).map_err(|_| AccelError::Init)?;
        lsm.set_accelerometer_output(lsm6ds33::AccelerometerOutput::Rate13)
            .map_err(|_| AccelError::Init)?;
        lsm.set_accelerometer_scale(lsm6ds33::AccelerometerScale::G04)
            .map_err(|_| AccelError::Init)?;
        Ok(Self { lsm })
    }
}

pub struct Read;

#[derive(Clone, Copy)]
pub struct AccelValues {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Actor for Accelerometer {
    type Message<'m> = Read;
    type Response = Option<AccelValues>;
    type OnMountFuture<'m, M>
    where
        Self: 'm,
        M: 'm,
    = impl Future<Output = ()> + 'm;
    fn on_mount<'m, M>(
        &'m mut self,
        _: Address<Self>,
        inbox: &'m mut M,
    ) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self> + 'm,
        Self: 'm,
    {
        async move {
            loop {
                if let Some(mut m) = inbox.next().await {
                    let response = if let Ok((x, y, z)) = self.lsm.read_accelerometer() {
                        info!("Accel: x: {}, y: {}, z: {}", x, y, z);
                        let x = (x * i16::MAX as f32) as i16;
                        let y = (y * i16::MAX as f32) as i16;
                        let z = (z * i16::MAX as f32) as i16;
                        Some(AccelValues { x, y, z })
                    } else {
                        None
                    };
                    m.set_response(response);
                }
            }
        }
    }
}
