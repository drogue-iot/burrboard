use anyhow::anyhow;
use bluer::{
    gatt::remote::{Characteristic, Service},
    Adapter, Address, Device,
};
use core::pin::Pin;
use futures::{Stream, StreamExt};
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub struct BurrBoard {
    adapter: Arc<Adapter>,
    device: Address,
    board: Option<Device>,
}

const BOARD_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x0000186000001000800000805f9b34fb);

const SENSORS_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002a6e00001000800000805f9b34fb);
const INTERVAL_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001b2500001000800000805f9b34fb);
const RED_LED_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002ae200001000800000805f9b34fb);
const GREEN_LED_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002ae300001000800000805f9b34fb);
const BLUE_LED_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002ae400001000800000805f9b34fb);
const YELLOW_LED_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002ae500001000800000805f9b34fb);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Led {
    Red,
    Green,
    Blue,
    Yellow,
}

impl FromStr for Led {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "red" => Ok(Led::Red),
            "green" => Ok(Led::Green),
            "blue" => Ok(Led::Blue),
            "yellow" => Ok(Led::Yellow),
            _ => Err("unknown LED name"),
        }
    }
}

unsafe impl Send for BurrBoard {}

impl BurrBoard {
    pub fn new(device: &str, adapter: Arc<Adapter>) -> Self {
        Self {
            device: Address::from_str(device).unwrap(),
            adapter,
            board: None,
        }
    }

    async fn connect(&mut self) -> bluer::Result<&mut Device> {
        if self.board.is_none() {
            loop {
                if let Ok(device) = self.adapter.device(self.device) {
                    // Make sure we get a fresh start
                    let _ = device.disconnect().await;
                    sleep(Duration::from_secs(2)).await;
                    match device.is_connected().await {
                        Ok(false) => {
                            log::debug!("Connecting...");
                            loop {
                                match device.connect().await {
                                    Ok(()) => break,
                                    Err(err) => {
                                        log::error!("Connect error: {}", &err);
                                    }
                                }
                            }
                            log::debug!("Connected1");
                            self.board.replace(device);
                            break;
                        }
                        Ok(true) => {
                            log::debug!("Connected2");
                            self.board.replace(device);
                            break;
                        }
                        Err(e) => {
                            log::info!("Error checking connection, retrying: {:?}", e);
                        }
                    }
                }
                sleep(Duration::from_secs(2)).await;
            }
        }
        Ok(self.board.as_mut().unwrap())
    }

    pub async fn set_interval(&mut self, i: u16) -> bluer::Result<()> {
        self.write_char(BOARD_SERVICE_UUID, INTERVAL_CHAR_UUID, &i.to_le_bytes())
            .await
    }

    pub async fn set_led(&mut self, led: Led, value: bool) -> bluer::Result<()> {
        let c = match led {
            Led::Red => RED_LED_CHAR_UUID,
            Led::Green => GREEN_LED_CHAR_UUID,
            Led::Blue => BLUE_LED_CHAR_UUID,
            Led::Yellow => YELLOW_LED_CHAR_UUID,
        };
        let val = if value { 1 } else { 0 };
        self.write_char(BOARD_SERVICE_UUID, c, &[val]).await
    }

    pub fn address(&self) -> Address {
        self.device
    }

    pub async fn read_sensors(&mut self) -> bluer::Result<serde_json::Value> {
        let data = self
            .read_char(BOARD_SERVICE_UUID, SENSORS_CHAR_UUID)
            .await?;
        Ok(Self::data_to_json(&data))
    }

    fn data_to_json(data: &[u8]) -> serde_json::Value {
        assert_eq!(data.len(), 22);

        let temp: f32 = (i16::from_le_bytes([data[0], data[1]]) as f32) / 100.0;
        let brightness: u16 = u16::from_le_bytes([data[2], data[3]]);

        let battery: u8 = data[4];

        let counter_a = u16::from_le_bytes([data[5], data[6]]);
        let counter_b = u16::from_le_bytes([data[7], data[8]]);

        let accel: (f32, f32, f32) = (
            f32::from_le_bytes([data[9], data[10], data[11], data[12]]),
            f32::from_le_bytes([data[13], data[14], data[15], data[16]]),
            f32::from_le_bytes([data[17], data[18], data[19], data[20]]),
        );

        let buttons_leds = data[21];
        let button_a = (buttons_leds & 0x1) != 0;
        let button_b = ((buttons_leds >> 1) & 0x1) != 0;

        let red_led = ((buttons_leds >> 2) & 0x1) != 0;
        let green_led = ((buttons_leds >> 3) & 0x1) != 0;
        let blue_led = ((buttons_leds >> 4) & 0x1) != 0;
        let yellow_led = ((buttons_leds >> 5) & 0x1) != 0;

        json!({"temperature": {"value": temp}, "light": { "value": brightness },
                "led_1": { "state": red_led },
                "led_2": { "state": green_led },
                "led_3": { "state": blue_led },
                "led_4": { "state": yellow_led },
                "accelerometer": {
            "x": accel.0,
            "y": accel.1,
            "z": accel.2,
                }, "device": { "battery": (battery as f32) / 100.0 }, "button_a": { "presses": counter_a, "state": button_a  } , "button_b": { "presses": counter_b, "state": button_b} })
    }

    pub async fn stream_sensors(
        &mut self,
    ) -> Result<Pin<Box<impl Stream<Item = serde_json::Value>>>, anyhow::Error> {
        let sensors = self
            .stream_char(BOARD_SERVICE_UUID, SENSORS_CHAR_UUID)
            .await?
            .map(|data| Self::data_to_json(&data));

        Ok(Box::pin(sensors))
    }

    async fn read_char(&mut self, service: uuid::Uuid, c: uuid::Uuid) -> bluer::Result<Vec<u8>> {
        let service = self.find_service(service).await?.unwrap();
        let c = self.find_char(&service, c).await?.unwrap();

        let value = c.read().await?;
        Ok(value)
    }

    async fn write_char(
        &mut self,
        service: uuid::Uuid,
        c: uuid::Uuid,
        value: &[u8],
    ) -> bluer::Result<()> {
        let service = self.find_service(service).await?.unwrap();
        let c = self.find_char(&service, c).await?.unwrap();

        c.write(value).await
    }

    async fn stream_char(
        &mut self,
        service: uuid::Uuid,
        c: uuid::Uuid,
    ) -> Result<impl Stream<Item = Vec<u8>>, anyhow::Error> {
        if let Some(service) = self.find_service(service).await? {
            if let Some(c) = self.find_char(&service, c).await? {
                return Ok(c.notify().await?);
            }
        }
        Err(anyhow!("Error locating service {} and char {}", service, c))
    }

    async fn find_char(
        &mut self,
        service: &Service,
        characteristic: uuid::Uuid,
    ) -> bluer::Result<Option<Characteristic>> {
        for c in service.characteristics().await? {
            let uuid = c.uuid().await?;
            if uuid == characteristic {
                return Ok(Some(c));
            }
        }
        Ok(None)
    }

    async fn find_service(&mut self, service: uuid::Uuid) -> bluer::Result<Option<Service>> {
        let device = self.connect().await?;
        for s in device.services().await? {
            let uuid = s.uuid().await?;
            if uuid == service {
                return Ok(Some(s));
            }
        }
        Ok(None)
    }
}
