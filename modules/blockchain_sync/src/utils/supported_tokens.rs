use crate::services::etherscan::EtherScanChain;

#[derive(Debug, thiserror::Error)]
pub enum BlockchainSyncError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Unsupported blockchain: {0:?}")]
    UnsupportedBlockchain(SupportedBlockchains),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedBlockchains {
    EtherScan(EtherScanChain),
    Tron,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "blockchain.stable_coin_name", rename_all = "UPPERCASE")]
pub enum StableCoinName {
    USDT,
    USDC,
    DAI,
}

impl StableCoinName {
    pub fn info(&self) -> StableCoin {
        match self {
            StableCoinName::USDT => USDT,
            StableCoinName::USDC => USDC,
            StableCoinName::DAI => DAI,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StableCoin {
    name: StableCoinName,
    contract_addresses: &'static [(SupportedBlockchains, &'static str)],
}

impl StableCoin {
    pub fn get_contract_address(&self, on_chain: SupportedBlockchains) -> Option<&'static str> {
        self.contract_addresses
            .iter()
            .find(|(chain, _)| chain == &on_chain)
            .map(|(_, addr)| *addr)
    }
}

pub const USDT: StableCoin = StableCoin {
    name: StableCoinName::USDT,
    contract_addresses: &[
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Ethereum),
            "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        ),
        (
            SupportedBlockchains::Tron,
            "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Polygon),
            "0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7",
        ),
    ],
};

pub const USDC: StableCoin = StableCoin {
    name: StableCoinName::USDC,
    contract_addresses: &[
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Ethereum),
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::AvalancheC),
            "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::ArbitrumOne),
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Polygon),
            "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Optimism),
            "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Base),
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Linea),
            "0x176211869cA2b568f2A7D4EE941E073a821EE1ff",
        ),
    ],
};

pub const DAI: StableCoin = StableCoin {
    name: StableCoinName::DAI,
    contract_addresses: &[
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Ethereum),
            "0x6B175474E89094C44Da98b954EedeAC495271d0F",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::AvalancheC),
            "0xbA7dEebBFC5fA1100Fb055a87773e1E99Cd3507a",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::ArbitrumOne),
            "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Polygon),
            "0x82E64f49Ed5EC1bC6e43DAD4FC8Af9bb3A2312EE",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Optimism),
            "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Base),
            "0x50c5725949A6F0c72E6C4a641F24049A917DB0Cb",
        ),
        (
            SupportedBlockchains::EtherScan(EtherScanChain::Linea),
            "0x4AF15ec2A0BD43Db75dd04E62FAA3B8EF36b00d5",
        ),
    ],
};
