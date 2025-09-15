use anyhow::{anyhow, Result};
use reqwest::{Client, Url};
use serde_json::json;
use std::collections::BTreeMap;
use std::time::Duration;

use crate::models::{AllocationsData, GraphQLQuery, GraphQLResponse, Indexer, POIResponse};

pub struct GraphClient {
    client: Client,
    network_url: String,
    api_key: String,
}

impl GraphClient {
    pub fn new(api_key: String) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        let network_url = "https://gateway.thegraph.com/api/subgraphs/id/DZz4kDTdmzWLWsV373w2bSmoar3umKKH9y82SUKr5qmp".to_string();

        Ok(Self {
            client,
            network_url,
            api_key,
        })
    }

    pub async fn fetch_indexers(&self, deployment: &str) -> Result<BTreeMap<String, Indexer>> {
        let query = format!(
            r#"{{
                allocations( 
                    where: {{
                        status: Active
                        subgraphDeployment_: {{ ipfsHash: "{}" }}
                    }}
                ) {{
                    indexer {{
                        id
                        url
                    }}
                }}
            }}"#,
            deployment
        );

        let mut request = self
            .client
            .post(&self.network_url)
            .json(&GraphQLQuery { query });

        request = request.bearer_auth(&self.api_key);

        let response = request.send().await?;
        let data: GraphQLResponse<AllocationsData> = response.json().await?;

        let indexers = data
            .data
            .allocations
            .into_iter()
            .map(|allocation| (allocation.indexer.id.clone(), allocation.indexer))
            .collect();

        Ok(indexers)
    }
}

pub struct POIClient {
    client: Client,
}

impl POIClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

        Ok(Self { client })
    }

    pub async fn fetch_poi(&self, url: &str, deployment: &str, block: u64) -> Result<String> {
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
        block: u64,
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

pub async fn check_divergence_at_block(
    poi_client: &POIClient,
    indexers: &BTreeMap<String, Indexer>,
    deployment: &str,
    block: u64,
    correct_indexer_id: &str,
) -> Result<(bool, Vec<String>)> {
    let correct_indexer = indexers
        .get(correct_indexer_id)
        .ok_or_else(|| anyhow!("Correct indexer not found in active allocations"))?;

    let correct_poi = poi_client
        .fetch_poi_with_retry(&correct_indexer.url, deployment, block, 3)
        .await?;

    let mut diverged_indexers = Vec::new();

    for (indexer_id, indexer) in indexers.iter() {
        if indexer_id == correct_indexer_id {
            continue;
        }

        match poi_client
            .fetch_poi_with_retry(&indexer.url, deployment, block, 3)
            .await
        {
            Ok(poi) => {
                if poi != correct_poi {
                    diverged_indexers.push(indexer_id.clone());
                }
            }
            Err(_) => {
                continue;
            }
        }
    }

    Ok((!diverged_indexers.is_empty(), diverged_indexers))
}
