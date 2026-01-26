CREATE SCHEMA IF NOT EXISTS blockchain;

CREATE TYPE "blockchain"."supported_blockchains" AS ENUM (
    'ethereum',
    'polygon',
    'base',
    'arbitrumone',
    'linea',
    'optimism',
    'avalanchec',
    'tron'
);

CREATE TYPE "blockchain"."etherscan_chain" AS ENUM (
    'ethereum',
    'polygon',
    'base',
    'arbitrumone',
    'linea',
    'optimism',
    'avalanchec'
);

CREATE TYPE "blockchain"."stable_coin_name" AS ENUM (
    'USDT',
    'USDC',
    'DAI'
);

CREATE TABLE IF NOT EXISTS "blockchain"."merchant_wallet_address"
(
    id                   BIGSERIAL PRIMARY KEY,
    address              VARCHAR(255)                            NOT NULL UNIQUE,
    chain                "blockchain"."supported_blockchains"    NOT NULL,
    enabled_stable_coins "blockchain"."stable_coin_name"[]       NOT NULL DEFAULT '{}',
    active               BOOLEAN                                 NOT NULL DEFAULT TRUE
);

CREATE INDEX IF NOT EXISTS idx_merchant_wallet_address_chain ON "blockchain"."merchant_wallet_address" (chain);
CREATE INDEX IF NOT EXISTS idx_merchant_wallet_address_active ON "blockchain"."merchant_wallet_address" (active);

CREATE TABLE IF NOT EXISTS "blockchain"."customer_addresses"
(
    id       BIGSERIAL PRIMARY KEY,
    user_id  UUID                                   NOT NULL REFERENCES "auth"."user_account" (id) ON DELETE CASCADE,
    chain    "blockchain"."supported_blockchains"  NOT NULL,
    address  VARCHAR(255)                          NOT NULL,
    UNIQUE (user_id, chain, address)
);

CREATE INDEX IF NOT EXISTS idx_customer_addresses_user_id ON "blockchain"."customer_addresses" (user_id);
CREATE INDEX IF NOT EXISTS idx_customer_addresses_chain ON "blockchain"."customer_addresses" (chain);
CREATE INDEX IF NOT EXISTS idx_customer_addresses_address ON "blockchain"."customer_addresses" (address);

CREATE TABLE IF NOT EXISTS "blockchain"."trc20_stable_coin_token_transfer"
(
    id              BIGSERIAL PRIMARY KEY,
    token_name      "blockchain"."stable_coin_name"  NOT NULL,
    from_address    VARCHAR(255)                     NOT NULL,
    to_address      VARCHAR(255)                     NOT NULL,
    txn_hash        VARCHAR(255)                     NOT NULL UNIQUE,
    value           DECIMAL(38, 18)                  NOT NULL,
    block_number    BIGINT                           NOT NULL,
    block_timestamp TIMESTAMP                        NOT NULL,
    confirmed       BOOLEAN                          NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_trc20_transfer_token_name ON "blockchain"."trc20_stable_coin_token_transfer" (token_name);
CREATE INDEX IF NOT EXISTS idx_trc20_transfer_from_address ON "blockchain"."trc20_stable_coin_token_transfer" (from_address);
CREATE INDEX IF NOT EXISTS idx_trc20_transfer_to_address ON "blockchain"."trc20_stable_coin_token_transfer" (to_address);
CREATE INDEX IF NOT EXISTS idx_trc20_transfer_block_number ON "blockchain"."trc20_stable_coin_token_transfer" (block_number);
CREATE INDEX IF NOT EXISTS idx_trc20_transfer_confirmed ON "blockchain"."trc20_stable_coin_token_transfer" (confirmed);

CREATE TABLE IF NOT EXISTS "blockchain"."trc20_stablecoin_pending_deposit"
(
    id               BIGSERIAL PRIMARY KEY,
    token_name       "blockchain"."stable_coin_name"  NOT NULL,
    user_address     VARCHAR(255),
    wallet_address   VARCHAR(255)                     NOT NULL,
    value            DECIMAL(38, 18)                  NOT NULL,
    started_at       TIMESTAMP                        NOT NULL DEFAULT NOW(),
    last_scanned_at  TIMESTAMP                        NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trc20_pending_token_name ON "blockchain"."trc20_stablecoin_pending_deposit" (token_name);
CREATE INDEX IF NOT EXISTS idx_trc20_pending_wallet_address ON "blockchain"."trc20_stablecoin_pending_deposit" (wallet_address);
CREATE INDEX IF NOT EXISTS idx_trc20_pending_user_address ON "blockchain"."trc20_stablecoin_pending_deposit" (user_address);

CREATE TABLE IF NOT EXISTS "blockchain"."erc20_stablecoin_token_transfer"
(
    id              BIGSERIAL PRIMARY KEY,
    token_name      "blockchain"."stable_coin_name"  NOT NULL,
    chain           "blockchain"."etherscan_chain"   NOT NULL,
    from_address    VARCHAR(255)                     NOT NULL,
    to_address      VARCHAR(255)                     NOT NULL,
    txn_hash        VARCHAR(255)                     NOT NULL,
    value           DECIMAL(38, 18)                  NOT NULL,
    block_number    BIGINT                           NOT NULL,
    block_timestamp TIMESTAMP                        NOT NULL,
    UNIQUE (chain, txn_hash)
);

CREATE INDEX IF NOT EXISTS idx_erc20_transfer_token_name ON "blockchain"."erc20_stablecoin_token_transfer" (token_name);
CREATE INDEX IF NOT EXISTS idx_erc20_transfer_chain ON "blockchain"."erc20_stablecoin_token_transfer" (chain);
CREATE INDEX IF NOT EXISTS idx_erc20_transfer_from_address ON "blockchain"."erc20_stablecoin_token_transfer" (from_address);
CREATE INDEX IF NOT EXISTS idx_erc20_transfer_to_address ON "blockchain"."erc20_stablecoin_token_transfer" (to_address);
CREATE INDEX IF NOT EXISTS idx_erc20_transfer_block_number ON "blockchain"."erc20_stablecoin_token_transfer" (block_number);

CREATE TABLE IF NOT EXISTS "blockchain"."erc20_stablecoin_pending_deposit"
(
    id               BIGSERIAL PRIMARY KEY,
    token_name       "blockchain"."stable_coin_name"  NOT NULL,
    chain            "blockchain"."etherscan_chain"   NOT NULL,
    user_address     VARCHAR(255),
    wallet_address   VARCHAR(255)                     NOT NULL,
    value            DECIMAL(38, 18)                  NOT NULL,
    started_at       TIMESTAMP                        NOT NULL DEFAULT NOW(),
    last_scanned_at  TIMESTAMP                        NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_erc20_pending_token_name ON "blockchain"."erc20_stablecoin_pending_deposit" (token_name);
CREATE INDEX IF NOT EXISTS idx_erc20_pending_chain ON "blockchain"."erc20_stablecoin_pending_deposit" (chain);
CREATE INDEX IF NOT EXISTS idx_erc20_pending_wallet_address ON "blockchain"."erc20_stablecoin_pending_deposit" (wallet_address);
CREATE INDEX IF NOT EXISTS idx_erc20_pending_user_address ON "blockchain"."erc20_stablecoin_pending_deposit" (user_address);
