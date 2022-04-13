use crate::Leds;
use crate::{accel::*, analog::*, counter::*};
use core::future::Future;
use drogue_device::{
    drivers::ble::gatt::dfu::{FirmwareGattService, FirmwareService, FirmwareServiceEvent},
    firmware::*,
    flash::*,
    Actor, ActorContext, Address, Inbox,
};
use embassy::executor::Spawner;
use nrf_softdevice::ble::Connection;
use nrf_softdevice::ble::{gatt_server, peripheral};
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
    pub firmware: FirmwareService,
}

/// Gatt services for our module
#[nrf_softdevice::gatt_service(uuid = "1860")]
pub struct BurrBoardService {
    #[characteristic(uuid = "2a6e", read, notify)]
    pub sensors: Vec<u8, 22>,

    #[characteristic(uuid = "2ae2", read, write)]
    pub red_led: u8,
    #[characteristic(uuid = "2ae3", read, write)]
    pub green_led: u8,
    #[characteristic(uuid = "2ae4", read, write)]
    pub blue_led: u8,
    #[characteristic(uuid = "2ae5", read, write)]
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

pub struct BurrBoardMonitor {
    ticker: Ticker,
    service: &'static BurrBoardService,
    analog: Address<AnalogRequest>,
    button_a: Address<CounterRequest>,
    button_b: Address<CounterRequest>,
    accel: Address<AccelRequest>,
    connections: Vec<Connection, 2>,
    notifications: bool,
    leds: Leds,
}

impl BurrBoardMonitor {
    pub fn new(
        service: &'static BurrBoardService,
        analog: Address<AnalogRequest>,
        accel: Address<AccelRequest>,
        button_a: Address<CounterRequest>,
        button_b: Address<CounterRequest>,
        leds: Leds,
    ) -> Self {
        Self {
            service,
            connections: Vec::new(),
            ticker: Ticker::every(Duration::from_secs(1)),
            analog,
            accel,
            button_a,
            button_b,
            leds,
            notifications: false,
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
            BurrBoardServiceEvent::SensorsCccdWrite { notifications } => {
                self.notifications = *notifications;
            }
            BurrBoardServiceEvent::ReportIntervalWrite(period) => {
                info!("Changing report interval to {} ms", *period);
                self.ticker = Ticker::every(Duration::from_millis(*period as u64));
            }

            BurrBoardServiceEvent::RedLedWrite(val) => {
                if *val == 0 {
                    self.leds.red.off().ok();
                } else {
                    self.leds.red.on().ok();
                }
            }
            BurrBoardServiceEvent::GreenLedWrite(val) => {
                if *val == 0 {
                    self.leds.green.off().ok();
                } else {
                    self.leds.green.on().ok();
                }
            }
            BurrBoardServiceEvent::BlueLedWrite(val) => {
                if *val == 0 {
                    self.leds.blue.off().ok();
                } else {
                    self.leds.blue.on().ok();
                }
            }
            BurrBoardServiceEvent::YellowLedWrite(val) => {
                if *val == 0 {
                    self.leds.yellow.off().ok();
                } else {
                    self.leds.yellow.on().ok();
                }
            }
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
    {
        async move {
            loop {
                let inbox_fut = inbox.next();
                let ticker_fut = self.ticker.next();

                pin_mut!(inbox_fut);
                pin_mut!(ticker_fut);

                match select(inbox_fut, ticker_fut).await {
                    Either::Left((m, _)) => match m {
                        MonitorEvent::Connected(conn) => {
                            self.add_connection(&conn);
                        }
                        MonitorEvent::Disconnected(conn) => {
                            self.remove_connection(&conn);
                        }
                        MonitorEvent::Event(event) => {
                            self.handle_event(&event);
                        }
                    },
                    Either::Right((_, _)) => {
                        let mut data: Vec<u8, 22> = Vec::new();
                        let analog = self.analog.request(AnalogRead).await;

                        data.extend_from_slice(&analog.temperature.to_le_bytes())
                            .ok();
                        data.extend_from_slice(&analog.brightness.to_le_bytes())
                            .ok();
                        data.push(analog.battery).ok();

                        let (button_a, counter_a) = self.button_a.request(CounterRead).await;
                        let (button_b, counter_b) = self.button_b.request(CounterRead).await;

                        data.extend_from_slice(&counter_a.to_le_bytes()).ok();
                        data.extend_from_slice(&counter_b.to_le_bytes()).ok();

                        let accel = self.accel.request(AccelRead).await;
                        data.extend_from_slice(&accel.x.to_le_bytes()).ok();
                        data.extend_from_slice(&accel.y.to_le_bytes()).ok();
                        data.extend_from_slice(&accel.z.to_le_bytes()).ok();

                        let buttons_leds = button_a as u8;
                        let buttons_leds = buttons_leds | (button_b as u8) << 1;
                        let buttons_leds = buttons_leds | (self.leds.red.is_on() as u8) << 2;
                        let buttons_leds = buttons_leds | (self.leds.green.is_on() as u8) << 3;
                        let buttons_leds = buttons_leds | (self.leds.blue.is_on() as u8) << 4;
                        let buttons_leds = buttons_leds | (self.leds.yellow.is_on() as u8) << 5;

                        data.push(buttons_leds).ok();

                        self.service.sensors_set(data.clone()).ok();

                        for c in self.connections.iter() {
                            if self.notifications {
                                self.service.sensors_notify(&c, data.clone()).ok();
                            }
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
    monitor: Address<MonitorEvent>,
    firmware: Address<FirmwareServiceEvent>,
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

        monitor.notify(MonitorEvent::Connected(conn.clone())).await;
        let res = gatt_server::run(&conn, server, |e| match e {
            BurrBoardServerEvent::Board(e) => {
                let _ = monitor.try_notify(MonitorEvent::Event(e));
            }
            BurrBoardServerEvent::DeviceInfo(_) => {}
            BurrBoardServerEvent::Firmware(e) => {
                let _ = firmware.try_notify(e);
            }
        })
        .await;
        monitor.notify(MonitorEvent::Disconnected(conn)).await;

        if let Err(e) = res {
            info!("gatt_server run exited with error: {:?}", e);
        }
    }
}

pub struct GattApp {
    server: BurrBoardServer,

    monitor: ActorContext<BurrBoardMonitor>,
    firmware: ActorContext<
        FirmwareGattService<'static, SharedFirmwareManager<'static, SharedFlash<'static, Flash>>>,
    >,
}

impl GattApp {
    pub fn enable(sd: &'static Softdevice) -> Self {
        let server = gatt_server::register(sd).unwrap();
        Self {
            server,
            monitor: ActorContext::new(),
            firmware: ActorContext::new(),
        }
    }

    pub fn mount(&'static self, s: Spawner, sd: &'static Softdevice, p: &BoardPeripherals) {
        let monitor = self.monitor.mount(
            s,
            BurrBoardMonitor::new(
                &self.server.board,
                p.analog.clone(),
                p.accel.clone(),
                p.counter_a.clone(),
                p.counter_b.clone(),
                p.leds.clone(),
            ),
        );

        let firmware = self.firmware.mount(
            s,
            FirmwareGattService::new(
                &self.server.firmware,
                p.dfu.clone(),
                crate::FIRMWARE_REVISION
                    .unwrap_or(crate::FIRMWARE_VERSION)
                    .as_bytes(),
                64,
            )
            .unwrap(),
        );
        s.spawn(bluetooth_task(sd, &self.server, monitor, firmware))
            .unwrap();
    }
}
