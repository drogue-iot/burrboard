use bluer::{
    gatt::remote::{Characteristic, Service},
    Device,
};
use core::pin::Pin;
use futures::{Stream, StreamExt};
use serde_json::json;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

pub struct BurrBoard {
    device: Device,
}

const BOARD_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x0000186000001000800000805f9b34fb);

const SENSORS_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002a6e00001000800000805f9b34fb);

const INTERVAL_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001b2500001000800000805f9b34fb);

const RED_LED_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002ae200001000800000805f9b34fb);
const GREEN_LED_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002ae300001000800000805f9b34fb);
const BLUE_LED_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002ae400001000800000805f9b34fb);
const YELLOW_LED_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002ae500001000800000805f9b34fb);

const FIRMWARE_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x0000186100001000800000805f9b34fb);

const CONTROL_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x0000123600001000800000805f9b34fb);
const OFFSET_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x0000123500001000800000805f9b34fb);
const FIRMWARE_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x0000123400001000800000805f9b34fb);
const VERSION_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x0000123700001000800000805f9b34fb);

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
    pub fn new(device: Device) -> Self {
        Self { device }
    }

    pub async fn set_interval(&self, i: u16) -> bluer::Result<()> {
        self.write_char(BOARD_SERVICE_UUID, INTERVAL_CHAR_UUID, &i.to_le_bytes())
            .await
    }

    pub async fn set_led(&self, led: Led, value: bool) -> bluer::Result<()> {
        let c = match led {
            Led::Red => RED_LED_CHAR_UUID,
            Led::Green => GREEN_LED_CHAR_UUID,
            Led::Blue => BLUE_LED_CHAR_UUID,
            Led::Yellow => YELLOW_LED_CHAR_UUID,
        };
        let val = if value { 1 } else { 0 };
        self.write_char(BOARD_SERVICE_UUID, c, &[val]).await
    }

    async fn read_firmware_offset(&self) -> bluer::Result<u32> {
        let data = self
            .read_char(FIRMWARE_SERVICE_UUID, OFFSET_CHAR_UUID)
            .await?;
        Ok(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub async fn read_firmware_version(&self) -> bluer::Result<String> {
        let data = self
            .read_char(FIRMWARE_SERVICE_UUID, VERSION_CHAR_UUID)
            .await?;
        Ok(String::from_str(core::str::from_utf8(&data).unwrap()).unwrap())
    }

    pub async fn mark_booted(&self) -> bluer::Result<()> {
        // Trigger DFU process
        self.write_char(FIRMWARE_SERVICE_UUID, CONTROL_CHAR_UUID, &[4])
            .await
    }

    pub fn address(&self) -> bluer::Address {
        self.device.address()
    }

    pub async fn update_firmware(&self, firmware: &[u8]) -> Result<(), anyhow::Error> {
        let mut buf = [0; 16];

        // Trigger DFU process
        self.write_char(FIRMWARE_SERVICE_UUID, CONTROL_CHAR_UUID, &[1])
            .await?;

        println!("Triggered DFU init sequence");
        // Wait until firmware offset is reset
        while self.read_firmware_offset().await? != 0 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        println!(
            "Offset is reset, starting write of {} bytes",
            firmware.len()
        );
        let mut offset: u32 = 0;

        for chunk in firmware.chunks(16) {
            buf[0..chunk.len()].copy_from_slice(chunk);
            if chunk.len() < buf.len() {
                buf[chunk.len()..].fill(0);
            }
            self.write_char(FIRMWARE_SERVICE_UUID, FIRMWARE_CHAR_UUID, &buf)
                .await?;
            log::info!("Write {} bytes at offset {}", buf.len(), offset);
            offset += buf.len() as u32;
            if offset % 4096 == 0 {
                println!("{} bytes written", offset)
            }

            // Wait until firmware offset is incremented
            while self.read_firmware_offset().await? != offset {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
        if offset % 4096 == 0 {
            println!("{} bytes written", offset)
        }

        // Write signal that DFU process is done and should be applied
        log::debug!("DFU process done, setting reset");
        self.write_char(FIRMWARE_SERVICE_UUID, CONTROL_CHAR_UUID, &[2])
            .await?;

        Ok(())
    }

    pub async fn update_firmware_from_file(&self, firmware: &Path) -> Result<(), anyhow::Error> {
        println!("Updating firmware from file {:?}", firmware);
        let mut f = File::open(firmware)?;

        let mut buffer = Vec::new();
        // read the whole file
        f.read_to_end(&mut buffer)?;
        self.update_firmware(&buffer[..]).await
    }

    pub async fn read_sensors(&self) -> bluer::Result<serde_json::Value> {
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
        &self,
    ) -> Result<Pin<Box<impl Stream<Item = serde_json::Value>>>, anyhow::Error> {
        let sensors = self
            .stream_char(BOARD_SERVICE_UUID, SENSORS_CHAR_UUID)
            .await?
            .map(|data| Self::data_to_json(&data));

        Ok(Box::pin(sensors))
    }

    async fn read_char(&self, service: uuid::Uuid, c: uuid::Uuid) -> bluer::Result<Vec<u8>> {
        let service = self.find_service(service).await?.unwrap();
        let c = self.find_char(&service, c).await?.unwrap();

        let value = c.read().await?;
        Ok(value)
    }

    async fn write_char(
        &self,
        service: uuid::Uuid,
        c: uuid::Uuid,
        value: &[u8],
    ) -> bluer::Result<()> {
        let service = self.find_service(service).await?.unwrap();
        let c = self.find_char(&service, c).await?.unwrap();

        c.write(value).await
    }

    async fn stream_char(
        &self,
        service: uuid::Uuid,
        c: uuid::Uuid,
    ) -> bluer::Result<impl Stream<Item = Vec<u8>>> {
        let service = self.find_service(service).await?.unwrap();
        let c = self.find_char(&service, c).await?.unwrap();

        c.notify().await
    }

    async fn find_char(
        &self,
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

    async fn find_service(&self, service: uuid::Uuid) -> bluer::Result<Option<Service>> {
        for s in self.device.services().await? {
            let uuid = s.uuid().await?;
            if uuid == service {
                return Ok(Some(s));
            }
        }
        Ok(None)
    }
}
