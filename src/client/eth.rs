use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    result: Option<BlockResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BlockResult {
    number: String,
}

pub struct EthClient {
    client: Client,
    url: String,
}

impl EthClient {
    pub fn new(url: String) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

        Ok(Self { client, url })
    }

    pub async fn get_chain_head_block_number(&self) -> Result<u32> {
        let body = json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": ["latest", false],
            "id": 1
        });
        
        let response = self.client.post(&self.url).json(&body).send().await?;
        let json_response: JsonRpcResponse = response.json().await?;
        
        let block = json_response
            .result
            .ok_or_else(|| anyhow!("No result in JSON-RPC response"))?;
        
        // Parse hex string (e.g., "0x1234") to u32
        let block_number = if block.number.starts_with("0x") {
            u32::from_str_radix(&block.number[2..], 16)?
        } else {
            block.number.parse::<u32>()?
        };
        
        Ok(block_number)
    }
}
