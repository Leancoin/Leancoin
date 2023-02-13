use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, MintTo, Transfer};

use crate::account::ContractState;
use crate::context::VestedWalletContext;
use crate::error::MyError;

use crate::{MINT_SEED, PROGRAM_ACCOUNT_SEED};

pub fn transfer_tokens<'a, 'b>(
    authority: AccountInfo<'a>,
    to: AccountInfo<'a>,
    program: AccountInfo<'a>,
    seed: &'b str,
    nonce: u8,
    amount: u64,
) -> Result<()> {
    let seeds = &[seed.as_bytes(), &[nonce]];
    let signer_seeds = &[&seeds[..]];

    let from = authority.to_account_info();
    let authority = authority.to_account_info();

    let cpi_accounts = Transfer {
        from,
        to,
        authority,
    };

    let cpi_ctx =
        CpiContext::new_with_signer(program.to_account_info(), cpi_accounts, signer_seeds);

    token::transfer(cpi_ctx, amount)
}

pub fn mint_tokens<'a>(
    mint: AccountInfo<'a>,
    to: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    program: AccountInfo<'a>,
    nonce: u8,
    amount: u64,
) -> Result<()> {
    let seeds = &[MINT_SEED.as_bytes(), &[nonce]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = MintTo {
        mint,
        to,
        authority,
    };

    let cpi_ctx = CpiContext::new_with_signer(program, cpi_accounts, signer_seeds);

    token::mint_to(cpi_ctx, amount)
}

pub fn burn_tokens<'a>(
    mint: AccountInfo<'a>,
    from: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    program: AccountInfo<'a>,
    nonce: u8,
    amount: u64,
) -> Result<()> {
    let seeds = &[PROGRAM_ACCOUNT_SEED.as_bytes(), &[nonce]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Burn {
        mint,
        from,
        authority,
    };

    let cpi_ctx = CpiContext::new_with_signer(program, cpi_accounts, signer_seeds);

    token::burn(cpi_ctx, amount)
}

pub fn valid_owner(state: &ContractState, signer: &AccountInfo) -> Result<()> {
    require!(signer.key.eq(&state.authority), MyError::Unauthorized);

    Ok(())
}

pub fn valid_signer(signer: &AccountInfo) -> Result<()> {
    require!(signer.is_signer, MyError::Unauthorized);

    Ok(())
}

pub fn ethereum_token_state_mapping_not_performed_yet(state: &ContractState) -> Result<()> {
    require!(
        !state.import_ethereum_token_state_already_performed,
        MyError::EthereumTokenStateMappingAlreadyPerformed
    );

    Ok(())
}

pub struct Timestamp {
    pub year: i64,
    pub month: i64,
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
}

pub fn parse_timestamp(timestamp: i64) -> Timestamp {
    let mut time = timestamp;
    let seconds = time % 60;
    time /= 60;
    let minutes = time % 60;
    time /= 60;
    let hours = time % 24;
    time /= 24;

    let mut days = time;
    let mut year = 1970;
    let mut month = 0;
    let month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    while days >= 365 {
        if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            if days >= 366 {
                days -= 366;
                year += 1;
                continue;
            }
        } else {
            days -= 365;
            year += 1;
        }
    }

    while month < 12 {
        let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let month_length = if month == 2 && is_leap {
            29
        } else {
            month_days[month]
        };

        if days < month_length {
            break;
        }
        days -= month_length;
        month += 1;
    }

    Timestamp {
        year,
        month: month as i64,
        days: days + 1,
        hours,
        minutes,
        seconds,
    }
}

pub fn calculate_month_difference(start: i64, end: i64) -> u64 {
    let start = parse_timestamp(start);
    let end = parse_timestamp(end);

    let months = (end.year - start.year) * 12 + (end.month - start.month);
    months.abs() as u64
}

pub fn calculate_unlocked_amount_partnership_wallet(
    vesting_start_account_balance: u64,
    months_since_vesting_start: u64,
) -> u64 {
    match months_since_vesting_start {
        0 => 0,
        1 => vesting_start_account_balance / 2,
        _ => vesting_start_account_balance
    }
}

pub fn calculate_unlocked_amount_marketing_wallet(
    vesting_start_account_balance: u64,
    months_since_vesting_start: u64,
) -> u64 {
    if months_since_vesting_start < 12 {
        return 0;
    }

    let amount_unlocked_after_one_year = (vesting_start_account_balance / 100) * 40;
    let amount_unlocked_every_month_after_one_year = (vesting_start_account_balance / 100) * 5;
    let amount_unlocked = amount_unlocked_after_one_year
        + ((months_since_vesting_start - 12) * amount_unlocked_every_month_after_one_year);

    amount_unlocked.min(vesting_start_account_balance)
}

