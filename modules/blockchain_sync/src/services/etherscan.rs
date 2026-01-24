use crate::utils::supported_tokens::{BlockchainSyncError, StableCoin, SupportedBlockchains};
use compact_str::CompactString;
use kanau::processor::Processor;
use std::sync::Arc;
use tokio::sync::RwLock;

const ETHERSCAN_API_URL: &str = "https://api.etherscan.io/v2/api";

#[derive(Clone)]
pub struct EtherScanApiService {
    pub client: reqwest::Client,
    pub api_key: Arc<RwLock<CompactString>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(rename_all = "lowercase", type_name = "blockchain.etherscan_chain")]
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

impl serde::Serialize for EtherScanChain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&(*self as i64).to_string())
    }
}

impl<'de> serde::Deserialize<'de> for EtherScanChain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let value: i64 = s.parse().map_err(serde::de::Error::custom)?;
        match value {
            1 => Ok(EtherScanChain::Ethereum),
            137 => Ok(EtherScanChain::Polygon),
            8453 => Ok(EtherScanChain::Base),
            42161 => Ok(EtherScanChain::ArbitrumOne),
            59144 => Ok(EtherScanChain::Linea),
            10 => Ok(EtherScanChain::Optimism),
            43114 => Ok(EtherScanChain::AvalancheC),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["1", "137", "8453", "42161", "59144", "10", "43114"],
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchErc20TokenTransfers {
    pub chain: EtherScanChain,
    pub stable_coin: StableCoin,
    pub address: CompactString,
    pub start_block: u64,
    pub end_block: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc20TokenTransferResponseItem {
    pub block_number: String,
    pub time_stamp: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub token_decimal: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct EtherScanResponse<T> {
    status: String,
    message: String,
    result: T,
}

impl Processor<FetchErc20TokenTransfers> for EtherScanApiService {
    type Output = Vec<Erc20TokenTransferResponseItem>;
    type Error = BlockchainSyncError;
    #[tracing::instrument(skip_all, err)]
    async fn process(
        &self,
        input: FetchErc20TokenTransfers,
    ) -> Result<Vec<Erc20TokenTransferResponseItem>, BlockchainSyncError> {
        let Some(contract_address) = input
            .stable_coin
            .get_contract_address(SupportedBlockchains::EtherScan(input.chain))
        else {
            return Err(BlockchainSyncError::UnsupportedBlockchain(
                SupportedBlockchains::EtherScan(input.chain),
            ));
        };
        let chain_id = input.chain as i64;
        let response = self
            .client
            .get(ETHERSCAN_API_URL)
            .query(&[
                ("apiKey", self.api_key.read().await.as_str()),
                ("chainid", chain_id.to_string().as_str()),
                ("module", "account"),
                ("action", "tokentx"),
                ("contractaddress", contract_address),
                ("address", input.address.as_str()),
                ("startblock", input.start_block.to_string().as_str()),
                ("endblock", input.end_block.to_string().as_str()),
                ("page", "1"),
                ("offset", "100"),
                ("sort", "asc"),
            ])
            .send()
            .await?;
        let response: EtherScanResponse<Vec<Erc20TokenTransferResponseItem>> =
            response.json().await?;
        if response.status != "1" {
            return Err(BlockchainSyncError::EtherScanError(response.message));
        }
        Ok(response.result)
    }
}
