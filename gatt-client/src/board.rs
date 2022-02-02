use bluer::{
    gatt::remote::{Characteristic, Service},
    Device,
};
use futures::{pin_mut, stream, Stream, StreamExt};
use serde_json::json;

pub struct BurrBoard {
    device: Device,
}

pub const BOARD_SERVICE_UUID: uuid::Uuid =
    uuid::Uuid::from_u128(0x0000186000001000800000805f9b34fb);
pub const TEMPERATURE_CHAR_UUID: uuid::Uuid =
    uuid::Uuid::from_u128(0x00002a6e00001000800000805f9b34fb);

impl BurrBoard {
    pub fn new(device: Device) -> Self {
        Self { device }
    }

    pub async fn read_sensors(&self) -> bluer::Result<serde_json::Value> {
        let service = self.find_service(BOARD_SERVICE_UUID).await?.unwrap();
        let c = self
            .find_char(&service, TEMPERATURE_CHAR_UUID)
            .await?
            .unwrap();

        let value = c.read().await?;
        let temp: u8 = value[0];

        Ok(json!({ "temperature": temp }))
    }

    pub async fn stream_sensors(&self) -> bluer::Result<impl Stream<Item = serde_json::Value>> {
        let service = self.find_service(BOARD_SERVICE_UUID).await?.unwrap();
        let c = self
            .find_char(&service, TEMPERATURE_CHAR_UUID)
            .await?
            .unwrap();

        let temperature = c.notify().await?;
        Ok(temperature.map(|v| json!({ "temperature": v })))
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
