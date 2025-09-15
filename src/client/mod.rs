use anyhow::{anyhow, Result};
use std::collections::BTreeMap;

use crate::{client::poi::POIClient, models::Indexer};

pub mod ipfs;
pub mod poi;
pub mod subgraph;

pub async fn check_divergence_at_block(
    poi_client: &POIClient,
    indexers: &BTreeMap<String, Indexer>,
    deployment: &str,
    block: u32,
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
