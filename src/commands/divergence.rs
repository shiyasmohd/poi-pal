use anyhow::{anyhow, Result};
use clap::Args;
use colored::Colorize;
use std::collections::BTreeMap;

use crate::client::{check_divergence_at_block, GraphClient, POIClient};
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
    start_block: u64,

    #[arg(long, help = "End block for binary search")]
    end_block: u64,

    #[arg(long, help = "Indexer ID with correct POI")]
    correct_indexer_id: String,

    #[arg(long, help = "API key for The Graph")]
    api_key: String,
}

impl CheckDivergenceCommand {
    pub async fn execute(self) -> Result<()> {
        display_header("POI Divergence Checker");
        display_info("Deployment", &self.deployment);
        display_info(
            "Search Range",
            &format!("{} → {}", self.start_block, self.end_block),
        );
        display_info("Reference Indexer", &self.correct_indexer_id);

        println!("\n{}", "Fetching active indexers...".bright_cyan());

        let graph_client = GraphClient::new(self.api_key.clone())?;
        let indexers = graph_client.fetch_indexers(&self.deployment).await?;

        if !indexers.contains_key(&self.correct_indexer_id) {
            display_error(&format!(
                "Reference indexer '{}' not found in active allocations",
                self.correct_indexer_id
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

        match self.find_diverged_block(&poi_client, &indexers).await? {
            Some(block) => {
                display_divergence_summary(true, Some(block), self.start_block, self.end_block);

                println!("\n{}", "Fetching POIs at diverged block...".bright_cyan());
                self.display_pois_at_block(&poi_client, &indexers, block)
                    .await?;
            }
            None => {
                display_divergence_summary(false, None, self.start_block, self.end_block);
                display_success("All indexers have matching POIs in the specified range");
            }
        }

        Ok(())
    }

    async fn find_diverged_block(
        &self,
        poi_client: &POIClient,
        indexers: &BTreeMap<String, Indexer>,
    ) -> Result<Option<u64>> {
        let mut left = self.start_block;
        let mut right = self.end_block;
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
                &self.correct_indexer_id,
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
        block: u64,
    ) -> Result<()> {
        let mut pois = Vec::new();
        let mut failed_indexers = Vec::new();

        for (indexer_id, indexer) in indexers {
            match poi_client
                .fetch_poi_with_retry(&indexer.url, &self.deployment, block, 3)
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

        let poi_groups = group_pois_by_hash(indexers, &pois, &self.correct_indexer_id);
        display_poi_groups(poi_groups, block, &self.correct_indexer_id);

        Ok(())
    }
}
