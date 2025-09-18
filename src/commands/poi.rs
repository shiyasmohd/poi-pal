use anyhow::{anyhow, Result};
use clap::Args;
use colored::Colorize;
use std::sync::Arc;
use tokio::task::JoinSet;

use crate::client::eth::EthClient;
use crate::client::ipfs::IpfsClient;
use crate::client::registry::RegistryClient;
use crate::client::{poi::POIClient, subgraph::GraphClient};
use crate::models::IndexerPOI;
use crate::utils::{display_error, display_header, display_info, display_pois, display_success};

#[derive(Debug, Args)]
pub struct PoiCommand {
    #[arg(help = "Deployment ID (IPFS hash)")]
    deployment: String,

    #[arg(long, help = "Block number to fetch POI for")]
    block: Option<u32>,

    #[arg(long, help = "API key for The Graph", env = "GRAPH_API_KEY")]
    api_key: String,

    #[arg(long, help = "Max retries for fetching POIs", default_value = "3")]
    max_retries: u32,

    #[arg(
        long,
        help = "IPFS base URL to fetch subgraph manifest",
        default_value = "https://ipfs.thegraph.com"
    )]
    ipfs_url: String,

    #[arg(
        long,
        help = "Indexers to include for POI fetching (check only these)",
        value_delimiter = ','
    )]
    only_indexers: Option<Vec<String>>,
}

impl PoiCommand {
    pub async fn execute(self) -> Result<()> {
        display_header("Proof of Indexing (POI) Fetcher");
        display_info("Deployment", &self.deployment);

        // Fetch block if not provided
        let block = match self.block {
            Some(b) => b,
            None => {
                println!(
                    "\n{}",
                    "Block not provided. Fetching chain head block...".bright_cyan()
                );

                // Fetch manifest from IPFS
                println!("{}", "Fetching manifest from IPFS...".bright_cyan());
                let ipfs_client = IpfsClient::new(self.ipfs_url.clone())?;
                let manifest = ipfs_client.fetch_manifest(&self.deployment).await?;

                // Get network from manifest
                println!("{}", "Fetching network from manifest...".bright_cyan());
                let network = ipfs_client
                    .get_network(&manifest)
                    .await?
                    .ok_or_else(|| anyhow!("Network not found in manifest"))?;
                display_info("Network", &network);

                // Get RPC URL from registry
                println!("{}", "Fetching RPC URL from registry...".bright_cyan());
                let registry_client = RegistryClient::new().await?;
                let rpc_url = registry_client.get_public_rpc_url(&network).await?;
                display_info("RPC URL", &rpc_url);

                // Fetch chain head block
                println!("{}", "Fetching chain head block...".bright_cyan());
                let eth_client = EthClient::new(rpc_url)?;
                let head_block = eth_client.get_chain_head_block_number().await?;
                display_success(&format!("Using chain head block: {}", head_block));
                head_block - 15
            }
        };

        display_info("Block", &block.to_string());

        println!("\n{}", "Fetching active indexers...".bright_cyan());

        let graph_client = GraphClient::new(self.api_key)?;
        let mut indexers = graph_client.fetch_indexers(&self.deployment).await?;

        if indexers.is_empty() {
            display_error("No active indexers found for this deployment");
            return Ok(());
        }

        display_success(&format!("Found {} active indexers", indexers.len()));

        // Filter to only include specified indexers
        if let Some(ref include_list) = self.only_indexers {
            let initial_count = indexers.len();
            indexers.retain(|id, _| include_list.contains(id));
            let filtered_count = initial_count - indexers.len();
            if filtered_count > 0 {
                display_info("Total indexers", &format!("{}", initial_count));
                display_info("Checking indexers", &format!("{}", indexers.len()));
            }

            if indexers.is_empty() {
                display_error("None of the specified indexers are active for this deployment");
                return Ok(());
            }
        }

        println!("\n{}", "Fetching POIs from indexers...".bright_cyan());

        let poi_client = Arc::new(POIClient::new()?);
        let mut pois = Vec::new();
        let mut failed_count = 0;

        let mut tasks = JoinSet::new();

        for (indexer_id, indexer) in indexers.iter() {
            let id = indexer_id.clone();
            let url = indexer.url.clone();
            let deployment = self.deployment.clone();
            let block_num = block;
            let poi_client = Arc::clone(&poi_client);
            let max_retries = self.max_retries;

            tasks.spawn(async move {
                let poi_result = poi_client
                    .fetch_poi_with_retry(&url, &deployment, block_num, max_retries)
                    .await;
                (id, url, poi_result)
            });
        }

        while let Some(result) = tasks.join_next().await {
            match result {
                Ok((indexer_id, indexer_url, poi_result)) => {
                    print!("  {} {:<50} ", "→".bright_cyan(), indexer_id);

                    match poi_result {
                        Ok(poi) => {
                            println!("{}", "✓".green());
                            pois.push(IndexerPOI {
                                indexer_id,
                                indexer_url,
                                poi,
                            });
                        }
                        Err(e) => {
                            println!("{} ({})", "✗".red(), e.to_string().bright_black());
                            failed_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "  {} Task failed: {}",
                        "✗".red(),
                        e.to_string().bright_black()
                    );
                    failed_count += 1;
                }
            }
        }

        if failed_count > 0 {
            display_info(
                "Failed to fetch POI from",
                &format!("{} indexer(s)", failed_count),
            );
        }

        display_pois(pois, block, &self.deployment);

        Ok(())
    }
}
