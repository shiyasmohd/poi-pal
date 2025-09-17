<div align="center" style="font-family:'Montserrat', sans-serif;">

## 🔍 PoiPal
🚀 &nbsp;Blazing fast CLI tool for Proof of Indexing (POI) Investigations on The Graph
<br/>
<br/>
[![Crates.io](https://img.shields.io/crates/v/poipal?style=flat-square)](https://crates.io/crates/poipal)
[![Crates.io](https://img.shields.io/crates/d/poipal?style=flat-square)](https://crates.io/crates/poipal)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE-MIT)
[![Contributors](https://img.shields.io/github/contributors/shiyasmohd/poi-pal?style=flat-square)](https://github.com/shiyasmohd/poi-pal/graphs/contributors)

</div>

A powerful command-line interface for fetching and analyzing Proof of Indexing (POI) data from The Graph Network. PoiPal helps indexers and developers verify POI consistency, detect divergences, and troubleshoot indexing issues with beautiful, table-formatted output.

## ✨ Features

- 🔄 **Multi-threaded POI fetching** for blazing-fast performance
- 📊 **Table-formatted output** grouped by POI hash
- 🔍 **Binary search divergence detection** to find exact divergence points
- 🤖 **Automatic block detection** from IPFS manifests and chain head

## Prerequisites 🛠️
- Rust - [Install Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)
- The Graph API Key (for network subgraph access). You can get it from [The Graph Studio](https://thegraph.com/studio/)

## Installation 💻
```bash
cargo install poipal
```

## Usage

PoiPal provides two main commands:

### 1. POI Command - Fetch POIs for a specific block

```bash
poipal poi --deployment <DEPLOYMENT_ID> --block <BLOCK_NUMBER> --api-key <GRAPH_API_KEY>
```

**Example:**
```bash
poipal poi \
  --deployment QmdKXcBUHR3UyURqVRQHu1oV6VUkBrhi2vNvMx3bNDnUCc \
  --block 370000000 \
  --api-key your_graph_api_key_here
```

### 2. Check Divergence Command - Find POI divergences using binary search

```bash
poipal check-divergence \
  --deployment <DEPLOYMENT_ID> \
  --start-block <START_BLOCK> \
  --end-block <END_BLOCK> \
  --indexer <TRUSTED_INDEXER_ID> \
  --api-key <GRAPH_API_KEY>
```

**Example with all parameters:**
```bash
poipal check-divergence \
  --deployment QmdKXcBUHR3UyURqVRQHu1oV6VUkBrhi2vNvMx3bNDnUCc \
  --start-block 370000000 \
  --end-block 371000000 \
  --indexer 0xbdfb5ee5a2abf4fc7bb1bd1221067aef7f9de491 \
  --api-key your_graph_api_key_here
```

**Example with auto-detection (recommended):**
```bash
# Auto-detects start block from IPFS manifest and end block from chain head
poipal check-divergence \
  --deployment QmdKXcBUHR3UyURqVRQHu1oV6VUkBrhi2vNvMx3bNDnUCc \
  --indexer 0xbdfb5ee5a2abf4fc7bb1bd1221067aef7f9de491 \
  --api-key your_graph_api_key_here
```

## Environment Variables 🔧

Set these environment variables to avoid passing them as CLI arguments:

```bash
# Required: The Graph API key
export GRAPH_API_KEY="your_graph_api_key_here"

# Optional: Trusted indexer for divergence checking
export TRUSTED_INDEXER="0xbdfb5ee5a2abf4fc7bb1bd1221067aef7f9de491"
```

## Command Options

### POI Command Options
| Option | Description | Required | Environment Variable |
|--------|-------------|----------|---------------------|
| `--deployment` | Deployment ID (IPFS hash) | ✅ | - |
| `--block` | Block number to fetch POI for | ✅ | - |
| `--api-key` | The Graph API key | ✅ | `GRAPH_API_KEY` |

### Check Divergence Options
| Option | Description | Required | Default | Environment Variable |
|--------|-------------|----------|---------|---------------------|
| `--deployment` | Deployment ID (IPFS hash) | ✅ | - | - |
| `--start-block` | Start block for binary search | ❌ | Auto-detect from IPFS | - |
| `--end-block` | End block for binary search | ❌ | Auto-detect from chain | - |
| `--indexer` | Trusted indexer ID | ✅ | - | `TRUSTED_INDEXER` |
| `--api-key` | The Graph API key | ✅ | - | `GRAPH_API_KEY` |
| `--ipfs-url` | IPFS gateway URL | ❌ | `https://ipfs.thegraph.com` | - |
| `--max-retries` | Max retries for POI fetching | ❌ | `3` | - |

## Example Output

### POI Command Output
```
================================================================================
POIs for deployment QmYHgc1xGgnGvTx3sxy8FVf7jh4WGiJwS9WfKKDynLCy7 at block 19000000
================================================================================

Found 5 indexers with 1 unique POI(s)

════════════════════════════════════════════════════════════════════════════════
POI Hash: 0xf0642535812254bb5ec91283a1ec2714546c4dbe199157812f175303c35c6925
Count: 5 indexer(s)
────────────────────────────────────────────────────────────────────────────────
 Indexer ID                                  │ URL
────────────────────────────────────────────────────────────────────────────────
 0x63c9dc729ba7a22bb8605216b24a34b902e5fe94  │ https://production-indexer.infradao.tech
 0x7bb834017672b1135466661d8dd69c5dd0b3bf51  │ https://graphprodl2.0xcryptovestor.com
 0x9082f497bdc512d08ffde50d6fce28e72c2addcf  │ https://indexer.holographic.network/
 0xedca8740873152ff30a2696add66d1ab41882beb  │ https://arbitrum.graph.pinax.network/
 0xf92f430dd8567b0d466358c79594ab58d919a6d4  │ https://graph-l2prod.ellipfra.com/
════════════════════════════════════════════════════════════════════════════════
```

### Check Divergence Output
```
================================================================================
POI Divergence Checker
================================================================================
Deployment: QmYHgc1xGgnGvTx3sxy8FVf7jh4WGiJwS9WfKKDynLCy7

Fetching start block from IPFS...
✓ Fetched start block: 18500000

Fetching network from manifest...
Network: arbitrum-one
Fetching RPC URL from registry...
RPC URL: https://arbitrum-one.publicnode.com
Fetching chain head block...
✓ Fetched end block: 19250000

Search Range: 18500000 → 19250000
Reference Indexer: 0x63c9dc729ba7a22bb8605216b24a34b902e5fe94

Fetching active indexers...
✓ Found 5 active indexers

Starting binary search for diverged block...
────────────────────────────────────────────────────────────────
→ Checking block 18875000 (range: 18500000 - 19250000)... ✓ All POIs match
→ Checking block 19062500 (range: 18875001 - 19250000)... ✗ Divergence found (2 indexers)
→ Checking block 18968750 (range: 18875001 - 19062499)... ✓ All POIs match

✗ Divergence found at block 19062500
```

## How to Run Locally 🏠

1. **Clone repository & change directory**
```bash
git clone https://github.com/shiyasmohd/poipal.git
cd poipal
```

2. **Set up environment variables**
```bash
export GRAPH_API_KEY="your_graph_api_key_here"
export TRUSTED_INDEXER="0x63c9dc729ba7a22bb8605216b24a34b902e5fe94"
```

3. **Run the program**
```bash
# Fetch POIs for a specific block
cargo run -- poi \
  --deployment QmYHgc1xGgnGvTx3sxy8FVf7jh4WGiJwS9WfKKDynLCy7 \
  --block 19000000

# Check for divergences
cargo run -- check-divergence \
  --deployment QmYHgc1xGgnGvTx3sxy8FVf7jh4WGiJwS9WfKKDynLCy7
```

## How to Get Your Graph API Key 🔑

1. Visit [The Graph Studio](https://thegraph.com/studio/)
2. Connect your wallet and create an account
3. Go to your dashboard and find the "API Keys" section
4. Create a new API key for network subgraph access
5. Set it as an environment variable: `export GRAPH_API_KEY="your_key_here"`

## Contributing 🤝

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License 📝

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support 💬

If you have any questions or need help, please open an issue on GitHub or reach out to the maintainers.

---

<div align="center">
Made with ❤️ by <a href="https://x.com/0xshiyasmohd">shiyasmohd</a>
</div>
