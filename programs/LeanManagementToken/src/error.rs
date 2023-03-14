use anchor_lang::prelude::*;

/// The enum defining all errors used by the contract.
#[error_code]
pub enum LeancoinError {
    #[msg("You are not an owner")]
    Unauthorized = 0,
    #[msg("Cannot get the bump.")]
    CannotGetBump = 1,
    #[msg("Ethereum token state mapping already performed")]
    EthereumTokenStateMappingAlreadyPerformed = 2,
    #[msg("Account from remaining accounts not found in user info")]
    MismatchBetweenRemainingAccountsAndUserInfo = 3,
    #[msg("Tokens can be burned only between the 1st and the 5th day of the month")]
    TooLateToBurnTokens = 4,
    #[msg("Tokens already burned this month.")]
    TokensAlreadyBurned = 5,
    #[msg("Not enough tokens to withdraw")]
    NotEnoughTokens = 6,
    #[msg("User duplicated in user info")]
    UserDuplicatedInUserInfo = 7,
    #[msg("Program account balance is not zero")]
    ProgramAccountBalanceIsNotZero = 8,
    #[msg("Community wallet balance is zero")]
    CommunityWalletBalanceIsZero = 9,
    #[msg("Partnership wallet balance is zero")]
    PartnershipWalletBalanceIsZero = 10,
    #[msg("Marketing wallet balance is zero")]
    MarketingWalletBalanceIsZero = 11,
    #[msg("Liquidity wallet balance is zero")]
    LiquidityWalletBalanceIsZero = 12,
    #[msg("Cannot convert to i64")]
    CannotConvertToI64 = 13,
    #[msg("Cannot convert to u8")]
    CannotConvertToU8 = 14,
    #[msg("Invalid timestamp")]
    InvalidTimestamp = 15,
    #[msg("Cannot convert to u128")]
    CannotConvertToU128 = 16,
    #[msg("Cannot convert to u64")]
    CannotConvertToU64 = 17,
}
