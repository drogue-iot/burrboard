use crate::counter::{Counter, CounterMessage};
use crate::Leds;
use crate::{
    accel::{AccelValues, Accelerometer, Read as AccelRead},
    analog::{AnalogSensors, Read as AnalogRead},
};
use core::future::Future;
use drogue_device::{
    actors::dfu::{DfuCommand, FirmwareManager},
    actors::flash::SharedFlashHandle,
    traits::led::Led,
    Actor, ActorContext, Address, Inbox,
};
use embassy::executor::Spawner;
use nrf_softdevice::ble::{gatt_server, peripheral};
use nrf_softdevice::ble::{Connection, FixedGattValue};
use nrf_softdevice::raw;
use nrf_softdevice::{Flash, Softdevice};

use embassy::time::Duration;

use crate::board::*;
use embassy::time::Ticker;
use futures::{future::select, future::Either, pin_mut, StreamExt};
use heapless::Vec;

#[nrf_softdevice::gatt_server]
pub struct BurrBoardServer {
    pub board: BurrBoardService,
    pub device_info: DeviceInformationService,
    pub firmware: FirmwareUpdateService,
}

/// Gatt services for our module
#[nrf_softdevice::gatt_service(uuid = "1860")]
pub struct BurrBoardService {
    #[characteristic(uuid = "2a6e", read, notify)]
    pub temperature: i8,
    #[characteristic(uuid = "2b01", read, notify)]
    pub brightness: u16,

    #[characteristic(uuid = "2101", read, notify)]
    pub accel: Vec<u8, 6>,

    #[characteristic(uuid = "2a19", read, notify)]
    pub battery_level: u8,

    #[characteristic(uuid = "2aeb", read, notify)]
    pub button_a: u32,

    #[characteristic(uuid = "2aec", read, notify)]
    pub button_b: u32,

    #[characteristic(uuid = "2ae2", write)]
    pub red_led: u8,
    #[characteristic(uuid = "2ae3", write)]
    pub green_led: u8,
    #[characteristic(uuid = "2ae4", write)]
    pub blue_led: u8,
    #[characteristic(uuid = "2ae5", write)]
    pub yellow_led: u8,

    #[characteristic(uuid = "1b25", read, write)]
    pub report_interval: u16,
}

#[nrf_softdevice::gatt_service(uuid = "180a")]
pub struct DeviceInformationService {
    #[characteristic(uuid = "2a24", read)]
    pub model_number: Vec<u8, 32>,
    #[characteristic(uuid = "2a25", read)]
    pub serial_number: Vec<u8, 32>,
    #[characteristic(uuid = "2a27", read)]
    pub hardware_revision: Vec<u8, 4>,
    #[characteristic(uuid = "2a29", read)]
    pub manufacturer_name: Vec<u8, 32>,
}

#[nrf_softdevice::gatt_service(uuid = "1861")]
pub struct FirmwareUpdateService {
    #[characteristic(uuid = "1234", write)]
    firmware: Vec<u8, 64>,

    #[characteristic(uuid = "1235", read)]
    offset: u32,

    #[characteristic(uuid = "1236", write)]
    control: u8,

    #[characteristic(uuid = "1237", read)]
    pub version: Vec<u8, 16>,
}

pub struct BurrBoardMonitor {
    ticker: Ticker,
    service: &'static BurrBoardService,
    analog: Address<AnalogSensors>,
    button_a: Address<Counter>,
    button_b: Address<Counter>,
    accel: Option<Address<Accelerometer>>,
    connections: Vec<Connection, 2>,
    notifications: Notifications,
}

pub struct Notifications {
    temperature: bool,
    brightness: bool,
    accel: bool,
    battery_level: bool,
    button_a: bool,
    button_b: bool,
}

impl BurrBoardMonitor {
    pub fn new(
        service: &'static BurrBoardService,
        analog: Address<AnalogSensors>,
        accel: Option<Address<Accelerometer>>,
        button_a: Address<Counter>,
        button_b: Address<Counter>,
    ) -> Self {
        Self {
            service,
            connections: Vec::new(),
            ticker: Ticker::every(Duration::from_secs(1)),
            analog,
            accel,
            button_a,
            button_b,
            notifications: Notifications {
                temperature: false,
                brightness: false,
                accel: false,
                battery_level: false,
                button_a: false,
                button_b: false,
            },
        }
    }

