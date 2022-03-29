pub struct Gateway {
    client: reqwest::Client,
    http: String,
    user: String,
    password: String,
}

impl Gateway {
    pub fn new(http: String, user: String, password: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            http,
            user,
            password,
        }
    }

    // Best effort publish event
    pub async fn publish(&self, device: &str, data: &[u8]) {
        match self
            .client
            .post(&self.http)
            .query(&[("as", device)])
            .basic_auth(&self.user, Some(&self.password))
            .json(&data)
            .send()
            .await
        {
            Ok(resp) if !resp.status().is_success() => {
                println!(
                    "Error response {}: {}",
                    resp.status(),
                    resp.text().await.unwrap_or_default()
                );
            }
            Ok(_) => {}
            Err(e) => {
                println!("Request error: {:?}", e);
            }
        }
    }
}
