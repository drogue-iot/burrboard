use cfg_if::cfg_if;
use core::future::Future;
use drogue_device::{Actor, Address, Inbox, Request};
use embassy_nrf::{
    interrupt,
    peripherals::{P0_11, P0_12, TWISPI0},
    twim,
};

#[cfg(feature = "adxl")]
use adxl343::Adxl343;

#[cfg(feature = "lsm")]
use lsm6ds33::Lsm6ds33;

pub struct Accelerometer {
    #[cfg(feature = "lsm")]
    lsm: Lsm6ds33<twim::Twim<'static, TWISPI0>>,
    #[cfg(feature = "adxl")]
    adxl: Adxl343<twim::Twim<'static, TWISPI0>>,
}

pub enum AccelError {
    Init,
}

impl Accelerometer {
    pub fn new(twi: TWISPI0, sda: P0_12, scl: P0_11) -> Result<Self, AccelError> {
        let mut config = twim::Config::default();
        config.frequency = twim::Frequency::K100;
        let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
        let _i2c = twim::Twim::new(twi, irq, sda, scl, config);

        #[cfg(feature = "adxl")]
        {
            // Data rate 100Hz by default
            let adxl = adxl343::Adxl343::new(_i2c).map_err(|_| AccelError::Init)?;
            return Ok(Self { adxl });
        }

        #[cfg(feature = "lsm")]
        {
            let mut lsm = lsm6ds33::Lsm6ds33::new(_i2c, 0x6A).map_err(|_| AccelError::Init)?;
            lsm.set_accelerometer_output(lsm6ds33::AccelerometerOutput::Rate13)
                .map_err(|_| AccelError::Init)?;
            lsm.set_accelerometer_scale(lsm6ds33::AccelerometerScale::G04)
                .map_err(|_| AccelError::Init)?;
            return Ok(Self { lsm });
        }

        #[cfg(not(any(feature = "lsm", feature = "adxl")))]
        Ok(Self {})
    }
}

pub struct AccelRead;

#[derive(Clone, Copy)]
pub struct AccelValues {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub type AccelRequest = Request<AccelRead, AccelValues>;

impl Actor for Accelerometer {
    type Message<'m> = Request<AccelRead, AccelValues>;
    type OnMountFuture<'m, M> = impl Future<Output = ()> + 'm
    where
        Self: 'm,
        M: 'm + Inbox<Self::Message<'m>>;
    fn on_mount<'m, M>(
        &'m mut self,
        _: Address<Self::Message<'m>>,
        mut inbox: M,
    ) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self::Message<'m>> + 'm,
        Self: 'm,
    {
        async move {
            loop {
                let m = inbox.next().await;
                let values = {
                    cfg_if! {
                        if #[cfg(feature = "lsm")] {
                            if let Ok((x, y, z)) = self.lsm.read_accelerometer() {
                                trace!("Accel: x: {}, y: {}, z: {}", x, y, z);
                                Some(AccelValues { x, y, z })
                            } else {
                                None
                            }
                        } else if #[cfg(feature = "adxl")] {
                            use adxl343::accelerometer::Accelerometer;
                            if let Ok(val) = self.adxl.accel_norm() {
                                let x = val.x;
                                let y = val.y;
                                let z = val.z;
                                trace!("Accel: x: {}, y: {}, z: {}", x, y, z);
                                Some(AccelValues { x, y, z })
                            } else {
                                None
                            }
                        } else {
                            Some(AccelValues {x: 0.0, y: 0.0, z: 0.0})
                        }
                    }
                };
                if let Some(values) = values {
                    m.reply(values).await;
                }
            }
        }
    }
}