    pub fn add_connection(&mut self, connection: &Connection) {
        self.connections.push(connection.clone()).ok().unwrap();
    }

    pub fn remove_connection(&mut self, connection: &Connection) {
        for i in 0..self.connections.len() {
            if self.connections[i].handle() == connection.handle() {
                self.connections.swap_remove(i);
                break;
            }
        }
    }

    pub fn handle_event(&mut self, event: &BurrBoardServiceEvent) {
        match event {
            BurrBoardServiceEvent::TemperatureCccdWrite { notifications } => {
                self.notifications.temperature = *notifications;
            }
            BurrBoardServiceEvent::BrightnessCccdWrite { notifications } => {
                self.notifications.brightness = *notifications;
            }
            BurrBoardServiceEvent::BatteryLevelCccdWrite { notifications } => {
                self.notifications.battery_level = *notifications;
            }
            BurrBoardServiceEvent::ButtonACccdWrite { notifications } => {
                self.notifications.button_a = *notifications;
            }
            BurrBoardServiceEvent::ButtonBCccdWrite { notifications } => {
                self.notifications.button_b = *notifications;
            }
            BurrBoardServiceEvent::ReportIntervalWrite(period) => {
                info!("Changing report interval to {} ms", *period);
                self.ticker = Ticker::every(Duration::from_millis(*period as u64));
            }
            _ => {}
        }
    }
}

pub enum MonitorEvent {
    Connected(Connection),
    Disconnected(Connection),
    Event(BurrBoardServiceEvent),
}

