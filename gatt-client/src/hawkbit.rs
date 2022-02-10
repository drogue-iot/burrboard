use std::time::Duration;

pub struct HawkbitClient {
    url: String,
    tenant: String,
    controller: String,
    device_id: String,
    token: String,
}

impl HawkbitClient {
    pub fn new(url: &str, tenant: &str, controller: &str, device_id: &str, token: &str) -> Self {
        Self {
            url: url.to_string(),
            tenant: tenant.to_string(),
            controller: controller.to_string(),
            device_id: device_id.to_string(),
            token: token.to_string(),
        }
    }

    pub async fn wait_update(&mut self) -> std::io::Result<()> {
        let client = reqwest::Client::new();
        loop {
            let url = format!(
                "{}/{}/controller/v1/{}",
                &self.url, &self.tenant, &self.controller
            );
            println!("URL: {}", &url);
            let res = client
                .get(&url)
                .bearer_auth(&self.token)
                .header("Accept", "application/hal+json")
                .send()
                .await;
            println!("Resposnse: {:?}", res);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        Ok(())
    }
}
