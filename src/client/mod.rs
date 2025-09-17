use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::task::JoinSet;

use crate::{client::poi::POIClient, models::Indexer};

pub mod eth;
pub mod ipfs;
pub mod poi;
pub mod registry;
pub mod subgraph;
pub mod update;

pub async fn check_divergence_at_block(
    poi_client: POIClient,
    indexers: &BTreeMap<String, Indexer>,
    deployment: &str,
    block: u32,
    correct_indexer_id: &str,
    max_retries: u32,
) -> Result<(bool, Vec<String>)> {
    let correct_indexer = indexers
        .get(correct_indexer_id)
        .ok_or_else(|| anyhow!("Correct indexer not found in active allocations"))?;

    let correct_poi = poi_client
        .fetch_poi_with_retry(&correct_indexer.url, deployment, block, max_retries)
        .await?;

    let mut tasks = JoinSet::new();
    let poi_client = Arc::new(poi_client);

    for (indexer_id, indexer) in indexers.iter() {
        if indexer_id == correct_indexer_id {
            continue;
        }

        let id = indexer_id.clone();
        let url = indexer.url.clone();
        let deployment = deployment.to_string();
        let poi_client = Arc::clone(&poi_client);

        tasks.spawn(async move {
            let poi_result = poi_client
                .fetch_poi_with_retry(&url, &deployment, block, max_retries)
                .await;
            (id, poi_result)
        });
    }

    let mut diverged_indexers = Vec::new();
    while let Some(result) = tasks.join_next().await {
        if let Ok((id, Ok(poi))) = result {
            if poi != correct_poi {
                diverged_indexers.push(id);
            }
        }
    }

    Ok((!diverged_indexers.is_empty(), diverged_indexers))
}
