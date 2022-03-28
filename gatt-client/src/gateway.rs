pub struct Gateway {
    http: String,
    application: String,
    device: String,
    password: String,
}

impl Gateway {
    pub fn new(http: String, application: String, device: String, password: String) -> Self {
        Self {
            http,
            application,
            device,
            password,
        }
    }

    pub async fn publish(&self, device: &str, data: &[u8]) -> Result<(), anyhow::Error> {
        todo!()
    }
}
