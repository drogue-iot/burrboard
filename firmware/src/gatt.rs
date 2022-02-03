use crate::counter::{Counter, CounterMessage};
use crate::dfu::{DfuCommand, FirmwareManager};
use crate::flash::SharedFlashHandle;
use crate::{
    accel::{AccelValues, Accelerometer, Read as AccelRead},
    analog::{AnalogSensors, Read as AnalogRead},
};
use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
use nrf_softdevice::ble::{Connection, FixedGattValue};

use embassy::time::Duration;

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
    firmware: Vec<u8, 16>,

    #[characteristic(uuid = "1235", read)]
    offset: u32,

    #[characteristic(uuid = "1236", write)]
    control: u8,
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
            ticker: Ticker::every(Duration::from_secs(2)),
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
    buffer: [u8; 4096],
    b_offset: usize,
    f_offset: usize,
    dfu: Address<FirmwareManager<SharedFlashHandle>>,
}

impl BurrBoardFirmware {
    pub fn new(
        service: &'static FirmwareUpdateService,
        dfu: Address<FirmwareManager<SharedFlashHandle>>,
    ) -> Self {
        Self {
            service,
            dfu,
            buffer: [0; 4096],
            b_offset: 0,
            f_offset: 0,
        }
    }

    async fn flush(&mut self) {
        info!("Flushing Firmware buffer!");
        if self.b_offset > 0 {
            self.dfu
                .request(DfuCommand::Write(
                    self.f_offset as u32,
                    &self.buffer[..self.b_offset],
                ))
                .unwrap()
                .await;
            self.f_offset += self.b_offset;
            self.b_offset = 0;
        }
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
                            } else if *value == 2 {
                                self.flush().await;
                            } else if *value == 3 {
                                // Sanity check
                                let offset = self.service.offset_get().unwrap();
                                if offset != self.f_offset as u32 {
                                    info!(
                                        "Service offset({}) differs from flush offset({})!",
                                        offset, self.f_offset
                                    );
                                } else {
                                    self.dfu.notify(DfuCommand::Swap).unwrap();
                                }
                            }
                        }
                        FirmwareUpdateServiceEvent::FirmwareWrite(value) => {
                            let offset = self.service.offset_get().unwrap();
                            self.buffer[self.b_offset..self.b_offset + value.len()]
                                .copy_from_slice(&value);
                            self.b_offset += value.len();
                            self.service.offset_set(offset + value.len() as u32);
                            if self.b_offset == self.buffer.len() {
                                self.flush().await;
                            }
                        }
                    }
                }
            }
        }
    }
}
