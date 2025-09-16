use anyhow::Result;
use clap::Args;
use colored::Colorize;
use std::sync::Arc;
use tokio::task::JoinSet;

use crate::client::{poi::POIClient, subgraph::GraphClient};
use crate::models::IndexerPOI;
use crate::utils::{display_error, display_header, display_info, display_pois, display_success};

#[derive(Debug, Args)]
pub struct PoiCommand {
    #[arg(long, help = "Deployment ID (IPFS hash)")]
    deployment: String,

    #[arg(long, help = "Block number to fetch POI for")]
    block: u32,

    #[arg(long, help = "API key for The Graph", env = "GRAPH_API_KEY")]
    api_key: String,
}

impl PoiCommand {
    pub async fn execute(self) -> Result<()> {
        display_header("Proof of Indexing (POI) Fetcher");
        display_info("Deployment", &self.deployment);
        display_info("Block", &self.block.to_string());

        println!("\n{}", "Fetching active indexers...".bright_cyan());

        let graph_client = GraphClient::new(self.api_key)?;
        let indexers = graph_client.fetch_indexers(&self.deployment).await?;

        if indexers.is_empty() {
            display_error("No active indexers found for this deployment");
            return Ok(());
        }

        display_success(&format!("Found {} active indexers", indexers.len()));

        println!("\n{}", "Fetching POIs from indexers...".bright_cyan());

        let poi_client = Arc::new(POIClient::new()?);
        let mut pois = Vec::new();
        let mut failed_count = 0;

        let mut tasks = JoinSet::new();

        for (indexer_id, indexer) in indexers.iter() {
            let id = indexer_id.clone();
            let url = indexer.url.clone();
            let deployment = self.deployment.clone();
            let block = self.block;
            let poi_client = Arc::clone(&poi_client);

            tasks.spawn(async move {
                let poi_result = poi_client
                    .fetch_poi_with_retry(&url, &deployment, block, 3)
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

        display_pois(pois, self.block, &self.deployment);

        Ok(())
    }
}
