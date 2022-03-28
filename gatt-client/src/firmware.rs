use bluer::Adapter;
use drgdfu::{FirmwareUpdater, GattBoard};
use std::sync::Arc;
use std::time::Duration;

pub struct FirmwareClient {
    boards: Vec<GattBoard>,
    updater: FirmwareUpdater,
}

impl FirmwareClient {
    pub fn new(url: String, user: String, password: String, adapter: Arc<Adapter>) -> Self {
        Self {
            boards: Vec::new(),
            updater: FirmwareUpdater::Cloud {
                client: reqwest::Client::new(),
                url,
                user,
                password,
                timeout: Duration::from_secs(30),
            },
        }
    }
}
