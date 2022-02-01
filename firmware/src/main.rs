#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![allow(unused)]

use drogue_device::{
    actors::button::{Button, ButtonPressed},
    drivers::button::Button as ButtonDriver,
    drivers::led::Led as LedDriver,
    traits::led::Led as _,
    ActorContext, Address,
};
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_nrf::config::Config;
use embassy_nrf::gpio::{Input, Level, NoPin, Output, OutputDrive, Pull};
use embassy_nrf::interrupt::Priority;
use embassy_nrf::peripherals::{P0_02, P0_03, P0_04, P0_05, P0_06, P0_26, P0_27, P0_28, P0_30};
use embassy_nrf::uarte;
use embassy_nrf::{interrupt, Peripherals};
use nrf_softdevice::ble::{gatt_server, peripheral};
use nrf_softdevice::raw;
use nrf_softdevice::Softdevice;

#[cfg(all(feature = "defmt", not(feature = "log")))]
use panic_probe as _;

#[cfg(all(feature = "defmt", not(feature = "log")))]
use defmt_rtt as _;

mod fmt;

#[cfg(feature = "log")]
mod logger;

#[cfg(not(feature = "defmt"))]
use panic_reset as _;

mod accel;
mod analog;
mod counter;
mod dfu;
mod flash;
mod gatt;

use accel::*;
use analog::*;
use counter::*;
use dfu::*;
use flash::*;
use gatt::*;

pub type RedLed = LedDriver<Output<'static, P0_06>>;
pub type GreenLed = LedDriver<Output<'static, P0_30>>;
pub type BlueLed = LedDriver<Output<'static, P0_28>>;
pub type YellowLed = LedDriver<Output<'static, P0_02>>;

pub type ButtonA = ButtonDriver<Input<'static, P0_27>>;
pub type ButtonB = ButtonDriver<Input<'static, P0_26>>;

pub type BatteryPin = P0_04;
pub type TemperaturePin = P0_05;
pub type LightPin = P0_03;

// Application must run at a lower priority than softdevice
fn config() -> Config {
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    config
}

#[embassy::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

#[embassy::main(config = "config()")]
async fn main(s: embassy::executor::Spawner, p: Peripherals) {
    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 4,
            rc_temp_ctiv: 2,
            accuracy: 7,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 6,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 128 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: 32768,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 3,
            central_role_count: 0,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: b"BurrBoard" as *const u8 as _,
            current_len: 9,
            max_len: 9,
            write_perm: unsafe { core::mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);
    s.spawn(softdevice_task(sd)).unwrap();

    static GATT: Forever<BurrBoardServer> = Forever::new();
    let server = GATT.put(gatt_server::register(sd).unwrap());

    #[cfg(feature = "log")]
    {
        logger::init(uarte::Uarte::new(
            p.UARTE0,
            interrupt::take!(UARTE0_UART0),
            p.P0_24,
            p.P0_25,
            NoPin,
            NoPin,
            Default::default(),
        ));
    }

    // Ensure accel is ready
    Timer::after(Duration::from_millis(500)).await;

    // Actor for accelerometer
    static ACCEL: ActorContext<Accelerometer> = ActorContext::new();
    let accel: Option<Address<Accelerometer>> = Accelerometer::new(p.TWISPI0, p.P0_12, p.P0_11)
        .map(|a| ACCEL.mount(s, a))
        .ok();

    // Actor for all analog sensors
    static ANALOG: ActorContext<AnalogSensors> = ActorContext::new();
    let analog = ANALOG.mount(s, AnalogSensors::new(p.SAADC, p.P0_05, p.P0_03, p.P0_04));

    // LEDs
    let mut red = RedLed::new(Output::new(p.P0_06, Level::Low, OutputDrive::Standard));
    let mut green = GreenLed::new(Output::new(p.P0_30, Level::Low, OutputDrive::Standard));
    let mut blue = BlueLed::new(Output::new(p.P0_28, Level::Low, OutputDrive::Standard));
    let mut yellow = YellowLed::new(Output::new(p.P0_02, Level::Low, OutputDrive::Standard));

    // Actor for button A and press counter
    static COUNTER_A: ActorContext<Counter> = ActorContext::new();
    static BUTTON_A: ActorContext<
        Button<ButtonDriver<Input<'static, P0_27>>, ButtonPressed<Counter>>,
    > = ActorContext::new();
    let counter_a = COUNTER_A.mount(s, Counter::new(BoardButton::A, &server.board));
    let button_a = BUTTON_A.mount(
        s,
        Button::new(
            ButtonDriver::new(Input::new(p.P0_27, Pull::None)),
            ButtonPressed(counter_a, CounterMessage::Increment),
        ),
    );

    // Actor for button B and press counter
    static COUNTER_B: ActorContext<Counter> = ActorContext::new();
    static BUTTON_B: ActorContext<
        Button<ButtonDriver<Input<'static, P0_26>>, ButtonPressed<Counter>>,
    > = ActorContext::new();
    let counter_b = COUNTER_B.mount(s, Counter::new(BoardButton::B, &server.board));
    let button_b = BUTTON_B.mount(
        s,
        Button::new(
            ButtonDriver::new(Input::new(p.P0_26, Pull::None)),
            ButtonPressed(counter_b, CounterMessage::Increment),
        ),
    );

    // Actor for shared access to flash
    static FLASH: ActorContext<SharedFlash> = ActorContext::new();
    let flash = FLASH.mount(s, SharedFlash::new(sd));

    // Actor for DFU
    static DFU: ActorContext<FirmwareManager<SharedFlashHandle>> = ActorContext::new();
    DFU.mount(s, FirmwareManager::new(SharedFlashHandle(flash)));

    // Self test
    red.on();
    Timer::after(Duration::from_secs(1)).await;
    green.on();
    Timer::after(Duration::from_secs(1)).await;
    blue.on();
    Timer::after(Duration::from_secs(1)).await;
    yellow.on();
    Timer::after(Duration::from_secs(1)).await;
    red.off();
    green.off();
    blue.off();
    yellow.off();

    // BLE Gatt test service
    #[cfg(feature = "gatt")]
    {
        static MONITOR: ActorContext<BurrBoardMonitor> = ActorContext::new();
        let monitor = MONITOR.mount(
            s,
            BurrBoardMonitor::new(&server.board, analog, accel, counter_a, counter_b),
        );
        s.spawn(bluetooth_task(
            sd,
            server,
            Leds {
                red,
                green,
                blue,
                yellow,
            },
            monitor,
        ))
        .unwrap();
    }
}

