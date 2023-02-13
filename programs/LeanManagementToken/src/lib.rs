use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use anchor_spl::token::{self, Burn};

use context::*;
use error::*;
use utils::*;

mod account;
mod context;
mod error;
mod utils;

pub const MINT_SEED: &str = "mint";
pub const PROGRAM_ACCOUNT_SEED: &str = "program_account";
pub const BURNING_ACCOUNT_SEED: &str = "burning_account";

const CONTRACT_STATE_SEED: &str = "contract_state";
const VESTING_STATE_SEED: &str = "vesting_state";

const COMMUNITY_ACCOUNT_SEED: &str = "community_account";
const PARTNERSHIP_ACCOUNT_SEED: &str = "partnership_account";
const MARKETING_ACCOUNT_SEED: &str = "marketing_account";
const LIQUIDITY_ACCOUNT_SEED: &str = "liquidity_account";

declare_id!("BA8o4sePXLFZqjLrXaTWaeD3MZBrWYYV1DNVbQdwNbKj");

#[program]
pub mod leancoin {
    use super::*;

    pub fn initialize(
        ctx: Context<InitializeContext>,
        mint_nonce: u8,
        program_account_nonce: u8,
        burning_account_nonce: u8,
        community_wallet_nonce: u8,
        liquidity_wallet_nonce: u8,
        marketing_wallet_nonce: u8,
        partnership_wallet_nonce: u8,
    ) -> Result<()> {
        let contract_state = &mut ctx.accounts.contract_state;
        let vesting_state = &mut ctx.accounts.vesting_state;

        contract_state.authority = ctx.accounts.signer.key();
        contract_state.mint_nonce = mint_nonce;
        contract_state.import_ethereum_token_state_already_performed = false;
        contract_state.program_account_nonce = program_account_nonce;
        contract_state.burning_account_nonce = burning_account_nonce;
        contract_state.last_burning_month = 0;
        contract_state.last_burning_year = 0;

        vesting_state.start_timestamp = 0;
        vesting_state.initial_community_wallet_balance = 0;
        vesting_state.initial_partnership_wallet_balance = 0;
        vesting_state.initial_marketing_wallet_balance = 0;
        vesting_state.initial_liquidity_wallet_balance = 0;

        vesting_state.community_wallet_nonce = community_wallet_nonce;
        vesting_state.liquidity_wallet_nonce = liquidity_wallet_nonce;
        vesting_state.marketing_wallet_nonce = marketing_wallet_nonce;
        vesting_state.partnership_wallet_nonce = partnership_wallet_nonce;

        Ok(())
    }

