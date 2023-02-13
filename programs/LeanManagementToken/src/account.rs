use anchor_lang::prelude::*;

#[account]
pub struct ContractState {
    pub mint_nonce: u8,
    pub import_ethereum_token_state_already_performed: bool,
    pub program_account_nonce: u8,
    pub burning_account_nonce: u8,
    pub last_burning_month: u8,
    pub last_burning_year: u8,
    pub authority: Pubkey,
}

#[account]
pub struct VestingState {
    pub community_wallet_nonce: u8,
    pub initial_community_wallet_balance: u64,

    pub partnership_wallet_nonce: u8,
    pub initial_partnership_wallet_balance: u64,

    pub marketing_wallet_nonce: u8,
    pub initial_marketing_wallet_balance: u64,

    pub liquidity_wallet_nonce: u8,
    pub initial_liquidity_wallet_balance: u64,

    pub start_timestamp: i64,
}
