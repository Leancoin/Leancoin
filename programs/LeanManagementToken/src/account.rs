use anchor_lang::{
    prelude::{account, borsh, AnchorDeserialize, AnchorSerialize, InitSpace},
    solana_program::pubkey::Pubkey,
};

/// The account that holds the state of the contract.
/// It is initialized only once during contract initialization.
/// Part of the state is never updated (nonces and authority) while the other parts can be updated one or more times.
///
/// It is used to store the following data:
/// - information if the Ethereum token state import has already been performed,
/// - contract state nonce,
/// - the mint nonce,
/// - the program account nonce,
/// - the burning account nonce,
/// - the last burning month and year,
/// - the authority which is set to the signer of the transaction when contract is initialized so the signer becomes contract's owner.
#[account]
#[derive(InitSpace)]
pub struct ContractState {
    pub import_ethereum_token_state_already_performed: bool,

    pub contract_state_nonce: u8,
    pub mint_nonce: u8,
    pub program_account_nonce: u8,
    pub burning_account_nonce: u8,

    pub last_burning_month: u8,
    pub last_burning_year: i64,

    pub authority: Pubkey,
}

/// The account that holds the state of the vesting.
/// It is initialized only once during contract initialization.
/// The state is updated only once after the initialization - during Ethereum token state import.
///
/// It is used to store the following data:
/// - vesting state nonce,
/// - the community wallet nonce,
/// - the community wallet initial balance after Ethereum token state import,
/// - the partnership wallet nonce,
/// - the partnership wallet initial balance after Ethereum token state import,
/// - the marketing wallet nonce,
/// - the marketing wallet initial balance after Ethereum token state import,
/// - the liquidity wallet nonce,
/// - the liquidity wallet initial balance after Ethereum token state import,
/// - the vesting start timestamp which is used to calculate the amount of unlocked tokens for each wallet, it is set to the timestamp of Ethereum token state import.
#[account]
#[derive(InitSpace)]
pub struct VestingState {
    pub vesting_state_nonce: u8,

    pub community_wallet_nonce: u8,
    pub initial_community_wallet_balance: u64,
    pub already_withdrawn_community_wallet_amount: u64,

    pub partnership_wallet_nonce: u8,
    pub initial_partnership_wallet_balance: u64,
    pub already_withdrawn_partnership_wallet_amount: u64,

    pub marketing_wallet_nonce: u8,
    pub initial_marketing_wallet_balance: u64,
    pub already_withdrawn_marketing_wallet_amount: u64,

    pub liquidity_wallet_nonce: u8,
    pub initial_liquidity_wallet_balance: u64,
    pub already_withdrawn_liquidity_wallet_amount: u64,

    pub start_timestamp: i64,
}
