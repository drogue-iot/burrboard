use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareMetadata {
    pub version: String,
    pub size: usize,
    pub data: FirmwareData,
}

#[derive(Debug)]
pub enum FirmwareError {
    Io(std::io::Error),
    Parse(serde_json::Error),
}

impl core::fmt::Display for FirmwareError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::Parse(e) => e.fmt(f),
        }
    }
}

impl From<std::io::Error> for FirmwareError {
    fn from(error: std::io::Error) -> Self {
        FirmwareError::Io(error)
    }
}

impl From<serde_json::Error> for FirmwareError {
    fn from(error: serde_json::Error) -> Self {
        FirmwareError::Parse(error)
    }
}

impl serde::ser::StdError for FirmwareError {}

impl FirmwareMetadata {
    pub fn from_file(path: &PathBuf) -> Result<Self, FirmwareError> {
        let data = std::fs::read_to_string(path)?;
        let metadata = serde_json::from_str(&data)?;
        Ok(metadata)
    }

    pub fn from_http(url: String, size: usize, version: String) -> Self {
        Self {
            version,
            size,
            data: FirmwareData::Http(url),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FirmwareData {
    #[serde(rename = "file")]
    File(PathBuf),
    #[serde(rename = "http")]
    Http(String),
}
