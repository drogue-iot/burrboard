use drgdfu::{FirmwareUpdater, GattBoard};
use std::time::Duration;

pub struct FirmwareClient {
    url: String,
    user: String,
    password: String,
}

impl FirmwareClient {
    pub fn new(url: String, user: String, password: String) -> Self {
        Self {
            url,
            user,
            password,
        }
    }

    pub async fn run(&self, board: &mut GattBoard) {
        let updater = FirmwareUpdater::Cloud {
            client: reqwest::Client::new(),
            url: self.url.clone(),
            user: self.user.clone(),
            password: self.password.clone(),
            timeout: Duration::from_secs(30),
        };

        let name = board.address().to_string();

        if let Err(e) = updater.run(board, Some(&name)).await {
            log::warn!("Updating failed for {}: {:?}", board.address(), e);
        }
    }
}
