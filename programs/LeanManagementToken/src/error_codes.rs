use anchor_lang::prelude::error_code;

/// The enum defining all errors used by the contract.
#[error_code]
pub enum LeancoinError {
    #[msg("You are not an owner")]
    Unauthorized = 0,
    #[msg("End time must be later than start time")]
    EndTimeMustBeLaterThanStartTime = 1,
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
    #[msg("Account info must be unique")]
    NonUniqueAccountInfo = 7,
    #[msg("Wallet name must be unique")]
    DuplicatedWalletName = 8,
    #[msg("Program account balance is not zero")]
    ProgramAccountBalanceIsNotZero = 9,
    #[msg("Community wallet balance is zero")]
    CommunityWalletBalanceIsZero = 10,
    #[msg("Partnership wallet balance is zero")]
    PartnershipWalletBalanceIsZero = 11,
    #[msg("Marketing wallet balance is zero")]
    MarketingWalletBalanceIsZero = 12,
    #[msg("Liquidity wallet balance is zero")]
    LiquidityWalletBalanceIsZero = 13,
    #[msg("Cannot convert to i64")]
    CannotConvertToI64 = 14,
    #[msg("Cannot convert to u8")]
    CannotConvertToU8 = 15,
    #[msg("Invalid timestamp")]
    InvalidTimestamp = 16,
    #[msg("Cannot convert to u128")]
    CannotConvertToU128 = 17,
    #[msg("Cannot convert to u64")]
    CannotConvertToU64 = 18,
}
