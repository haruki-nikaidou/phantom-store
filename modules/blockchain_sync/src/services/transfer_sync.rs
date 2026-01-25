use crate::services::etherscan::EtherScanApiService;
use crate::services::tronscan::TronScanApiService;
use framework::redis::RedisConnection;
use framework::sqlx::DatabaseProcessor;

#[derive(Clone)]
pub struct BlockchainTransferSyncService {
    pub db: DatabaseProcessor,
    pub redis: RedisConnection,
    pub etherscan: EtherScanApiService,
    pub tronscan: TronScanApiService,
}
