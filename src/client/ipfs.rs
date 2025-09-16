use std::time::Duration;

use anyhow::Result;
use regex::Regex;
use reqwest::Client;

pub struct IpfsClient {
    client: Client,
    url: String,
}

impl IpfsClient {
    pub fn new(url: String) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
        Ok(Self { client, url })
    }

    pub async fn fetch_manifest(&self, hash: &str) -> Result<String> {
        let url = format!("{}/ipfs/api/v0/cat?arg={}", self.url, hash);
        let response = self.client.get(url).send().await?;
        let body = response.text().await?;
        Ok(body)
    }

    pub async fn get_start_block(&self, manifest: &str) -> Result<u32> {
        let re = Regex::new(r"startBlock:\s*(\d+)").unwrap();
        let start_blocks: Vec<u32> = re
            .captures_iter(&manifest)
            .filter_map(|cap| cap[1].parse::<u32>().ok())
            .collect();

        Ok(start_blocks
            .iter()
            .min()
            .map(|min| min)
            .unwrap_or(&0)
            .to_owned())
    }

    pub async fn get_network(&self, manifest: &str) -> Result<Option<String>> {
        let re = Regex::new(r"network:\s*(\w+)").unwrap();
        let networks: Vec<String> = re
            .captures_iter(manifest)
            .map(|cap| cap[1].to_string())
            .collect();
        Ok(networks.first().cloned())
    }
}
