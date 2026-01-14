use compact_str::CompactString;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod erc20_transfers;
pub mod chains;
mod tokens;

#[derive(Clone)]
pub struct EtherScanApiService {
    pub client: reqwest::Client,
    pub api_key: Arc<RwLock<CompactString>>,
}
