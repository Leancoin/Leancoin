use anchor_lang::prelude::*;

#[error_code]
pub enum MyError {
    #[msg("You are not an owner")]
    Unauthorized = 0,
    #[msg("Ethereum token state mapping already performed")]
    EthereumTokenStateMappingAlreadyPerformed = 1,
    #[msg("Account from remaining accounts not found in user info")]
    MismatchBetweenRemainingAccountsAndUserInfo = 2,
    #[msg("Tokens can be burned only between the 1st and the 5th day of the month")]
    TooLateToBurnTokens = 3,
    #[msg("Tokens already burned this month.")]
    TokensAlreadyBurned = 4,
    #[msg("Not enough tokens to withdraw")]
    NotEnoughTokens = 5,
    #[msg("User duplicated in user info")]
    UserDuplicatedInUserInfo = 6,
    #[msg("Program account balance is not zero")]
    ProgramAccountBalanceIsNotZero = 7,
    #[msg("Community wallet balance is zero")]
    CommunityWalletBalanceIsZero = 8,
    #[msg("Partnership wallet balance is zero")]
    PartnershipWalletBalanceIsZero = 9,
    #[msg("Marketing wallet balance is zero")]
    MarketingWalletBalanceIsZero = 10,
    #[msg("Liquidity wallet balance is zero")]
    LiquidityWalletBalanceIsZero = 11,
}