pub struct Leds {
    pub red: RedLed,
    pub green: GreenLed,
    pub blue: BlueLed,
    pub yellow: YellowLed,
}

#[embassy::task]
async fn bluetooth_task(
    sd: &'static Softdevice,
    server: &'static BurrBoardServer,
    mut leds: Leds,
    monitor: Address<BurrBoardMonitor>,
) {
    #[rustfmt::skip]
    let adv_data = &[
        0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x60, 0x18,
        0x0a, 0x09, b'B', b'u', b'r', b'r', b'B', b'o', b'a', b'r', b'd',
    ];
    #[rustfmt::skip]
    let scan_data = &[
        0x03, 0x03, 0x09, 0x18,
    ];

    loop {
        let config = peripheral::Config::default();
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data,
            scan_data,
        };
        let conn = unwrap!(peripheral::advertise_connectable(sd, adv, &config).await);

        info!("advertising done!");

        monitor.notify(MonitorEvent::Connected(conn.clone()));
        let res = gatt_server::run(&conn, server, |e| match e {
            BurrBoardServerEvent::Board(event) => match event {
                BurrBoardServiceEvent::RedLedWrite(val) => {
                    if val == 0 {
                        leds.red.off();
                    } else {
                        leds.red.on();
                    }
                }
                BurrBoardServiceEvent::GreenLedWrite(val) => {
                    if val == 0 {
                        leds.green.off();
                    } else {
                        leds.green.on();
                    }
                }
                BurrBoardServiceEvent::BlueLedWrite(val) => {
                    if val == 0 {
                        leds.blue.off();
                    } else {
                        leds.blue.on();
                    }
                }
                BurrBoardServiceEvent::YellowLedWrite(val) => {
                    if val == 0 {
                        leds.yellow.off();
                    } else {
                        leds.yellow.on();
                    }
                }
                e => {
                    monitor.notify(MonitorEvent::Event(e));
                }
                _ => {}
            },
            BurrBoardServerEvent::DeviceInfo(_) => {}
            BurrBoardServerEvent::Firmware(_) => {}
        })
        .await;
        monitor.notify(MonitorEvent::Disconnected(conn));

        if let Err(e) = res {
            info!("gatt_server run exited with error: {:?}", e);
        }
    }
}