    #[access_control(valid_owner(&ctx.accounts.contract_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer) ethereum_token_state_mapping_not_performed_yet(&ctx.accounts.contract_state))]
    pub fn import_ethereum_token_state<'info>(
        ctx: Context<'_, '_, '_, 'info, ImportEthereumTokenStateContext<'info>>,
        account_info_from_ethereum: Vec<AccountInfoFromEthereum>,
        amount_token_to_mint: u64,
        amount_token_to_burn: u64,
    ) -> Result<()> {
        let contract_state = &mut ctx.accounts.contract_state;
        let vesting_state = &mut ctx.accounts.vesting_state;

        let mint_nonce = contract_state.mint_nonce;
        let program_account_nonce = contract_state.program_account_nonce;
        let timestamp = clock::Clock::get()?.unix_timestamp;

        vesting_state.start_timestamp = timestamp;

        mint_tokens(
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.program_account.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            mint_nonce,
            amount_token_to_mint,
        )?;

        burn_tokens(
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.program_account.to_account_info(),
            ctx.accounts.program_account.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            program_account_nonce,
            amount_token_to_burn,
        )?;

        for account in ctx.remaining_accounts.iter() {
            let matching_users = account_info_from_ethereum
                .iter()
                .filter(|user_info| user_info.account_public_key == account.key())
                .collect::<Vec<&AccountInfoFromEthereum>>();

            require!(matching_users.len() <= 1, MyError::UserDuplicatedInUserInfo);

            let user_info = match matching_users.first() {
                Some(user) => user,
                None => return err!(MyError::MismatchBetweenRemainingAccountsAndUserInfo),
            };

            transfer_tokens(
                ctx.accounts.program_account.to_account_info(),
                account.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                PROGRAM_ACCOUNT_SEED,
                program_account_nonce,
                user_info.account_balance,
            )?;

            match user_info.wallet_name.as_str() {
                "community" => vesting_state.initial_community_wallet_balance = user_info.account_balance,
                "partnership" => {
                    vesting_state.initial_partnership_wallet_balance = user_info.account_balance
                }
                "marketing" => vesting_state.initial_marketing_wallet_balance = user_info.account_balance,
                "liquidity" => vesting_state.initial_liquidity_wallet_balance = user_info.account_balance,
                _ => {}
            }
        }

        require!(ctx.accounts.program_account.amount == 0, MyError::ProgramAccountBalanceIsNotZero);
        require!(vesting_state.initial_community_wallet_balance != 0, MyError::CommunityWalletBalanceIsZero);
        require!(vesting_state.initial_partnership_wallet_balance != 0, MyError::PartnershipWalletBalanceIsZero);
        require!(vesting_state.initial_marketing_wallet_balance != 0, MyError::MarketingWalletBalanceIsZero);
        require!(vesting_state.initial_liquidity_wallet_balance != 0, MyError::LiquidityWalletBalanceIsZero); 

        contract_state.import_ethereum_token_state_already_performed = true;

        Ok(())
    }

    pub fn burn(ctx: Context<BurnContext>) -> Result<()> {
        let contract_state = &mut ctx.accounts.contract_state;
        let timestamp = clock::Clock::get()?.unix_timestamp;
        let now = parse_timestamp(timestamp);

        require!(now.days as u8 <= 5, MyError::TooLateToBurnTokens);
        require!(
            contract_state.last_burning_month != now.month as u8
                || contract_state.last_burning_year != now.year as u8,
            MyError::TokensAlreadyBurned
        );

        let seeds = &[
            BURNING_ACCOUNT_SEED.as_bytes(),
            &[contract_state.burning_account_nonce],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.burning_account.to_account_info(),
            authority: ctx.accounts.burning_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        let amount = token::accessor::amount(&ctx.accounts.burning_account.to_account_info())? / 20;

        token::burn(cpi_ctx, amount)?;

        contract_state.last_burning_month = now.month as u8;
        contract_state.last_burning_year = now.year as u8;

        Ok(())
    }

    #[access_control(valid_owner(&ctx.accounts.contract_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer))]
    pub fn withdraw_tokens_from_community_wallet<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawTokensFromCommunityWalletContext<'info>>,
        amount_to_withdraw: u64,
    ) -> Result<()> {
        let vesting_state = &mut ctx.accounts.vesting_state;
        let months_since_first_vesting = calculate_month_difference(
            vesting_state.start_timestamp,
            clock::Clock::get()?.unix_timestamp,
        );

        let unlocked_amount = calculate_unlocked_amount_community_wallet(
            vesting_state.initial_community_wallet_balance,
            months_since_first_vesting,
        );
        let amount_available_to_withdraw =
            ctx.accounts.community_account.amount.min(unlocked_amount);

        withdraw_vested_tokens(ctx, amount_to_withdraw, amount_available_to_withdraw)?;

        Ok(())
    }

    #[access_control(valid_owner(&ctx.accounts.contract_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer))]
    pub fn withdraw_tokens_from_partnership_wallet<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawTokensFromPartnershipWalletContext<'info>>,
        amount_to_withdraw: u64,
    ) -> Result<()> {
        let vesting_state = &mut ctx.accounts.vesting_state;
        let months_since_first_vesting = calculate_month_difference(
            vesting_state.start_timestamp,
            clock::Clock::get()?.unix_timestamp,
        );

        let unlocked_amount = calculate_unlocked_amount_partnership_wallet(
            vesting_state.initial_partnership_wallet_balance,
            months_since_first_vesting,
        );
        let amount_available_to_withdraw =
            ctx.accounts.partnership_account.amount.min(unlocked_amount);

        withdraw_vested_tokens(ctx, amount_to_withdraw, amount_available_to_withdraw)?;

        Ok(())
    }

    #[access_control(valid_owner(&ctx.accounts.contract_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer))]
    pub fn withdraw_tokens_from_marketing_wallet<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawTokensFromMarketingWalletContext<'info>>,
        amount_to_withdraw: u64,
    ) -> Result<()> {
        let vesting_state = &mut ctx.accounts.vesting_state;
        let months_since_first_vesting = calculate_month_difference(
            vesting_state.start_timestamp,
            clock::Clock::get()?.unix_timestamp,
        );

        let unlocked_amount = calculate_unlocked_amount_marketing_wallet(
            vesting_state.initial_marketing_wallet_balance,
            months_since_first_vesting,
        );
        let amount_available_to_withdraw =
            ctx.accounts.marketing_account.amount.min(unlocked_amount);

        withdraw_vested_tokens(ctx, amount_to_withdraw, amount_available_to_withdraw)?;

        Ok(())
    }
    
    #[access_control(valid_owner(&ctx.accounts.contract_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer))]
    pub fn withdraw_tokens_from_liquidity_wallet<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawTokensFromLiquidityWalletContext<'info>>,
        amount_to_withdraw: u64,
    ) -> Result<()> {
        let vesting_state = &mut ctx.accounts.vesting_state;
        let months_since_first_vesting = calculate_month_difference(
            vesting_state.start_timestamp,
            clock::Clock::get()?.unix_timestamp,
        );

        let unlocked_amount = calculate_unlocked_amount_liquidity_wallet(
            vesting_state.initial_liquidity_wallet_balance,
            months_since_first_vesting,
        );
        let amount_available_to_withdraw =
            ctx.accounts.liquidity_account.amount.min(unlocked_amount);

        withdraw_vested_tokens(ctx, amount_to_withdraw, amount_available_to_withdraw)?;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AccountInfoFromEthereum {
    pub wallet_name: String,
    pub account_public_key: Pubkey,
    pub account_balance: u64,
}