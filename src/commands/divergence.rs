use anyhow::{anyhow, Result};
use clap::Args;
use colored::Colorize;
use std::collections::BTreeMap;

use crate::client::eth::EthClient;
use crate::client::ipfs::IpfsClient;
use crate::client::registry::RegistryClient;
use crate::client::{check_divergence_at_block, poi::POIClient, subgraph::GraphClient};
use crate::models::Indexer;
use crate::utils::{
    display_divergence_summary, display_error, display_header, display_info, display_poi_groups,
    display_success, display_warning, group_pois_by_hash,
};

#[derive(Debug, Args)]
pub struct CheckDivergenceCommand {
    #[arg(long, help = "Deployment ID (IPFS hash)")]
    deployment: String,

    #[arg(long, help = "Start block for binary search")]
    start_block: Option<u32>,

    #[arg(long, help = "End block for binary search")]
    end_block: Option<u32>,

    #[arg(long, help = "Indexer ID with correct POI", env = "TRUSTED_INDEXER")]
    indexer: String,

    #[arg(long, help = "API key for The Graph", env = "GRAPH_API_KEY")]
    api_key: String,

    #[arg(
        long,
        help = "Timeout for fetching POIs",
        default_value = "https://ipfs.thegraph.com"
    )]
    ipfs_url: String,

    #[arg(long, help = "Max retries for fetching POIs", default_value = "3")]
    max_retries: u32,
}

impl CheckDivergenceCommand {
    pub async fn execute(self) -> Result<()> {
        display_header("POI Divergence Checker");
        display_info("Deployment", &self.deployment);

        let ipfs_url = self.ipfs_url.clone();
        let ipfs_client = IpfsClient::new(ipfs_url)?;

        let manifest = if self.start_block.is_none() || self.end_block.is_none() {
            Some(ipfs_client.fetch_manifest(&self.deployment).await?)
        } else {
            None
        };

        let start_block = match &self.start_block {
            Some(start_block) => *start_block,
            None => {
                println!("\n{}", "Fetching start block from IPFS...".bright_cyan());
                let block = ipfs_client
                    .get_start_block(manifest.as_ref().unwrap())
                    .await?;
                display_success(&format!("Fetched start block: {}", block));
                block
            }
        };

        let end_block = match &self.end_block {
            Some(end_block) => *end_block,
            None => {
                println!("\n{}", "Fetching network from manifest...".bright_cyan());
                let network = ipfs_client
                    .get_network(manifest.as_ref().unwrap())
                    .await?
                    .ok_or_else(|| anyhow!("Network not found in manifest"))?;
                display_info("Network", &network);

                println!("{}", "Fetching RPC URL from registry...".bright_cyan());
                let registry_client = RegistryClient::new().await?;
                let rpc_url = registry_client.get_public_rpc_url(&network).await?;
                display_info("RPC URL", &rpc_url);

                println!("{}", "Fetching chain head block...".bright_cyan());
                let eth_client = EthClient::new(rpc_url)?;
                let block = eth_client.get_chain_head_block_number().await?;
                display_success(&format!("Fetched end block: {}", block));
                block
            }
        };

        display_info("Search Range", &format!("{} → {}", start_block, end_block));
        display_info("Reference Indexer", &self.indexer);

        println!("\n{}", "Fetching active indexers...".bright_cyan());

        let graph_client = GraphClient::new(self.api_key.clone())?;
        let indexers = graph_client.fetch_indexers(&self.deployment).await?;

        if !indexers.contains_key(&self.indexer) {
            display_error(&format!(
                "Reference indexer '{}' not found in active allocations",
                self.indexer
            ));
            return Err(anyhow!("Invalid reference indexer"));
        }

        display_success(&format!("Found {} active indexers", indexers.len()));

        let poi_client = POIClient::new()?;

        println!(
            "\n{}",
            "Starting binary search for diverged block...".bright_cyan()
        );
        println!("{}", "─".repeat(60).bright_black());

        match self
            .find_diverged_block(&poi_client, &indexers, start_block, end_block)
            .await?
        {
            Some(block) => {
                display_divergence_summary(true, Some(block), start_block, end_block);

                println!("\n{}", "Fetching POIs at diverged block...".bright_cyan());
                self.display_pois_at_block(&poi_client, &indexers, block)
                    .await?;
            }
            None => {
                display_divergence_summary(false, None, start_block, end_block);
                display_success("All indexers have matching POIs in the specified range");
            }
        }

        Ok(())
    }

    async fn find_diverged_block(
        &self,
        poi_client: &POIClient,
        indexers: &BTreeMap<String, Indexer>,
        start_block: u32,
        end_block: u32,
    ) -> Result<Option<u32>> {
        let mut left = start_block;
        let mut right = end_block;
        let mut diverged_block = None;

        while left <= right {
            let mid = left + (right - left) / 2;

            print!(
                "{} Checking block {} (range: {} - {})... ",
                "→".bright_cyan(),
                mid.to_string().bright_white(),
                left.to_string().bright_black(),
                right.to_string().bright_black()
            );

            let (has_divergence, diverged_indexers) = check_divergence_at_block(
                poi_client,
                indexers,
                &self.deployment,
                mid,
                &self.indexer,
            )
            .await?;

            if has_divergence {
                println!(
                    "{} Divergence found ({} indexers)",
                    "✗".red(),
                    diverged_indexers.len().to_string().red()
                );
                diverged_block = Some(mid);
                right = mid - 1;
            } else {
                println!("{} All POIs match", "✓".green());
                left = mid + 1;
            }
        }

        Ok(diverged_block)
    }

    async fn display_pois_at_block(
        &self,
        poi_client: &POIClient,
        indexers: &BTreeMap<String, Indexer>,
        block: u32,
    ) -> Result<()> {
        let mut pois = Vec::new();
        let mut failed_indexers = Vec::new();

        for (indexer_id, indexer) in indexers {
            match poi_client
                .fetch_poi_with_retry(&indexer.url, &self.deployment, block, self.max_retries)
                .await
            {
                Ok(poi) => {
                    pois.push((indexer_id.clone(), poi));
                }
                Err(e) => {
                    failed_indexers.push((indexer_id.clone(), e.to_string()));
                }
            }
        }

        if !failed_indexers.is_empty() {
            display_warning(&format!(
                "Failed to fetch POI from {} indexer(s)",
                failed_indexers.len()
            ));
            for (indexer_id, error) in &failed_indexers {
                println!(
                    "  • {}: {}",
                    indexer_id.bright_black(),
                    error.bright_black()
                );
            }
        }

        let poi_groups = group_pois_by_hash(indexers, &pois, &self.indexer);
        display_poi_groups(poi_groups, block, &self.indexer);

        Ok(())
    }
}
