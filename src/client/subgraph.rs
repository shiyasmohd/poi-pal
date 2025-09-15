use anyhow::Result;
use reqwest::Client;
use std::collections::BTreeMap;
use std::time::Duration;

use crate::models::{AllocationsData, GraphQLQuery, GraphQLResponse, Indexer};

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
