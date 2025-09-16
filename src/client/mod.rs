use anyhow::{anyhow, Result};
use futures::future::join_all;
use std::collections::BTreeMap;

use crate::{client::poi::POIClient, models::Indexer};

pub mod eth;
pub mod ipfs;
pub mod poi;
pub mod registry;
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

    // Create futures for all indexers (except the correct one)
    let fetch_futures: Vec<_> = indexers
        .iter()
        .filter(|(id, _)| *id != correct_indexer_id)
        .map(|(indexer_id, indexer)| {
            let id = indexer_id.clone();
            let url = indexer.url.clone();
            let deployment = deployment.to_string();

            async move {
                let poi_result = poi_client
                    .fetch_poi_with_retry(&url, &deployment, block, 3)
                    .await;
                (id, poi_result)
            }
        })
        .collect();

    // Execute all futures concurrently
    let results = join_all(fetch_futures).await;

    // Process results to find diverged indexers
    let diverged_indexers: Vec<String> = results
        .into_iter()
        .filter_map(|(id, poi_result)| match poi_result {
            Ok(poi) if poi != correct_poi => Some(id),
            _ => None,
        })
        .collect();

    Ok((!diverged_indexers.is_empty(), diverged_indexers))
}
