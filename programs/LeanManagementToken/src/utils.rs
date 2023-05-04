use anchor_lang::prelude::{require, AccountInfo, Context, CpiContext, Result, ToAccountInfo};
use anchor_spl::token::{self, Burn, MintTo, Transfer};

use crate::account::ContractState;
use crate::context::VestedWalletContext;
use crate::error_codes::LeancoinError;

use crate::{MINT_SEED, PROGRAM_ACCOUNT_SEED};

/// DAYS_PER_MONTH is an array of integers that contains the number of days for each month, excluding December
const DAYS_PER_MONTH: [i64; 11] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30];

/// Transfers tokens between two accounts.
///
/// ### Arguments
///
/// * `authority` - the authority that is going to transfer the tokens, it also the source account
/// * `to` - the destination account
/// * `program_account` - the program account
/// * `program_account_seed` - the seed of the program account
/// * `program_account_nonce` - the nonce of the program account
/// * `amount` - the amount of tokens to transfer
///
/// ### Returns
/// The result of the transfer
pub fn transfer_tokens<'a>(
    authority: AccountInfo<'a>,
    to: AccountInfo<'a>,
    program_account: AccountInfo<'a>,
    program_account_seed: &str,
    program_account_nonce: u8,
    amount: u64,
) -> Result<()> {
    let seeds = &[program_account_seed.as_bytes(), &[program_account_nonce]];
    let signer_seeds = &[&seeds[..]];

    let from = authority.to_account_info();
    let authority = authority.to_account_info();

    let cpi_accounts = Transfer {
        from,
        to,
        authority,
    };

    let cpi_ctx = CpiContext::new_with_signer(
        program_account.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );

    token::transfer(cpi_ctx, amount)
}

