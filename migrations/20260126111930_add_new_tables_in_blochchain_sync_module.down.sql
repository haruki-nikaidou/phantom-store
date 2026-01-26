DROP INDEX IF EXISTS "blockchain"."idx_erc20_pending_user_address";
DROP INDEX IF EXISTS "blockchain"."idx_erc20_pending_wallet_address";
DROP INDEX IF EXISTS "blockchain"."idx_erc20_pending_chain";
DROP INDEX IF EXISTS "blockchain"."idx_erc20_pending_token_name";

DROP INDEX IF EXISTS "blockchain"."idx_erc20_transfer_block_number";
DROP INDEX IF EXISTS "blockchain"."idx_erc20_transfer_to_address";
DROP INDEX IF EXISTS "blockchain"."idx_erc20_transfer_from_address";
DROP INDEX IF EXISTS "blockchain"."idx_erc20_transfer_chain";
DROP INDEX IF EXISTS "blockchain"."idx_erc20_transfer_token_name";

DROP INDEX IF EXISTS "blockchain"."idx_trc20_pending_user_address";
DROP INDEX IF EXISTS "blockchain"."idx_trc20_pending_wallet_address";
DROP INDEX IF EXISTS "blockchain"."idx_trc20_pending_token_name";

DROP INDEX IF EXISTS "blockchain"."idx_trc20_transfer_confirmed";
DROP INDEX IF EXISTS "blockchain"."idx_trc20_transfer_block_number";
DROP INDEX IF EXISTS "blockchain"."idx_trc20_transfer_to_address";
DROP INDEX IF EXISTS "blockchain"."idx_trc20_transfer_from_address";
DROP INDEX IF EXISTS "blockchain"."idx_trc20_transfer_token_name";

DROP INDEX IF EXISTS "blockchain"."idx_customer_addresses_address";
DROP INDEX IF EXISTS "blockchain"."idx_customer_addresses_chain";
DROP INDEX IF EXISTS "blockchain"."idx_customer_addresses_user_id";

DROP INDEX IF EXISTS "blockchain"."idx_merchant_wallet_address_active";
DROP INDEX IF EXISTS "blockchain"."idx_merchant_wallet_address_chain";

DROP TABLE IF EXISTS "blockchain"."erc20_stablecoin_pending_deposit";
DROP TABLE IF EXISTS "blockchain"."erc20_stablecoin_token_transfer";
DROP TABLE IF EXISTS "blockchain"."trc20_stablecoin_pending_deposit";
DROP TABLE IF EXISTS "blockchain"."trc20_stable_coin_token_transfer";
DROP TABLE IF EXISTS "blockchain"."customer_addresses";
DROP TABLE IF EXISTS "blockchain"."merchant_wallet_address";

DROP TYPE IF EXISTS "blockchain"."stable_coin_name";
DROP TYPE IF EXISTS "blockchain"."etherscan_chain";
DROP TYPE IF EXISTS "blockchain"."supported_blockchains";

DROP SCHEMA IF EXISTS blockchain;
