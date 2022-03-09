#![cfg_attr(not(feature = "std"), no_std)]
#![no_main]

use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use heapless::{String, Vec};
use postcard::{from_bytes_cobs, to_vec_cobs};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    Start,
    Version,
    Finish,
    Booted,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Response {
    Ok,
    Err(Error),
    OkVersion(String<64>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Protocol,
    Actor,
    Flash,
}

pub const MAX_FRAME_SIZE: usize = 140;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Frame {
    Command(Command),
    Data(Vec<u8, 128>),
    Response(Response),
}

impl Frame {
    pub async fn encode<B: AsyncWriteExt + Unpin>(&self, writer: &mut B) -> Result<(), ()> {
        let v: Vec<u8, MAX_FRAME_SIZE> = to_vec_cobs(self).map_err(|e| {
            #[cfg(feature = "defmt")]
            defmt::info!("Error encoding response");
            ()
        })?;
        #[cfg(feature = "defmt")]
        defmt::info!("Encoding {} bytes", v.len());
        writer
            .write_all(&v.len().to_le_bytes()[..])
            .await
            .map_err(|_| ())?;
        writer.write_all(&v[..]).await.map_err(|_| ())?;
        Ok(())
    }

    pub async fn decode<B: AsyncBufReadExt + Unpin>(reader: &mut B) -> Result<Self, ()> {
        let mut v: [u8; MAX_FRAME_SIZE] = [0; MAX_FRAME_SIZE];

        reader.read_exact(&mut v[..4]).await.map_err(|_| ())?;
        let l = u32::from_le_bytes([v[0], v[1], v[2], v[3]]) as usize;

        reader.read_exact(&mut v[..l]).await.map_err(|_| ())?;
        let f: Frame = from_bytes_cobs(&mut v[..]).map_err(|_| ())?;
        Ok(f)
    }

    #[cfg(feature = "std")]
    pub fn encode_blocking<B: std::io::Write>(&self, writer: &mut B) -> Result<(), ()> {
        let v: Vec<u8, MAX_FRAME_SIZE> = to_vec_cobs(self).map_err(|_| ())?;
        writer
            .write_all(&v.len().to_le_bytes()[..])
            .map_err(|_| ())?;
        writer.write_all(&v[..]).map_err(|_| ())?;
        println!("Sending {} bytes", v.len());
        Ok(())
    }

    #[cfg(feature = "std")]
    pub fn decode_blocking<B: std::io::Read>(reader: &mut B) -> Result<Self, ()> {
        let mut v: [u8; MAX_FRAME_SIZE] = [0; MAX_FRAME_SIZE];

        reader.read_exact(&mut v[..4]).map_err(|_| ())?;
        let l = u32::from_le_bytes([v[0], v[1], v[2], v[3]]) as usize;

        println!("Going to read response of {} bytes", l);
        reader.read_exact(&mut v[..l]).map_err(|_| ())?;

        println!("Read {} bytes", l);
        let f: Frame = from_bytes_cobs(&mut v[..]).map_err(|e| {
            println!("Error: {:?}", e);
            ()
        })?;
        Ok(f)
    }
}