/// Mints tokens to given account.
///
/// ### Arguments
///
/// * `mint` - the mint account
/// * `to` - the destination account
/// * `authority` - the authority that is used to mint the tokens
/// * `program_account` - the program account
/// * `mint_nonce` - the nonce of the mint account
/// * `amount` - the amount of tokens to transfer
///
/// ### Returns
/// The result of the minting
pub fn mint_tokens<'a>(
    mint: AccountInfo<'a>,
    to: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    program_account: AccountInfo<'a>,
    mint_nonce: u8,
    amount: u64,
) -> Result<()> {
    let seeds = &[MINT_SEED.as_bytes(), &[mint_nonce]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = MintTo {
        mint,
        to,
        authority,
    };

    let cpi_ctx = CpiContext::new_with_signer(program_account, cpi_accounts, signer_seeds);

    token::mint_to(cpi_ctx, amount)
}

/// Removes tokens from given account by burning them.
///
/// ### Arguments
///
/// * `mint` - the authority that was used to mint the tokens
/// * `from` - the account holding the tokens to burn
/// * `authority` - the authority that is used to burn the tokens
/// * `program_account` - the program account
/// * `program_account_nonce` - the nonce of the program account
/// * `amount` - the amount of tokens to transfer
///
/// ### Returns
/// The result of the burning
pub fn burn_tokens<'a>(
    mint: AccountInfo<'a>,
    from: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    program_account: AccountInfo<'a>,
    program_account_nonce: u8,
    amount: u64,
) -> Result<()> {
    let seeds = &[PROGRAM_ACCOUNT_SEED.as_bytes(), &[program_account_nonce]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Burn {
        mint,
        from,
        authority,
    };

    let cpi_ctx = CpiContext::new_with_signer(program_account, cpi_accounts, signer_seeds);

    token::burn(cpi_ctx, amount)
}

/// Asserts that the signer is authorized to perform the action, i.e. if the signer is contract's owner.
///
/// ### Arguments
///
/// * `state` - the current state of the contract
/// * `signer` - the account which is the signer of the current transaction
///
/// ### Returns
/// An error if the signer is not an owner of the contract, otherwise a successful result.
pub fn valid_owner(state: &ContractState, signer: &AccountInfo) -> Result<()> {
    require!(signer.key.eq(&state.authority), LeancoinError::Unauthorized);

    Ok(())
}

/// Asserts that the given account is a signer.
///
/// ### Arguments
///
/// * `signer` - the account which is supposed to be a signer
///
/// ### Returns
/// An error if the account is not a signer, otherwise a successful result.
pub fn valid_signer(signer: &AccountInfo) -> Result<()> {
    require!(signer.is_signer, LeancoinError::Unauthorized);

    Ok(())
}

/// Asserts that the import of Ethereum token state has not yet been performed.
///
/// ### Arguments
///
/// * `state` - the current state of the contract
///
/// ### Returns
/// An error if the import has already been performed, otherwise a successful result.
pub fn ethereum_token_state_mapping_not_performed_yet(state: &ContractState) -> Result<()> {
    require!(
        !state.import_ethereum_token_state_already_performed,
        LeancoinError::EthereumTokenStateMappingAlreadyPerformed
    );

    Ok(())
}

/// Date time struct for the timestamp parsing
pub struct DateTime {
    pub year: i64,
    pub month: u8,
    pub days: u8,
}

/// Accepts the timestamp as an integer (i64) and returns DateTime struct
///
/// ### Arguments
///
/// * `timestamp` - the timestamp as a signed integer
///
/// ### Returns
/// DateTime struct created from the timestamp
pub fn parse_timestamp(timestamp: i64) -> Result<DateTime> {
    require!(timestamp >= 0, LeancoinError::InvalidTimestamp);

    let mut remaining_days = timestamp / (60 * 60 * 24);
    let mut year = 1970;
    let mut month = 1;

    while remaining_days >= 365 {
        if is_leap_year(year) {
            if remaining_days >= 366 {
                remaining_days -= 366;
                year += 1;
            } else {
                break;
            }
        } else {
            remaining_days -= 365;
            year += 1;
        }
    }

    let leap_year = is_leap_year(year);
    while month < 12 {
        let month_length = if month == 2 && leap_year {
            29
        } else {
            DAYS_PER_MONTH[month - 1]
        };

        if remaining_days < month_length {
            break;
        }
        remaining_days -= month_length;
        month += 1;
    }
    remaining_days += 1;

    let month: u8 = month.try_into().unwrap();
    let days: u8 = remaining_days.try_into().unwrap();

    Ok(DateTime { year, month, days })
}

/// Checks whether the given year is a leap year.
///
/// A year is a leap year if it is divisible by 4 and not by 100, or if it is
/// divisible by 400. For example, 1992 and 1996 are leap years, but 1900 is not.
///
/// ### Arguments
///
/// * `year` - the year to be checked
///
/// ### Returns
/// True if the provided year is a leap year, false otherwise.
fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Calculates the number of months between two timestamps.
/// Only month numbers are compared, days are ignored.
///
/// Examples:
/// - when start date is 01/04/2023 and end date is 01/05/2023, then the result is 1
/// - when start date is 27/04/2023 and end date is 01/05/2023, then the result is 1
/// - when start date is 01/04/2023 and end date is 01/06/2023, then the result is 2
/// - when start date is 27/04/2023 and end date is 01/06/2023, then the result is 2
/// - when start date is 27/04/2023 and end date is 01/12/2023, then the result is 8
/// - when start date is 27/04/2023 and end date is 01/04/2024, then the result is 12
///
/// ### Arguments
///
/// * `start` - the earlier timestamp
/// * `end` - the later timestamp
///
/// ### Returns
/// Number of months between two timestamps, ignoring days.
pub fn calculate_month_difference(start: i64, end: i64) -> Result<u64> {
    require!(end >= start, LeancoinError::EndTimeMustBeLaterThanStartTime);
    let start = parse_timestamp(start)?;
    let end = parse_timestamp(end)?;

    let end_month: i64 = end.month.try_into().unwrap();
    let start_month: i64 = start.month.try_into().unwrap();

    let month_difference = end_month - start_month;
    let months = (end.year - start.year) * 12 + month_difference;
    let months = months.try_into().unwrap();

    Ok(months)
}

/// Calculates the amount of unlocked tokens for the partnership wallet.
/// 50% of the initial wallet's balance is unlocked after 1 month.
/// The remaining part is unlocked after 2 months.
///
/// ### Arguments
///
/// * `vesting_start_account_balance` - the initial balance of the partnership wallet after Ethereum token state import
/// * `months_since_vesting_start` - number of full months since the Ethereum token state import
///
/// ### Returns
/// The amount of unlocked tokens for partnership wallet
pub fn calculate_unlocked_amount_partnership_wallet(
    vesting_start_account_balance: u64,
    months_since_vesting_start: u64,
) -> u64 {
    match months_since_vesting_start {
        0 => 0,
        1 => vesting_start_account_balance / 2,
        _ => vesting_start_account_balance,
    }
}

/// Calculates the amount of unlocked tokens for the marketing wallet.
/// 40% of the initial wallet's balance is unlocked after 1 year.
/// Starting from the 13th month, 5% of the initial wallet's balance is unlocked every month.
///
/// If the calculated amount is below 1, then 1 is returned as the calculated amount.
///
/// ### Arguments
///
/// * `vesting_start_account_balance` - the initial balance of the marketing wallet after Ethereum token state import
/// * `months_since_vesting_start` - number of full months since the Ethereum token state import
///
/// ### Returns
/// The amount of unlocked tokens for marketing wallet but not less than 1
pub fn calculate_unlocked_amount_marketing_wallet(
    vesting_start_account_balance: u64,
    months_since_vesting_start: u64,
) -> Result<u64> {
    if months_since_vesting_start < 12 {
        return Ok(0);
    }

    let (vesting_start_account_balance, months_since_vesting_start) = (
        u128::from(vesting_start_account_balance),
        u128::from(months_since_vesting_start),
    );

    let amount_unlocked = (vesting_start_account_balance * 40
        + (months_since_vesting_start - 12) * (vesting_start_account_balance * 5))
        / 100;

    Ok(u64::try_from(amount_unlocked.max(1).min(vesting_start_account_balance)).unwrap())
}

/// Calculates the amount of unlocked tokens for the community wallet.
/// 2.5% of the initial wallet's balance is unlocked immediately.
/// Additional 2.5% of the initial wallet's balance is unlocked every month.
/// So after 2 months: 7.5% of the initial balance is unlocked, after 3 months: 10%, after 4 months: 12.5% etc.
///
/// If the calculated amount is below 1, then 1 is returned as the calculated amount.
///
/// ### Arguments
///
/// * `vesting_start_account_balance` - the initial balance of the community wallet after Ethereum token state import
/// * `months_since_vesting_start` - number of full months since the Ethereum token state import
///
/// ### Returns
/// The amount of unlocked tokens for community wallet but not less than 1
pub fn calculate_unlocked_amount_community_wallet(
    vesting_start_account_balance: u64,
    months_since_vesting_start: u64,
) -> u64 {
    let (vesting_start_account_balance, months_since_vesting_start) = (
        u128::from(vesting_start_account_balance),
        u128::from(months_since_vesting_start),
    );
    let amount_unlocked = vesting_start_account_balance * (months_since_vesting_start + 1) / 40;

    u64::try_from(amount_unlocked.max(1).min(vesting_start_account_balance)).unwrap()
}

/// Calculates the amount of unlocked tokens for the liquidity wallet.
/// 50% of the initial wallet's balance is unlocked immediately.
/// The remaining part is unlocked after 1 year.
///
/// ### Arguments
///
/// * `vesting_start_account_balance` - the initial balance of the liquidity wallet after Ethereum token state import
/// * `months_since_vesting_start` - number of full months since the Ethereum token state import
///
/// ### Returns
/// The amount of unlocked tokens for liquidity wallet
pub fn calculate_unlocked_amount_liquidity_wallet(
    vesting_start_account_balance: u64,
    months_since_vesting_start: u64,
) -> u64 {
    match months_since_vesting_start {
        months if months >= 12 => vesting_start_account_balance,
        _ => vesting_start_account_balance / 2,
    }
}

/// Transfers tokens from one of the wallets affected by vesting mechanism: community, partnership, marketing or liquidity wallet.
/// The destination for the transfer is deposit wallet which is not managed by this contract.
///
/// The function also validates if the amount of tokens to withdraw is not greater than amount of already unlocked tokens.
/// It does not calculate the amount of unlocked tokens but instead it accepts the amount as an input parameter.
/// Hence, the amount of unlocked tokens should be calculated and validated before this function is invoked.
///
/// ### Arguments
///
/// * `ctx` - the program's context
/// * `amount_to_withdraw` - the amount of tokens to withdraw
/// * `amount_available_to_withdraw` - the amount of tokens available to withdraw from the source wallet
///
/// ### Returns
/// Tokens transfer result
pub fn withdraw_vested_tokens<'a, 'b, 'c, 'info, T>(
    ctx: Context<'a, 'b, 'c, 'info, T>,
    amount_to_withdraw: u64,
    amount_available_to_withdraw: u64,
) -> Result<()>
where
    T: VestedWalletContext<'info>,
{
    require!(
        amount_to_withdraw <= amount_available_to_withdraw,
        LeancoinError::NotEnoughTokens
    );

    transfer_tokens(
        ctx.accounts.vested_account().to_account_info(),
        ctx.accounts.deposit_wallet().to_account_info(),
        ctx.accounts.token_program().to_account_info(),
        ctx.accounts.vested_account_seed(),
        ctx.accounts.vested_account_nonce(),
        amount_to_withdraw,
    )?;

    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;
    use anchor_lang::prelude::Pubkey;
    use std::cell::RefCell;
    use std::rc::Rc;
    use test_case::test_case;

    impl PartialEq for DateTime {
        fn eq(&self, other: &Self) -> bool {
            self.year == other.year && self.month == other.month && self.days == other.days
        }
    }

    impl std::fmt::Debug for DateTime {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("DateTime")
                .field("year", &self.year)
                .field("month", &self.month)
                .field("days", &self.days)
                .finish()
        }
    }

    impl std::fmt::Debug for ContractState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ContractState")
                .field("mint_nonce", &self.mint_nonce)
                .field(
                    "import_ethereum_token_state_already_performed",
                    &self.import_ethereum_token_state_already_performed,
                )
                .field("program_account_nonce", &self.program_account_nonce)
                .field("burning_account_nonce", &self.burning_account_nonce)
                .field("last_burning_month", &self.last_burning_month)
                .field("last_burning_year", &self.last_burning_year)
                .field("authority", &self.authority)
                .finish()
        }
    }

    impl ContractState {
        pub fn default() -> Self {
            Self {
                contract_state_nonce: 0,
                mint_nonce: 0,
                import_ethereum_token_state_already_performed: false,
                program_account_nonce: 0,
                burning_account_nonce: 0,
                last_burning_month: 0,
                last_burning_year: 0,
                authority: Pubkey::new_unique(),
            }
        }
    }

    #[test_case( 0, DateTime { year: 1970, month: 1, days: 1 }; "timestamp 0")]
    #[test_case( 162000, DateTime { year: 1970, month: 1, days: 2 }; "timestamp 162000")]
    #[test_case( 94694400, DateTime { year: 1973, month: 1, days: 1 }; "timestamp 94694400")]
    #[test_case( 2678400, DateTime { year: 1970, month: 2, days: 1 }; "timestamp 2678400")]
    #[test_case( 5097600, DateTime { year: 1970, month: 3, days: 1 }; "timestamp 5097600")]
    #[test_case( 68256000, DateTime { year: 1972, month: 3, days: 1 }; "timestamp 68256000")]
    #[test_case( 31449600, DateTime { year: 1970, month: 12, days: 31 }; "timestamp 31449600")]
    #[test_case( 220838400, DateTime { year: 1976, month: 12, days: 31 }; "timestamp 220838400")]
    #[test_case( 978220800, DateTime { year: 2000, month: 12, days: 31 }; "timestamp 978220800")]
    #[test_case( 1609335304, DateTime { year: 2020, month: 12, days: 30 }; "timestamp 1609335304" )]
    #[test_case( 1620000000, DateTime { year: 2021, month: 5, days: 3 }; "timestamp 1620000000")]
    #[test_case( 1620002137, DateTime { year: 2021, month: 5, days: 3 }; "timestamp 1620002137")]
    #[test_case( 1378183924, DateTime { year: 2013, month: 9, days: 3 }; "timestamp 1378183924")]
    #[test_case( 959249016, DateTime { year: 2000, month: 5, days: 25 }; "timestamp 959249016")]
    #[test_case( 1336937134, DateTime { year: 2012, month: 5, days: 13 }; "timestamp 1336937134")]
    #[test_case( 1836183646,  DateTime { year: 2028, month: 3, days: 9 }; "timestamp 1836183646")]
    #[test_case( 1641052800,  DateTime { year: 2022, month: 1, days: 1 }; "timestamp 1641052800")]
    fn test_parse_timestamp(timestamp: i64, expected: DateTime) {
        let parsed_timestamp = parse_timestamp(timestamp).unwrap();
        assert_eq!(parsed_timestamp, expected);
    }

    #[test]
    fn test_parse_timestamp_error() {
        let parsed_timestamp = parse_timestamp(-1);
        assert!(parsed_timestamp.is_err());
    }

    #[test]
    #[should_panic]
    fn test_parse_timestamp_negative() {
        let timestamp: i64 = -1;
        parse_timestamp(timestamp).unwrap();
    }

    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 15, 0; "start = 03/05/21, end = 18/05/21, same month")]
    #[test_case( 1620000000, 1620000000, 0; "start = 03/05/21, end = 03/05/21, same month")]
    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 31 - (2 * 24 * 60 * 60), 1; "start = 03/05/21, end = 01/06/21, 1 month")]
    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 31, 1; "start = 03/05/21, end = 03/06/21, 1 month")]
    #[test_case( 1620000000 + 60 * 60 * 24 * 15, 1620000000 + 60 * 60 * 24 * 30, 1; "start = 18/05/21, end = 02/06/21, 1 month")]
    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 31 * 2, 2; "start = 03/05/21, end = 04/07/21, 2 months")]
    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 31 * 3, 3; "start = 03/05/21, end = 04/08/21, 3 months")]
    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 31 * 11, 11; "start = 03/05/21, end = 09/04/22, 11 months")]
    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 31 * 12, 12; "start = 03/05/21, end = 10/05/22, 12 months")]
    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 31 * 13, 13; "start = 03/05/21, end = 10/06/22, 13 months")]

    fn test_calculate_month_difference(start: i64, end: i64, expected: u64) {
        let months_since_vesting_start = calculate_month_difference(start, end).unwrap();
        assert_eq!(months_since_vesting_start, expected);
    }

    #[test_case(1000000000, 0, 0; "0 months")]
    #[test_case(1000000000, 1, 500000000; "1 month")]
    #[test_case(1000000000, 2, 1000000000; "2 months")]
    #[test_case(1000000000, 3, 1000000000; "3 months")]
    fn test_calculate_unlocked_amount_partnership_wallet(
        vesting_start_account_balance: u64,
        months_since_vesting_start: u64,
        expected: u64,
    ) {
        let amount_unlocked = calculate_unlocked_amount_partnership_wallet(
            vesting_start_account_balance,
            months_since_vesting_start,
        );
        assert_eq!(amount_unlocked, expected);
    }

    #[test_case(1000000000, 1, 0; "1 month")]
    #[test_case(1000000000, 2, 0; "2 months")]
    #[test_case(1000000000, 3, 0; "3 months")]
    #[test_case(1000000000, 12, 400000000; "12 months")]
    #[test_case(1000000000, 13, 450000000; "13 months")]
    #[test_case(1000000000, 50, 1000000000; "50 months")]
    #[test_case(1000000000, 100, 1000000000; "100 months")]
    #[test_case(0, 11, 0; "11 months with 0 tokens - no unlocked tokens")]
    #[test_case(0, 12, 0; "12 months with 0 tokens - no unlocked tokens")]
    #[test_case(0, 13, 0; "13 months with 0 tokens - no unlocked tokens")]
    #[test_case(0, 50, 0; "50 months with 0 tokens - no unlocked tokens")]
    #[test_case(0, 100, 0; "100 months with 0 tokens - no unlocked tokens")]
    #[test_case(1, 11, 0; "11 months with 1 token - no unlocked tokens")]
    #[test_case(1, 12, 1; "12 months with 1 token - one token unlocked")]
    #[test_case(1, 13, 1; "13 months with 1 token - one token unlocked")]
    #[test_case(1, 50, 1; "50 months with 1 token - one token unlocked")]
    #[test_case(1, 100, 1; "100 months with 1 token - one token unlocked")]
    fn test_calculate_unlocked_amount_marketing_wallet(
        vesting_start_account_balance: u64,
        months_since_vesting_start: u64,
        expected: u64,
    ) {
        let amount_unlocked = calculate_unlocked_amount_marketing_wallet(
            vesting_start_account_balance,
            months_since_vesting_start,
        )
        .unwrap();
        assert_eq!(amount_unlocked, expected);
    }

    #[test_case(1000000000, 1, 50000000; "1 month")]
    #[test_case(1000000000, 2, 75000000; "2 months")]
    #[test_case(1000000000, 3, 100000000; "3 months")]
    #[test_case(1000000000, 4, 125000000; "4 months")]
    #[test_case(1000000000, 5, 150000000; "5 months")]
    #[test_case(1000000000, 11, 300000000; "11 months")]
    #[test_case(1000000000, 12, 325000000; "12 months")]
    #[test_case(1000000000, 13, 350000000; "13 months")]
    #[test_case(1000000000, 38, 975000000; "38 months")]
    #[test_case(1000000000, 39, 1000000000; "39 months")]
    #[test_case(1000000000, 40, 1000000000; "40 months")]
    #[test_case(1000000000, 100, 1000000000; "100 months")]
    #[test_case(0, 1, 0; "1 month with 0 tokens - no unlocked tokens")]
    #[test_case(0, 38, 0; "38 months with 0 tokens - no unlocked tokens")]
    #[test_case(0, 39, 0; "39 months with 0 tokens - no unlocked tokens")]
    #[test_case(0, 100, 0; "100 months with 0 tokens - no unlocked tokens")]
    #[test_case(1, 1, 1; "1 month with 1 token - one token unlocked")]
    #[test_case(1, 38, 1; "38 months with 1 token - one token unlocked")]
    #[test_case(1, 39, 1; "39 months with 1 token - one token unlocked")]
    #[test_case(1, 100, 1; "100 months with 1 token - one token unlocked")]
    #[test_case(1000000000000000000, 100, 1000000000000000000; "100 months with 1000000000000000000 token - 1000000000000000000 token unlocked")]
    fn test_calculate_unlocked_amount_community_wallet(
        vesting_start_account_balance: u64,
        months_since_vesting_start: u64,
        expected: u64,
    ) {
        let amount_unlocked = calculate_unlocked_amount_community_wallet(
            vesting_start_account_balance,
            months_since_vesting_start,
        );
        assert_eq!(amount_unlocked, expected);
    }

    #[test_case(1000000000, 1, 500000000; "1 month")]
    #[test_case(1000000000, 2, 500000000; "2 months")]
    #[test_case(1000000000, 3, 500000000; "3 months")]
    #[test_case(1000000000, 4, 500000000; "4 months")]
    #[test_case(1000000000, 5, 500000000; "5 months")]
    #[test_case(1000000000, 11, 500000000; "11 months")]
    #[test_case(1000000000, 12, 1000000000; "12 months")]
    #[test_case(1000000000, 13, 1000000000; "13 months")]
    #[test_case(1000000000, 100, 1000000000; "100 months")]

    fn test_calculate_unlocked_amount_liquidity_wallet(
        vesting_start_account_balance: u64,
        months_since_vesting_start: u64,
        expected: u64,
    ) {
        let amount_unlocked = calculate_unlocked_amount_liquidity_wallet(
            vesting_start_account_balance,
            months_since_vesting_start,
        );
        assert_eq!(amount_unlocked, expected);
    }

    #[test]
    fn test_ethereum_token_state_mapping_not_performed_yet() {
        let state = ContractState {
            import_ethereum_token_state_already_performed: false,
            ..ContractState::default()
        };
        ethereum_token_state_mapping_not_performed_yet(&state).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_ethereum_token_state_mapping_not_performed_yet() {
        let state = ContractState {
            import_ethereum_token_state_already_performed: true,
            ..ContractState::default()
        };
        ethereum_token_state_mapping_not_performed_yet(&state).unwrap();
    }

    #[test]
    fn test_valid_signer() {
        let data: Rc<RefCell<&mut [u8]>> = Rc::new(RefCell::new(&mut [0u8; 0]));
        let mut binding = 0u64;
        let deps = AccountInfo {
            key: &Pubkey::new_unique(),
            is_signer: true,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding)),
            data,
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };

        valid_signer(&deps).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_valid_signer() {
        let data: Rc<RefCell<&mut [u8]>> = Rc::new(RefCell::new(&mut [0u8; 0]));
        let mut binding = 0u64;
        let deps = AccountInfo {
            key: &Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding)),
            data,
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };

        valid_signer(&deps).unwrap();
    }

    #[test]
    fn test_valid_owner() {
        let data: Rc<RefCell<&mut [u8]>> = Rc::new(RefCell::new(&mut [0u8; 0]));
        let authority = Pubkey::new_unique();
        let mut binding = 0u64;

        let signer = AccountInfo {
            key: &authority,
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding)),
            data,
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };
        let state = ContractState {
            authority,
            ..ContractState::default()
        };

        valid_owner(&state, &signer).unwrap()
    }

    #[test]
    #[should_panic]
    fn test_fail_valid_owner() {
        let data: Rc<RefCell<&mut [u8]>> = Rc::new(RefCell::new(&mut [0u8; 0]));
        let authority = Pubkey::new_unique();
        let mut binding = 0u64;

        let signer = AccountInfo {
            key: &authority,
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding)),
            data,
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };
        let state = ContractState {
            authority: Pubkey::new_unique(),
            ..ContractState::default()
        };

        valid_owner(&state, &signer).unwrap()
    }
}
