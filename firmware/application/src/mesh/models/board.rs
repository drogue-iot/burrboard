use super::super::COMPANY_IDENTIFIER;
use drogue_device::drivers::ble::mesh::model::Message;
use drogue_device::drivers::ble::mesh::model::ModelIdentifier;
use drogue_device::drivers::ble::mesh::pdu::access::Opcode;
use drogue_device::drivers::ble::mesh::InsufficientBuffer;
use drogue_device::opcode;
use heapless::Vec;

pub const BURRBOARD_SERVER: ModelIdentifier = ModelIdentifier::Vendor(COMPANY_IDENTIFIER, 0x0001);
pub const BURRBOARD_CLIENT: ModelIdentifier = ModelIdentifier::Vendor(COMPANY_IDENTIFIER, 0x0002);

#[derive(defmt::Format, Debug)]
pub struct BurrBoardState {
    pub temperature: i16,
    pub brightness: u16,
    pub accel: [i16; 3],
    pub battery: u8,
    pub button_a: u32,
    pub button_b: u32,
    pub red_led: bool,
    pub green_led: bool,
    pub blue_led: bool,
    pub yellow_led: bool,
}

pub enum BurrBoardMessage {
    State(BurrBoardState),
}

pub struct BurrBoardModel;

impl Message for BurrBoardMessage {
    fn opcode(&self) -> Opcode {
        BURRBOARD_STATE
    }

    fn emit_parameters<const N: usize>(
        &self,
        xmit: &mut heapless::Vec<u8, N>,
    ) -> Result<(), InsufficientBuffer> {
        match self {
            Self::State(state) => state.emit_parameters(xmit),
        }
    }
}

opcode!(BURRBOARD_STATE 0x12, 0x34);

impl BurrBoardState {
    fn emit_parameters<const N: usize>(
        &self,
        xmit: &mut Vec<u8, N>,
    ) -> Result<(), InsufficientBuffer> {
        xmit.extend_from_slice(&self.temperature.to_le_bytes())
            .map_err(|_| InsufficientBuffer)?;
        xmit.extend_from_slice(&self.brightness.to_le_bytes())
            .map_err(|_| InsufficientBuffer)?;
        xmit.extend_from_slice(&self.accel[0].to_le_bytes())
            .map_err(|_| InsufficientBuffer)?;
        xmit.extend_from_slice(&self.accel[1].to_le_bytes())
            .map_err(|_| InsufficientBuffer)?;
        xmit.extend_from_slice(&self.accel[2].to_le_bytes())
            .map_err(|_| InsufficientBuffer)?;
        xmit.push(self.battery).map_err(|_| InsufficientBuffer)?;
        xmit.extend_from_slice(&self.button_a.to_le_bytes())
            .map_err(|_| InsufficientBuffer)?;
        xmit.extend_from_slice(&self.button_b.to_le_bytes())
            .map_err(|_| InsufficientBuffer)?;
        xmit.push(if self.red_led { 1 } else { 0 })
            .map_err(|_| InsufficientBuffer)?;
        xmit.push(if self.green_led { 1 } else { 0 })
            .map_err(|_| InsufficientBuffer)?;
        xmit.push(if self.blue_led { 1 } else { 0 })
            .map_err(|_| InsufficientBuffer)?;
        xmit.push(if self.yellow_led { 1 } else { 0 })
            .map_err(|_| InsufficientBuffer)?;
        Ok(())
    }
}
