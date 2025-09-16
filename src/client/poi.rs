use anyhow::{anyhow, Result};
use reqwest::{Client, Url};
use serde_json::json;
use std::time::Duration;

use crate::models::POIResponse;

#[derive(Clone)]
pub struct POIClient {
    client: Client,
}

impl POIClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

        Ok(Self { client })
    }

    pub async fn fetch_poi(&self, url: &str, deployment: &str, block: u32) -> Result<String> {
        let url: Url = url.parse()?;
        let status_url = url.join("status")?;

        let query = format!(
            r#"{{ publicProofsOfIndexing(requests: [{{deployment: "{}", blockNumber: "{}"}}]) {{ deployment proofOfIndexing block {{ number }} }} }}"#,
            deployment, block
        );

        let response = self
            .client
            .post(status_url)
            .json(&json!({ "query": query }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch POI: HTTP {}", response.status()));
        }

        let mut poi_response: POIResponse = response.json().await?;

        if poi_response.data.pois.is_empty() {
            return Err(anyhow!("No POI found for block {}", block));
        }

        Ok(poi_response.data.pois.remove(0).poi)
    }

    pub async fn fetch_poi_with_retry(
        &self,
        url: &str,
        deployment: &str,
        block: u32,
        max_retries: u32,
    ) -> Result<String> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            match self.fetch_poi(url, deployment, block).await {
                Ok(poi) => return Ok(poi),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| anyhow!("Failed to fetch POI after {} retries", max_retries)))
    }
}
