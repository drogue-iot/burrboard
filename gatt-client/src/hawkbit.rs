use crate::firmware::{Deployment, FirmwareMetadata};
use bytes::Bytes;
use serde_json::json;
use std::time::Duration;

pub struct HawkbitClient {
    url: String,
    tenant: String,
    controller: String,
    token: String,
}

impl HawkbitClient {
    pub fn new(url: &str, tenant: &str, controller: &str, token: &str) -> Self {
        Self {
            url: url.to_string(),
            tenant: tenant.to_string(),
            controller: controller.to_string(),
            token: token.to_string(),
        }
    }

    pub async fn register(&self) -> std::io::Result<()> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/{}/controller/v1/{}",
            &self.url, &self.tenant, &self.controller
        );

        let attributes = json! {{
          "mode": "merge",
          "data": {
            "VIN": "JH4TB2H26CC000001",
            "hwRevision": "3"
          },
          "status": {
            "result": {
              "finished": "success"
            },
            "execution": "closed",
            "details": []
          }
        }};

        let res = client
            .put(&url)
            .header("Authorization", &format!("TargetToken {}", &self.token))
            .header("Accept", "application/hal+json")
            .json(&attributes)
            .send()
            .await;
        match res {
            Ok(_) => {
                println!("Successfully set attributes");
            }
            Err(e) => {
                println!("Error setting attributes: {:?}", e);
            }
        }
        Ok(())
    }

    pub async fn fetch_firmware(&self, path: &str) -> Result<Bytes, anyhow::Error> {
        let client = reqwest::Client::new();
        let res = client
            .get(path)
            .header("Authorization", &format!("TargetToken {}", &self.token))
            .header("Accept", "application/hal+json")
            .send()
            .await?
            .bytes()
            .await?;
        Ok(res)
    }

    pub async fn read_metadata(&self, path: &str) -> Result<Deployment, anyhow::Error> {
        let client = reqwest::Client::new();
        let res: serde_json::Value = client
            .get(path)
            .header("Authorization", &format!("TargetToken {}", &self.token))
            .header("Accept", "application/hal+json")
            .send()
            .await?
            .json()
            .await?;

        let id = res["id"].as_str().unwrap().to_string();
        let chunks = &res["deployment"]["chunks"];
        let chunk = &chunks[0];
        let version = chunk["version"].as_str().unwrap().to_string();
        let artifact = &chunk["artifacts"][0];
        let size: usize = artifact["size"].as_i64().unwrap() as usize;
        let path = artifact["_links"]["download-http"]["href"]
            .as_str()
            .unwrap();
        let metadata = FirmwareMetadata::from_http(path.to_string(), size, version);

        Ok(Deployment { id, metadata })
    }

    pub async fn provide_feedback(
        &self,
        deployment: &Deployment,
        success: bool,
    ) -> Result<(), anyhow::Error> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/{}/controller/v1/{}/deploymentBase/{}/feedback",
            &self.url, &self.tenant, &self.controller, deployment.id
        );

        let feedback = json! {
            {
                "id": deployment.id,
                "status": {
                    "result": {
                        "finished": if success { "success" } else { "failed" },
                    },
                    "execution": "closed",
                    "details": ["Update was successfully installed."],
                }

            }
        };

        client
            .post(&url)
            .header("Authorization", &format!("TargetToken {}", &self.token))
            .header("Accept", "application/hal+json")
            .json(&feedback)
            .send()
            .await?;

        Ok(())
    }

    pub async fn wait_update(&self) -> Result<Deployment, anyhow::Error> {
        let client = reqwest::Client::new();
        loop {
            let url = format!(
                "{}/{}/controller/v1/{}",
                &self.url, &self.tenant, &self.controller
            );
            let res = client
                .get(&url)
                .header("Authorization", &format!("TargetToken {}", &self.token))
                .header("Accept", "application/hal+json")
                .send()
                .await;

            let poll: Duration = match res {
                Ok(res) => {
                    let j: serde_json::Value = res.json().await.unwrap();

                    // If we have a deployment base, return download link
                    if let Some(links) = j.get("_links") {
                        if let Some(base) = links.get("deploymentBase") {
                            if let Some(href) = base.get("href") {
                                if let Some(path) = href.as_str() {
                                    return self.read_metadata(path).await;
                                }
                            }
                        }
                    }

                    let poll = j["config"]["polling"]["sleep"].as_str().unwrap();
                    let mut s = poll.splitn(3, ":");
                    let mut dur = chrono::Duration::zero();
                    if let Some(d) = s.next() {
                        dur = dur + chrono::Duration::days(d.parse::<i64>().unwrap());
                    }

                    if let Some(h) = s.next() {
                        dur = dur + chrono::Duration::hours(h.parse::<i64>().unwrap());
                    }

                    if let Some(s) = s.next() {
                        dur = dur + chrono::Duration::seconds(s.parse::<i64>().unwrap());
                    }
                    dur.to_std().unwrap()
                }
                Err(e) => {
                    println!("ERROR HAWKBIT: {:?}", e);
                    Duration::from_secs(5)
                }
            };
            println!("Polling hawkbit in {} seconds", poll.as_secs());
            tokio::time::sleep(poll).await;
        }
    }
}
