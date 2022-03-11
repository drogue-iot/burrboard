use drogue_device::traits::led::Led as _;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use nrf_softdevice::{raw, Flash, Softdevice};

use crate::board::*;
use crate::gatt::*;
use crate::mesh::*;

pub struct App {
    sd: &'static Softdevice,
    mode: Mode,
}

pub enum Mode {
    Gatt(GattApp),
    Mesh(MeshApp),
}

impl App {
    pub fn enable(s: Spawner, device_name: &'static str) -> App {
        let config = nrf_softdevice::Config {
            clock: Some(raw::nrf_clock_lf_cfg_t {
                source: raw::NRF_CLOCK_LF_SRC_RC as u8,
                rc_ctiv: 4,
                rc_temp_ctiv: 2,
                accuracy: 7,
            }),
            conn_gap: Some(raw::ble_gap_conn_cfg_t {
                conn_count: 2,
                event_length: 24,
            }),
            conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 30 }),
            gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
                adv_set_count: 1,
                periph_role_count: 1,
                central_role_count: 1,
                central_sec_count: 1,
                _bitfield_1: Default::default(),
            }),
            gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
                p_value: device_name.as_ptr() as *const u8 as _,
                current_len: device_name.len() as u16,
                max_len: device_name.len() as u16,
                write_perm: unsafe { core::mem::zeroed() },
                _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                    raw::BLE_GATTS_VLOC_STACK as u8,
                ),
            }),

            ..Default::default()
        };
        let sd = Softdevice::enable(&config);
        s.spawn(softdevice_task(sd)).unwrap();

        let p = unsafe { embassy_nrf::pac::Peripherals::steal() };
        let val = p.POWER.gpregret.read().bits();
        let mode = if val & 0x1 == if cfg!(feature = "gatt_first") { 0 } else { 1 } {
            info!("Running in GATT mode");
            let app = GattApp::enable(sd);
            Mode::Gatt(app)
        } else {
            info!("Running in MESH mode");
            let app = MeshApp::enable();
            Mode::Mesh(app)
        };

        Self { sd, mode }
    }

    pub fn flash(&self) -> Flash {
        Flash::take(self.sd)
    }

    pub fn mount(&'static self, s: Spawner, p: &BoardPeripherals) {
        match &self.mode {
            Mode::Gatt(app) => app.mount(s, self.sd, p),
            Mode::Mesh(app) => app.mount(s, self.sd, p),
        }
    }

    pub fn switch(&'static self) -> ! {
        unsafe {
            match self.mode {
                Mode::Gatt(_) => raw::sd_power_gpregret_clr(0, 0x1),
                Mode::Mesh(_) => raw::sd_power_gpregret_set(0, 0x1),
            };
        }
        cortex_m::peripheral::SCB::sys_reset();
    }

    pub async fn post(&self, leds: &mut Leds) {
        match &self.mode {
            Mode::Gatt(_) => {
                leds.red.on().ok();
                Timer::after(Duration::from_secs(1)).await;
                leds.green.on().ok();
                Timer::after(Duration::from_secs(1)).await;
                leds.blue.on().ok();
                Timer::after(Duration::from_secs(1)).await;
                leds.yellow.on().ok();
                Timer::after(Duration::from_secs(1)).await;
                leds.red.off().ok();
                leds.green.off().ok();
                leds.blue.off().ok();
                leds.yellow.off().ok();
            }
            Mode::Mesh(_) => {
                leds.yellow.on().ok();
                Timer::after(Duration::from_secs(1)).await;
                leds.blue.on().ok();
                Timer::after(Duration::from_secs(1)).await;
                leds.green.on().ok();
                Timer::after(Duration::from_secs(1)).await;
                leds.red.on().ok();
                Timer::after(Duration::from_secs(1)).await;
                leds.red.off().ok();
                leds.green.off().ok();
                leds.blue.off().ok();
                leds.yellow.off().ok();
            }
        }
    }
}

#[embassy::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}
