use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphQLQuery {
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct AllocationsData {
    pub allocations: Vec<Allocation>,
}

#[derive(Debug, Deserialize)]
pub struct Allocation {
    pub indexer: Indexer,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Indexer {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct POIResponse {
    pub data: POIData,
}

#[derive(Debug, Deserialize)]
pub struct POIData {
    #[serde(rename = "publicProofsOfIndexing")]
    pub pois: Vec<POI>,
}

#[derive(Debug, Deserialize)]
pub struct POI {
    pub deployment: Option<String>,
    #[serde(rename = "proofOfIndexing")]
    pub poi: String,
    pub block: Option<Block>,
}

#[derive(Debug, Deserialize)]
pub struct Block {
    pub number: String,
}

#[derive(Debug)]
pub struct IndexerPOI {
    pub indexer_id: String,
    pub indexer_url: String,
    pub poi: String,
}

#[derive(Debug)]
pub struct POIGroup {
    pub poi: String,
    pub indexers: BTreeMap<String, String>,
    pub is_correct: bool,
}
