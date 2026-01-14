use compact_str::CompactString;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct EtherScanApiService {
    pub client: reqwest::Client,
    pub api_key: Arc<RwLock<CompactString>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
/// https://docs.etherscan.io/supported-chains
pub enum EtherScanChain {
    Ethereum = 1,
    Polygon = 137,
    Base = 8453,
    ArbitrumOne = 42161,
    Linea = 59144,
    Optimism = 10,
    AvalancheC = 43114,
}
