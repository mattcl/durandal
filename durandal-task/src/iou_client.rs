use anyhow::{bail, Result};
use reqwest::blocking::Client;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct Request {
    url: String,
}

#[derive(Debug, Clone)]
pub struct IouClient {
    rq_client: Client,
    servers: Vec<String>,
}

impl IouClient {
    pub fn new(servers: Vec<String>) -> Self {
        Self {
            rq_client: Client::new(),
            servers,
        }
    }

    pub fn open(&self, url: &str) -> Result<()> {
        let req = Request {
            url: url.to_string(),
        };

        for server in &self.servers {
            let res = self.rq_client.post(server).json(&req).send()?;

            if !res.status().is_success() {
                bail!(format!(
                    "Could not successfully post url to IOU server: {:?}",
                    res.text()
                ));
            }
        }

        Ok(())
    }
}
