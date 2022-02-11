use bluer::{
    gatt::remote::{Characteristic, Service},
    Device,
};
use bytes::Bytes;
use core::pin::Pin;
use futures::{stream, stream_select, Stream, StreamExt};
use serde_json::json;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

pub struct BurrBoard {
    device: Device,
}

const BOARD_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x0000186000001000800000805f9b34fb);

const TEMPERATURE_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002a6e00001000800000805f9b34fb);
const BRIGHTNESS_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002b0100001000800000805f9b34fb);
const ACCEL_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x0000210100001000800000805f9b34fb);
const BATTERY_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002a1900001000800000805f9b34fb);
const BUTTON_A_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002aeb00001000800000805f9b34fb);
const BUTTON_B_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002aec00001000800000805f9b34fb);

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
        let mut buf = [0; 64];

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

        for chunk in firmware.chunks(64) {
            buf[0..chunk.len()].copy_from_slice(&chunk);
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

    pub async fn update_firmware_from_file(&self, firmware: &PathBuf) -> Result<(), anyhow::Error> {
        println!("Updating firmware from file {:?}", firmware);
        let mut f = File::open(firmware)?;

        let mut buffer = Vec::new();
        // read the whole file
        f.read_to_end(&mut buffer)?;
        self.update_firmware(&buffer[..]).await
    }

    pub async fn read_sensors(&self) -> bluer::Result<serde_json::Value> {
        let temp: u8 = self
            .read_char(BOARD_SERVICE_UUID, TEMPERATURE_CHAR_UUID)
            .await?[0];
        let data = self
            .read_char(BOARD_SERVICE_UUID, BRIGHTNESS_CHAR_UUID)
            .await?;
        let brightness: u16 = u16::from_le_bytes([data[0], data[1]]);

        let data = self.read_char(BOARD_SERVICE_UUID, ACCEL_CHAR_UUID).await?;
        let accel: (i16, i16, i16) = (
            i16::from_le_bytes([data[0], data[1]]),
            i16::from_le_bytes([data[2], data[3]]),
            i16::from_le_bytes([data[4], data[5]]),
        );

        let battery = self
            .read_char(BOARD_SERVICE_UUID, BATTERY_CHAR_UUID)
            .await?[0];

        let data = self
            .read_char(BOARD_SERVICE_UUID, BUTTON_A_CHAR_UUID)
            .await?;
        let button_a = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        let data = self
            .read_char(BOARD_SERVICE_UUID, BUTTON_B_CHAR_UUID)
            .await?;
        let button_b = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        Ok(
            json!({ "temperature": temp, "brightness": brightness, "accel": accel, "battery": battery, "button_a": button_a, "button_b": button_b }),
        )
    }

    pub async fn stream_sensors(
        &self,
    ) -> Result<Pin<Box<impl Stream<Item = serde_json::Value>>>, anyhow::Error> {
        let t = self
            .stream_char(BOARD_SERVICE_UUID, TEMPERATURE_CHAR_UUID)
            .await?
            .map(|v| json!({"temperature": v[0]}));

        let b = self
            .stream_char(BOARD_SERVICE_UUID, BRIGHTNESS_CHAR_UUID)
            .await?
            .map(|v| json!({"brightness": u16::from_le_bytes([v[0], v[1]])}));

        let accel = self
            .stream_char(BOARD_SERVICE_UUID, ACCEL_CHAR_UUID)
            .await?
            .map(|v| json!({"accel": (i16::from_le_bytes([v[0], v[1]]), i16::from_le_bytes([v[2], v[3]]), i16::from_le_bytes([v[4], v[5]]))}));

        let batt = self
            .stream_char(BOARD_SERVICE_UUID, BATTERY_CHAR_UUID)
            .await?
            .map(|v| json!({"battery": v[0]}));

        let b_a = self
            .stream_char(BOARD_SERVICE_UUID, BUTTON_A_CHAR_UUID)
            .await?
            .map(|v| json!({"button_a": u32::from_le_bytes([v[0], v[1], v[2], v[3]])}));

        let b_b = self
            .stream_char(BOARD_SERVICE_UUID, BUTTON_B_CHAR_UUID)
            .await?
            .map(|v| json!({"button_b": u32::from_le_bytes([v[0], v[1], v[2], v[3]])}));

        let j = stream_select!(
            Box::pin(t),
            Box::pin(b),
            Box::pin(accel),
            Box::pin(batt),
            Box::pin(b_a),
            Box::pin(b_b)
        );
        Ok(Box::pin(j))
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
        return Ok(None);
    }

    async fn find_service(&self, service: uuid::Uuid) -> bluer::Result<Option<Service>> {
        for s in self.device.services().await? {
            let uuid = s.uuid().await?;
            if uuid == service {
                return Ok(Some(s));
            }
        }
        return Ok(None);
    }
}
