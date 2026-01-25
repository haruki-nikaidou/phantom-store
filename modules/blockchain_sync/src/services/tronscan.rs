use crate::utils::supported_tokens::{BlockchainSyncError, StableCoin, SupportedBlockchains};
use compact_str::CompactString;
use kanau::processor::Processor;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct TronScanApiService {
    pub client: reqwest::Client,
    pub api_key: Arc<RwLock<CompactString>>,
}

const TRONSCAN_API_URL: &str = "https://apilist.tronscanapi.com/api/token_trc20/transfers";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchTronTokenTransfers {
    pub stable_coin: StableCoin,
    pub address: CompactString,
    pub index_start: u64,
    pub limit: u64,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub filter_token_value: rust_decimal::Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trc20TokenTransferResponseItem {
    pub block: u64,
    pub hash: String,
    pub timestamp: u64,
    pub owner_address: String,
    pub to_address: String,
    pub confirmed: bool,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TronScanResponse {
    data: Vec<Trc20TokenTransferResponseItem>,
    total: u64,
    range_total: u64,
}

impl Processor<FetchTronTokenTransfers> for TronScanApiService {
    type Output = Vec<Trc20TokenTransferResponseItem>;
    type Error = BlockchainSyncError;
    #[tracing::instrument(skip_all, err)]
    async fn process(
        &self,
        input: FetchTronTokenTransfers,
    ) -> Result<Vec<Trc20TokenTransferResponseItem>, BlockchainSyncError> {
        let Some(contract_address) = input
            .stable_coin
            .get_contract_address(SupportedBlockchains::Tron)
        else {
            return Err(BlockchainSyncError::UnsupportedBlockchain(
                SupportedBlockchains::Tron,
            ));
        };
        let response = self
            .client
            .get(TRONSCAN_API_URL)
            .query(&[
                ("limit", input.limit.to_string().as_str()),
                ("start", input.index_start.to_string().as_str()),
                ("contract_address", contract_address),
                (
                    "start_timestamp",
                    input.start_timestamp.to_string().as_str(),
                ),
                ("end_timestamp", input.end_timestamp.to_string().as_str()),
                // including unconfirmed transactions
                ("confirm", "false"),
                (
                    "filterTokenValue",
                    input.filter_token_value.to_string().as_str(),
                ),
                ("toAddress", input.address.as_str()),
            ])
            .header("TRON-PRO-API-KEY", self.api_key.read().await.as_str())
            .send()
            .await?;
        let response: TronScanResponse = response.json().await?;
        Ok(response.data)
    }
}
