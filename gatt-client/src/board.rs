use bluer::{
    gatt::remote::{Characteristic, Service},
    Device,
};
use serde_json::json;

pub struct BurrBoard {
    device: Device,
}

impl BurrBoard {
    pub fn new(device: Device) -> Self {
        Self { device }
    }

    pub async fn read_sensors(&self) -> bluer::Result<serde_json::Value> {
        const BOARD_SERVICE_UUID: uuid::Uuid =
            uuid::Uuid::from_u128(0x0000186000001000800000805f9b34fb);
        const TEMPERATURE_CHAR_UUID: uuid::Uuid =
            uuid::Uuid::from_u128(0x00002a6e00001000800000805f9b34fb);

        let service = self.find_service(BOARD_SERVICE_UUID).await?.unwrap();
        let c = self
            .find_char(&service, TEMPERATURE_CHAR_UUID)
            .await?
            .unwrap();

        let value = c.read().await?;
        let temp: u8 = value[0];

        Ok(json!({ "temperature": temp }))
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
