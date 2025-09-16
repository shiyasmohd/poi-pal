use anyhow::{anyhow, Result};
use graph_networks_registry::NetworksRegistry;

pub struct RegistryClient {
    registry: NetworksRegistry,
}

impl RegistryClient {
    pub async fn new() -> Result<Self> {
        let registry = NetworksRegistry::from_latest_version().await?;
        Ok(Self { registry })
    }

    pub async fn get_public_rpc_url(&self, network: &str) -> Result<String> {
        let network_info = self
            .registry
            .get_network_by_graph_id(network)
            .ok_or_else(|| anyhow!("Network '{}' not found in registry", network))?;

        let rpc_urls = network_info
            .rpc_urls
            .as_ref()
            .ok_or_else(|| anyhow!("No RPC URLs available for network '{}'", network))?;

        if rpc_urls.is_empty() {
            return Err(anyhow!("RPC URLs list is empty for network '{}'", network));
        }

        Ok(rpc_urls[0].clone())
    }
}
