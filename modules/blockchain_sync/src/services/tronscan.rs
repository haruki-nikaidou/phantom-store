use crate::utils::supported_tokens::StableCoin;
use compact_str::CompactString;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TronScanApiService {
    pub client: reqwest::Client,
    pub api_key: Arc<RwLock<CompactString>>,
}

const TRONSCAN_API_URL: &str = "https://apilist.tronscanapi.com/api/token_trc20/transfers";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchTronTokenTransfers {
    pub stable_coin: StableCoin,
    pub address: CompactString,
    pub start_block: u64,
    pub end_block: u64,
}