impl Actor for BurrBoardMonitor {
    type Message<'m> = MonitorEvent;

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
    {
        async move {
            loop {
                let inbox_fut = inbox.next();
                let ticker_fut = self.ticker.next();

                pin_mut!(inbox_fut);
                pin_mut!(ticker_fut);

                match select(inbox_fut, ticker_fut).await {
                    Either::Left((r, _)) => {
                        if let Some(mut m) = r {
                            match m.message() {
                                MonitorEvent::Connected(conn) => {
                                    self.add_connection(conn);
                                }
                                MonitorEvent::Disconnected(conn) => {
                                    self.remove_connection(conn);
                                }
                                MonitorEvent::Event(event) => {
                                    self.handle_event(event);
                                }
                            }
                        }
                    }
                    Either::Right((_, _)) => {
                        let accel = if let Some(accel) = self.accel {
                            accel.request(AccelRead).unwrap().await.unwrap()
                        } else {
                            AccelValues { x: 0, y: 0, z: 0 }
                        };
                        let analog = self.analog.request(AnalogRead).unwrap().await;
                        let button_a_presses = self
                            .button_a
                            .request(CounterMessage::Read)
                            .unwrap()
                            .await
                            .unwrap();
                        let button_b_presses = self
                            .button_b
                            .request(CounterMessage::Read)
                            .unwrap()
                            .await
                            .unwrap();

                        let temperature = (analog.temperature / 100) as i8;
                        self.service.temperature_set(temperature);
                        self.service.brightness_set(analog.brightness);
                        self.service.battery_level_set(analog.battery);
                        self.service.button_a_set(button_a_presses);
                        self.service.button_b_set(button_b_presses);

                        let x: [u8; 2] = accel.x.to_le_bytes();
                        let y: [u8; 2] = accel.y.to_le_bytes();
                        let z: [u8; 2] = accel.z.to_le_bytes();
                        self.service.accel_set(
                            Vec::from_slice(&[x[0], x[1], y[0], y[1], z[0], z[1]]).unwrap(),
                        );

                        for c in self.connections.iter() {
                            if self.notifications.temperature {
                                self.service.temperature_notify(&c, temperature).unwrap();
                            }
                            if self.notifications.brightness {
                                self.service
                                    .brightness_notify(&c, analog.brightness)
                                    .unwrap();
                            }
                            if self.notifications.battery_level {
                                self.service
                                    .battery_level_notify(&c, analog.battery)
                                    .unwrap();
                            }
                            if self.notifications.button_a {
                                self.service.button_a_notify(&c, button_a_presses).unwrap();
                            }
                            if self.notifications.button_b {
                                self.service.button_b_notify(&c, button_b_presses).unwrap();
                            }

                            if self.notifications.accel {
                                let x: [u8; 2] = accel.x.to_le_bytes();
                                let y: [u8; 2] = accel.y.to_le_bytes();
                                let z: [u8; 2] = accel.z.to_le_bytes();
                                self.service.accel_notify(
                                    &c,
                                    Vec::from_slice(&[x[0], x[1], y[0], y[1], z[0], z[1]]).unwrap(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct BurrBoardFirmware {
    service: &'static FirmwareUpdateService,
    dfu: Address<FirmwareManager<SharedFlashHandle<Flash>>>,
}

impl BurrBoardFirmware {
    pub fn new(
        service: &'static FirmwareUpdateService,
        dfu: Address<FirmwareManager<SharedFlashHandle<Flash>>>,
    ) -> Self {
        Self { service, dfu }
    }
}

impl Actor for BurrBoardFirmware {
    type Message<'m> = FirmwareUpdateServiceEvent;

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
    {
        async move {
            loop {
                if let Some(mut m) = inbox.next().await {
                    match m.message() {
                        FirmwareUpdateServiceEvent::ControlWrite(value) => {
                            info!("Write firmware control: {}", value);
                            if *value == 1 {
                                self.service.offset_set(0);
                                self.dfu.request(DfuCommand::Start).unwrap().await.unwrap();
                            } else if *value == 2 {
                                self.dfu.notify(DfuCommand::Finish).unwrap();
                            } else if *value == 3 {
                                self.dfu.notify(DfuCommand::Booted).unwrap();
                            }
                        }
                        FirmwareUpdateServiceEvent::FirmwareWrite(value) => {
                            let offset = self.service.offset_get().unwrap();
                            self.dfu
                                .request(DfuCommand::WriteBlock(value))
                                .unwrap()
                                .await
                                .unwrap();
                            self.service.offset_set(offset + value.len() as u32);
                        }
                    }
                }
            }
        }
    }
}

#[embassy::task]
pub async fn bluetooth_task(
    sd: &'static Softdevice,
    server: &'static BurrBoardServer,
    mut leds: Leds,
    monitor: Address<BurrBoardMonitor>,
    firmware: Address<BurrBoardFirmware>,
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
            BurrBoardServerEvent::Firmware(e) => {
                firmware.notify(e);
            }
        })
        .await;
        monitor.notify(MonitorEvent::Disconnected(conn));

        if let Err(e) = res {
            info!("gatt_server run exited with error: {:?}", e);
        }
    }
}

pub struct GattApp {
    sd: &'static Softdevice,
    server: BurrBoardServer,

    monitor: ActorContext<BurrBoardMonitor>,
    firmware: ActorContext<BurrBoardFirmware>,
}

impl GattApp {
    pub fn enable() -> (&'static Softdevice, Self) {
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
        let server = gatt_server::register(sd).unwrap();
        (
            sd,
            Self {
                sd,
                server,
                monitor: ActorContext::new(),
                firmware: ActorContext::new(),
            },
        )
    }

    pub fn flash(&self) -> Flash {
        Flash::take(self.sd)
    }

    pub fn mount(&'static self, s: Spawner, p: BoardPeripherals) {
        self.server.firmware.version_set(
            heapless::Vec::from_slice(
                crate::FIRMWARE_REVISION
                    .unwrap_or(crate::FIRMWARE_VERSION)
                    .as_bytes(),
            )
            .unwrap(),
        );
        let monitor = self.monitor.mount(
            s,
            BurrBoardMonitor::new(
                &self.server.board,
                p.analog,
                p.accel,
                p.counter_a,
                p.counter_b,
            ),
        );

        let firmware = self
            .firmware
            .mount(s, BurrBoardFirmware::new(&self.server.firmware, p.dfu));
        s.spawn(bluetooth_task(
            self.sd,
            &self.server,
            p.leds,
            monitor,
            firmware,
        ))
        .unwrap();
    }
}