pub fn calculate_unlocked_amount_community_wallet(
    vesting_start_account_balance: u64,
    months_since_vesting_start: u64,
) -> u64 {
    let amount_unlocked = vesting_start_account_balance / 40 * (months_since_vesting_start + 1);
    
    amount_unlocked.min(vesting_start_account_balance)
}

pub fn calculate_unlocked_amount_liquidity_wallet(
    vesting_start_account_balance: u64,
    months_since_vesting_start: u64,
) -> u64 {
    match months_since_vesting_start {
        months if months >= 12 => vesting_start_account_balance,
        _ => vesting_start_account_balance / 2
    }
}

pub fn withdraw_vested_tokens<'a, 'b, 'c, 'info, T>(
    ctx: Context<'a, 'b, 'c, 'info, T>,
    amount_to_withdraw: u64,
    amount_available_to_withdraw: u64,
) -> Result<()>
where
    T: VestedWalletContext<'info>
{
    require!(
        amount_to_withdraw <= amount_available_to_withdraw,
        MyError::NotEnoughTokens
    );

    transfer_tokens(
        ctx.accounts.vested_account().to_account_info(),
        ctx.accounts.deposit_wallet().to_account_info(),
        ctx.accounts.token_program().to_account_info(),
        ctx.accounts.vested_account_seed(),
        ctx.accounts.vested_account_nonce(),
        amount_to_withdraw
    )?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    impl PartialEq for Timestamp {
        fn eq(&self, other: &Self) -> bool {
            self.year == other.year
                && self.month == other.month
                && self.days == other.days
                && self.hours == other.hours
                && self.minutes == other.minutes
                && self.seconds == other.seconds
        }
    }

    impl std::fmt::Debug for Timestamp {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Timestamp")
                .field("year", &self.year)
                .field("month", &self.month)
                .field("days", &self.days)
                .field("hours", &self.hours)
                .field("minutes", &self.minutes)
                .field("seconds", &self.seconds)
                .finish()
        }
    }

    #[test_case( 0, Timestamp { year: 1970, month: 1, days: 1, hours: 0, minutes: 0, seconds: 0 }; "timestamp 0")]
    #[test_case( 162000, Timestamp { year: 1970, month: 1, days: 2, hours: 21, minutes: 0, seconds: 0 }; "timestamp 162000")]
    #[test_case( 1620000000, Timestamp { year: 2021, month: 5, days: 3, hours: 0, minutes: 0, seconds: 0 }; "timestamp 1620000000")]
    #[test_case( 1620002137, Timestamp { year: 2021, month: 5, days: 3, hours: 0, minutes: 35, seconds: 37 }; "timestamp 1620002137")]
    #[test_case( 1378183924, Timestamp { year: 2013, month: 9, days: 3, hours: 4, minutes: 52, seconds: 4 }; "timestamp 1378183924")]
    #[test_case( 959249016, Timestamp { year: 2000, month: 5, days: 25, hours: 10, minutes: 3, seconds: 36 }; "timestamp 959249016")]
    #[test_case( 1336937134, Timestamp { year: 2012, month: 5, days: 13, hours: 19, minutes: 25, seconds: 34 }; "timestamp 1336937134")]
    #[test_case( 1836183646,  Timestamp { year: 2028, month: 3, days: 9, hours: 3, minutes: 0, seconds: 46 }; "timestamp 1836183646")]
    fn test_parse_timestamp(timestamp: i64, expected: Timestamp) {
        let parsed_timestamp = parse_timestamp(timestamp);
        assert_eq!(parsed_timestamp, expected);
    }

    #[test_case( 1620000000, 1620000000, 0; "same month")]
    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 31, 1; "1 month")]
    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 31 * 2, 2; "2 months")]
    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 31 * 3, 3; "3 months")]
    #[test_case( 1620000000, 1620000000 + 60 * 60 * 24 * 31 * 12, 12; "12 months")]
    fn test_calculate_month_difference(start: i64, end: i64, expected: u64) {
        let months_since_vesting_start = calculate_month_difference(start, end);
        assert_eq!(months_since_vesting_start, expected);
    }

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
    fn test_calculate_unlocked_amount_marketing_wallet(
        vesting_start_account_balance: u64,
        months_since_vesting_start: u64,
        expected: u64,
    ) {
        let amount_unlocked = calculate_unlocked_amount_marketing_wallet(
            vesting_start_account_balance,
            months_since_vesting_start,
        );
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
    #[test_case(1000000000, 100, 1000000000; "100 months")]
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
}
