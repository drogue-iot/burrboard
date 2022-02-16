use drogue_device::{
    actors::button::{Button, ButtonPressed},
    actors::dfu::*,
    actors::flash::*,
    actors::led::Led,
    traits::led::Led as _,
    ActorContext, Address,
};
use embassy::executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Input, Level, NoPin, Output, OutputDrive, Pin, Pull};
use embassy_nrf::peripherals::{P0_02, P0_03, P0_04, P0_05, P0_06, P0_26, P0_27, P0_28, P0_30};
use nrf_softdevice::{raw, Flash, Softdevice};

use crate::accel::*;
use crate::analog::*;
use crate::board::*;
use crate::counter::*;
use crate::gatt::*;
use crate::mesh::*;

pub enum App {
    Gatt(GattApp),
    Mesh(MeshApp),
}

impl App {
    pub fn enable(s: Spawner) -> App {
        // Read mode
        let p = unsafe { embassy_nrf::pac::Peripherals::steal() };
        let val = p.POWER.gpregret.read().bits();
        let (sd, app) = if val & 0x1 == 1 {
            info!("Running in GATT mode");
            let (sd, app) = GattApp::enable();
            let app = Self::Gatt(app);
            (sd, app)
        } else {
            info!("Running in MESH mode");
            let (sd, app) = MeshApp::enable();
            let app = Self::Mesh(app);
            (sd, app)
        };
        s.spawn(softdevice_task(sd)).unwrap();
        app
    }

    pub fn flash(&self) -> Flash {
        match self {
            Self::Gatt(app) => app.flash(),
            Self::Mesh(app) => app.flash(),
        }
    }

    pub fn mount(&'static self, s: Spawner, p: BoardPeripherals) {
        match self {
            Self::Gatt(app) => app.mount(s, p),
            Self::Mesh(app) => app.mount(s, p),
        }
    }

    pub fn switch(&'static self) -> ! {
        let p = unsafe { embassy_nrf::pac::Peripherals::steal() };
        unsafe {
            match self {
                Self::Gatt(_) => raw::sd_power_gpregret_clr(0, 0x1),
                Self::Mesh(_) => raw::sd_power_gpregret_set(0, 0x1),
            };
        }
        cortex_m::peripheral::SCB::sys_reset();
    }
}

#[embassy::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}
