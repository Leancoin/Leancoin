//! Leancoin program

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

/// set seeds for pda accounts
pub const MINT_SEED: &str = "mint";
pub const PROGRAM_ACCOUNT_SEED: &str = "program_account";
pub const BURNING_ACCOUNT_SEED: &str = "burning_account";

const CONTRACT_STATE_SEED: &str = "contract_state";
const VESTING_STATE_SEED: &str = "vesting_state";

const COMMUNITY_ACCOUNT_SEED: &str = "community_account";
const PARTNERSHIP_ACCOUNT_SEED: &str = "partnership_account";
const MARKETING_ACCOUNT_SEED: &str = "marketing_account";
const LIQUIDITY_ACCOUNT_SEED: &str = "liquidity_account";

declare_id!("6h77QiaoS3UXaXPh7gDhfJetfkDpeW69ZdQ37EUCLL32");

/// This program is used to mint, burn and transfer tokens. It includes also a vesting mechanism.
#[program]
pub mod leancoin {
    use super::*;

    /// Initializes accounts and set states. It is the first function that must be called and it can be called only once.
    ///
    /// ### Arguments
    ///
    /// * `contract_state_nonce` - nonce for contract state account
    /// * `vesting_state_nonce` - nonce for vesting state account
    /// * `mint_nonce` - nonce for mint account
    /// * `program_account_nonce` - nonce for program account
    /// * `burning_account_nonce` - nonce for burning account
    /// * `community_wallet_nonce` - nonce for community wallet account
    /// * `liquidity_wallet_nonce` - nonce for liquidity wallet account
    /// * `marketing_wallet_nonce` - nonce for marketing wallet account
    /// * `partnership_wallet_nonce` - nonce for partnership wallet account
    pub fn initialize(
        ctx: Context<InitializeContext>,
        contract_state_nonce: u8,
        vesting_state_nonce: u8,
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
        contract_state.contract_state_nonce = contract_state_nonce;
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

        vesting_state.vesting_state_nonce = vesting_state_nonce;
        vesting_state.community_wallet_nonce = community_wallet_nonce;
        vesting_state.liquidity_wallet_nonce = liquidity_wallet_nonce;
        vesting_state.marketing_wallet_nonce = marketing_wallet_nonce;
        vesting_state.partnership_wallet_nonce = partnership_wallet_nonce;

        Ok(())
    }

    /// Imports token state from Ethereum. It mints, burns and transfer tokens based on the passed parameters that should specify the current token state on Ethereum.
    /// Additionally, it sets initial data related to burning and vesting like date (year and month) of the initial burning or initial state of accounts participating in vesting.
    /// The data is used later by burning and vesting functions.
    ///
    /// It is the second function that should be called and it can be called only once.
    ///
    /// ### Arguments
    ///
    /// * `account_info_from_ethereum` - a set of accounts reflecting those used on Ethereum; Leancoin tokens are transferred to these accounts
    /// * `amount_token_to_mint` - amount of tokens to mint to Program Account
    /// * `amount_token_to_burn` - amount of tokens to burn (also applied to Program Account)
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
            let matching_accounts = account_info_from_ethereum
                .iter()
                .filter(|account_info| account_info.account_public_key == account.key())
                .collect::<Vec<&AccountInfoFromEthereum>>();

            require!(
                matching_accounts.len() <= 1,
                LeancoinError::UserDuplicatedInUserInfo
            );

            let account_info = matching_accounts
                .first()
                .ok_or(LeancoinError::MismatchBetweenRemainingAccountsAndUserInfo)?;

            transfer_tokens(
                ctx.accounts.program_account.to_account_info(),
                account.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                PROGRAM_ACCOUNT_SEED,
                program_account_nonce,
                account_info.account_balance,
            )?;

            match account_info.wallet_name.as_str() {
                "community" => {
                    vesting_state.initial_community_wallet_balance = account_info.account_balance
                }
                "partnership" => {
                    vesting_state.initial_partnership_wallet_balance = account_info.account_balance
                }
                "marketing" => {
                    vesting_state.initial_marketing_wallet_balance = account_info.account_balance
                }
                "liquidity" => {
                    vesting_state.initial_liquidity_wallet_balance = account_info.account_balance
                }
                _ => {}
            }
        }

        require!(
            ctx.accounts.program_account.amount == 0,
            LeancoinError::ProgramAccountBalanceIsNotZero
        );
        require!(
            vesting_state.initial_community_wallet_balance != 0,
            LeancoinError::CommunityWalletBalanceIsZero
        );
        require!(
            vesting_state.initial_partnership_wallet_balance != 0,
            LeancoinError::PartnershipWalletBalanceIsZero
        );
        require!(
            vesting_state.initial_marketing_wallet_balance != 0,
            LeancoinError::MarketingWalletBalanceIsZero
        );
        require!(
            vesting_state.initial_liquidity_wallet_balance != 0,
            LeancoinError::LiquidityWalletBalanceIsZero
        );

        contract_state.import_ethereum_token_state_already_performed = true;

        Ok(())
    }

    /// Burns 5% of all the tokens currently held by the burning account.
    /// This function can be called only once per month and only between the 1st and the 5th day of the month.
    pub fn burn(ctx: Context<BurnContext>) -> Result<()> {
        let contract_state = &mut ctx.accounts.contract_state;
        let timestamp = clock::Clock::get()?.unix_timestamp;
        let now = parse_timestamp(timestamp)?;

        require!(now.days <= 5, LeancoinError::TooLateToBurnTokens);
        require!(
            contract_state.last_burning_month != now.month
                || contract_state.last_burning_year != now.year,
            LeancoinError::TokensAlreadyBurned
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

        contract_state.last_burning_month = now.month;
        contract_state.last_burning_year = now.year;

        Ok(())
    }

    /// Withdraws vested tokens from community wallet, if available.
    /// 2.5% of the initial wallet's balance is unlocked every month.
    ///
    /// ### Arguments
    ///
    /// * `amount_to_withdraw` - amount of tokens to withdraw
    #[access_control(valid_owner(&ctx.accounts.contract_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer))]
    pub fn withdraw_tokens_from_community_wallet<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawTokensFromCommunityWalletContext<'info>>,
        amount_to_withdraw: u64,
    ) -> Result<()> {
        let vesting_state = &mut ctx.accounts.vesting_state;
        let months_since_first_vesting = calculate_month_difference(
            vesting_state.start_timestamp,
            clock::Clock::get()?.unix_timestamp,
        )?;

        let unlocked_amount = calculate_unlocked_amount_community_wallet(
            vesting_state.initial_community_wallet_balance,
            months_since_first_vesting,
        );
        let amount_available_to_withdraw =
            ctx.accounts.community_account.amount.min(unlocked_amount);

        withdraw_vested_tokens(ctx, amount_to_withdraw, amount_available_to_withdraw)?;

        Ok(())
    }

    /// Withdraws vested tokens from partnership wallet, if available.
    /// 50% of the initial wallet's balance is unlocked after 1 month.
    /// The remaining part is unlocked after 2 months.
    ///
    /// ### Arguments
    ///
    /// * `amount_to_withdraw` - amount of tokens to withdraw
    #[access_control(valid_owner(&ctx.accounts.contract_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer))]
    pub fn withdraw_tokens_from_partnership_wallet<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawTokensFromPartnershipWalletContext<'info>>,
        amount_to_withdraw: u64,
    ) -> Result<()> {
        let vesting_state = &mut ctx.accounts.vesting_state;
        let months_since_first_vesting = calculate_month_difference(
            vesting_state.start_timestamp,
            clock::Clock::get()?.unix_timestamp,
        )?;

        let unlocked_amount = calculate_unlocked_amount_partnership_wallet(
            vesting_state.initial_partnership_wallet_balance,
            months_since_first_vesting,
        );
        let amount_available_to_withdraw =
            ctx.accounts.partnership_account.amount.min(unlocked_amount);

        withdraw_vested_tokens(ctx, amount_to_withdraw, amount_available_to_withdraw)?;

        Ok(())
    }

    /// Withdraws vested tokens from marketing wallet, if available.
    /// 40% of the initial wallet's balance is unlocked after 1 year.
    /// Starting from the 13th month, 5% of the initial wallet's balance is unlocked every month.
    ///
    /// ### Arguments
    ///
    /// * `amount_to_withdraw` - amount of tokens to withdraw
    #[access_control(valid_owner(&ctx.accounts.contract_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer))]
    pub fn withdraw_tokens_from_marketing_wallet<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawTokensFromMarketingWalletContext<'info>>,
        amount_to_withdraw: u64,
    ) -> Result<()> {
        let vesting_state = &mut ctx.accounts.vesting_state;
        let months_since_first_vesting = calculate_month_difference(
            vesting_state.start_timestamp,
            clock::Clock::get()?.unix_timestamp,
        )?;

        let unlocked_amount = calculate_unlocked_amount_marketing_wallet(
            vesting_state.initial_marketing_wallet_balance,
            months_since_first_vesting,
        )?;
        let amount_available_to_withdraw =
            ctx.accounts.marketing_account.amount.min(unlocked_amount);

        withdraw_vested_tokens(ctx, amount_to_withdraw, amount_available_to_withdraw)?;

        Ok(())
    }

    /// Withdraws vested tokens from liquidity wallet, if available.
    /// 50% of the initial wallet's balance is unlocked immediately.
    /// The remaining part is unlocked after 1 year.
    ///
    /// ### Arguments
    ///
    /// * `amount_to_withdraw` - amount of tokens to withdraw
    #[access_control(valid_owner(&ctx.accounts.contract_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer))]
    pub fn withdraw_tokens_from_liquidity_wallet<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawTokensFromLiquidityWalletContext<'info>>,
        amount_to_withdraw: u64,
    ) -> Result<()> {
        let vesting_state = &mut ctx.accounts.vesting_state;
        let months_since_first_vesting = calculate_month_difference(
            vesting_state.start_timestamp,
            clock::Clock::get()?.unix_timestamp,
        )?;

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

/// structure for storing information about the account
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AccountInfoFromEthereum {
    pub wallet_name: String,
    pub account_public_key: Pubkey,
    pub account_balance: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::ContractState;

    use anchor_lang::{system_program, InstructionData, ToAccountMetas};
    use anchor_spl::token::spl_token;
    use spl_token::state::Account;

    use solana_program::{
        hash::Hash, instruction::Instruction, program_pack::Pack, system_instruction,
    };
    use solana_program_test::*;

    use solana_sdk::{
        commitment_config::CommitmentLevel, signature::Keypair, signer::Signer,
        transaction::Transaction,
    };

    async fn initialize_instruction(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: Hash,
    ) -> Result<()> {
        let program_id = id();
        let (
            contract_state,
            contract_state_nonce,
            vesting_state,
            vesting_state_nonce,
            mint,
            mint_nonce,
            program_account,
            program_account_nonce,
            burning_account,
            burning_account_nonce,
            community_account,
            community_wallet_nonce,
            partnership_account,
            partnership_wallet_nonce,
            marketing_account,
            marketing_wallet_nonce,
            liquidity_account,
            liquidity_wallet_nonce,
        ) = get_pda_accounts();

        let token_program = spl_token::id();
        let signer = payer.pubkey();

        let data = instruction::Initialize {
            contract_state_nonce,
            vesting_state_nonce,
            mint_nonce,
            program_account_nonce,
            burning_account_nonce,
            community_wallet_nonce,
            liquidity_wallet_nonce,
            marketing_wallet_nonce,
            partnership_wallet_nonce,
        }
        .data();

        let accs = accounts::InitializeContext {
            contract_state,
            vesting_state,
            community_account,
            liquidity_account,
            marketing_account,
            partnership_account,
            mint,
            program_account,
            burning_account,
            token_program,
            signer,
            system_program: system_program::ID,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[payer], recent_blockhash);
        banks_client
            .process_transaction_with_commitment(transaction.clone(), CommitmentLevel::Finalized)
            .await
            .unwrap();

        Ok(())
    }

    async fn import_ethereum_token_state_instruction(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: Hash,
    ) -> Result<()> {
        let program_id = id();

        let (
            contract_state,
            _,
            vesting_state,
            _,
            mint,
            mint_nonce,
            program_account,
            program_account_nonce,
            burning_account,
            _,
            community_account,
            _,
            partnership_account,
            _,
            marketing_account,
            _,
            liquidity_account,
            _,
        ) = get_pda_accounts();

        let token_program = spl_token::id();
        let signer = payer.pubkey();

        let account_info_from_ethereum = get_accounts_to_mapping();
        let amount_token_to_mint = 10000000000000000000;
        let amount_token_to_burn = 1470000000000000000;

        let data = instruction::ImportEthereumTokenState {
            account_info_from_ethereum,
            amount_token_to_mint,
            amount_token_to_burn,
        }
        .data();

        let accs = accounts::ImportEthereumTokenStateContext {
            contract_state,
            vesting_state,
            mint,
            program_account,
            token_program,
            signer,
        };

        let mut accounts = accs.to_account_metas(Some(false));
        accounts.push(AccountMeta::new(burning_account, false));
        accounts.push(AccountMeta::new(community_account, false));
        accounts.push(AccountMeta::new(partnership_account, false));
        accounts.push(AccountMeta::new(marketing_account, false));
        accounts.push(AccountMeta::new(liquidity_account, false));

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &data, accounts)],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[payer], recent_blockhash);
        banks_client
            .process_transaction_with_commitment(transaction.clone(), CommitmentLevel::Finalized)
            .await
            .unwrap();

        let contract_state_info = banks_client
            .get_account_with_commitment(contract_state, CommitmentLevel::Finalized)
            .await
            .unwrap()
            .unwrap();

        let contract_state: ContractState =
            ContractState::try_deserialize_unchecked(&mut contract_state_info.data.as_slice())
                .unwrap();

        assert_eq!(
            contract_state.import_ethereum_token_state_already_performed,
            true
        );
        assert_eq!(contract_state.authority, signer);
        assert_eq!(contract_state.last_burning_month, 0);
        assert_eq!(contract_state.last_burning_year, 0);
        assert_eq!(contract_state.mint_nonce, mint_nonce);
        assert_eq!(contract_state.program_account_nonce, program_account_nonce);

        Ok(())
    }

    #[tokio::test]
    async fn test_initialize() {
        let program_id = id();
        let program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_import_ethereum_token_state() {
        let program_id = id();
        let mut program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        program_test.set_compute_max_units(500000);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        import_ethereum_token_state_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn test_burn() {
        let program_id = id();
        let mut program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        program_test.set_compute_max_units(500000);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let token_program = spl_token::id();

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
        import_ethereum_token_state_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (contract_state, _, _, _, mint, _, _, _, burning_account, _, _, _, _, _, _, _, _, _) =
            get_pda_accounts();

        let data = instruction::Burn {}.data();
        let accs = accounts::BurnContext {
            mint,
            contract_state,
            burning_account,
            token_program,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }

    #[tokio::test]
    async fn test_burn_change_clock() {
        let program_id = id();
        let mut program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        program_test.set_compute_max_units(500000);

        let token_program = spl_token::id();
        let mut program_test_context = program_test.start_with_context().await;

        //  Sunday, 5 March 2023 01:01:01
        let time_in_timestamp = 1677978061;
        set_time(&mut program_test_context, time_in_timestamp).await;

        let mut banks_client = program_test_context.banks_client;
        let payer = program_test_context.payer;
        let recent_blockhash = program_test_context.last_blockhash;

        let mut sub_clock = Clock::default();
        sub_clock.unix_timestamp += 2_160_000;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
        import_ethereum_token_state_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (contract_state, _, _, _, mint, _, _, _, burning_account, _, _, _, _, _, _, _, _, _) =
            get_pda_accounts();

        let burning_account_mint_balance =
            get_token_balance(&mut banks_client, &burning_account).await;
        let expected_burning_account_mint_balance = 1800000000000000000;
        assert_eq!(
            burning_account_mint_balance,
            expected_burning_account_mint_balance
        );

        let data = instruction::Burn {}.data();
        let accs = accounts::BurnContext {
            mint,
            contract_state,
            burning_account,
            token_program,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let burning_account_mint_balance =
            get_token_balance(&mut banks_client, &burning_account).await;
        let expected_burning_account_mint_balance = 1800000000000000000 - 1800000000000000000 / 20;
        assert_eq!(
            burning_account_mint_balance,
            expected_burning_account_mint_balance
        );
    }

    async fn get_token_balance(banks_client: &mut BanksClient, burning_account: &Pubkey) -> u64 {
        let burning_account_mint_account = banks_client
            .get_account(burning_account.clone())
            .await
            .unwrap()
            .unwrap();
        let burning_account_mint_state = spl_token::state::Account::unpack_from_slice(
            burning_account_mint_account.data.as_slice(),
        )
        .unwrap();

        burning_account_mint_state.amount
    }

    #[tokio::test]
    #[should_panic]
    async fn test_burn_change_clock_two_times_in_one_day() {
        let program_id = id();
        let program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        let mut program_test_context = program_test.start_with_context().await;
        let token_program = spl_token::id();

        //  Sunday, 5 March 2023 01:01:01
        let time_in_timestamp = 1677978061;
        set_time(&mut program_test_context, time_in_timestamp).await;

        let mut banks_client = program_test_context.banks_client;
        let payer = program_test_context.payer;
        let recent_blockhash = program_test_context.last_blockhash;

        let mut sub_clock = Clock::default();
        sub_clock.unix_timestamp += 2_160_000;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
        import_ethereum_token_state_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (contract_state, _, _, _, mint, _, _, _, burning_account, _, _, _, _, _, _, _, _, _) =
            get_pda_accounts();

        let data = instruction::Burn {}.data();
        let accs = accounts::BurnContext {
            mint,
            contract_state,
            burning_account,
            token_program,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        let mut program_test_context = program_test.start_with_context().await;

        //  Sunday, 5 March 2023 05:01:01
        let time_in_timestamp = 1677992461;
        set_time(&mut program_test_context, time_in_timestamp).await;

        let mut banks_client = program_test_context.banks_client;
        let payer = program_test_context.payer;
        let recent_blockhash = program_test_context.last_blockhash;

        let data = instruction::Burn {}.data();
        let accs = accounts::BurnContext {
            mint,
            contract_state,
            burning_account,
            token_program,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }

    #[tokio::test]
    async fn test_withdraw_tokens_from_community_wallet() {
        let program_id = id();
        let mut program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        program_test.set_compute_max_units(500000);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let token_program = spl_token::id();
        let signer = payer.pubkey();

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
        import_ethereum_token_state_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (
            contract_state,
            _,
            vesting_state,
            _,
            mint,
            _,
            _,
            _,
            _,
            _,
            community_account,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
        ) = get_pda_accounts();

        let data = instruction::WithdrawTokensFromCommunityWallet {
            amount_to_withdraw: 25_000_000_000_000_000,
        }
        .data();

        let deposit_wallet =
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint)
                .await
                .unwrap();

        let deposit_wallet_balance_before_withdraw_tokens_from_community_wallet_context =
            get_token_balance(&mut banks_client, &deposit_wallet).await;
        assert_eq!(
            deposit_wallet_balance_before_withdraw_tokens_from_community_wallet_context,
            0
        );

        let accs = accounts::WithdrawTokensFromCommunityWalletContext {
            vesting_state,
            deposit_wallet,
            signer,
            contract_state,
            community_account,
            token_program,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let deposit_wallet_balance_after_withdraw_tokens_from_community_wallet_context =
            get_token_balance(&mut banks_client, &deposit_wallet).await;
        assert_eq!(
            deposit_wallet_balance_after_withdraw_tokens_from_community_wallet_context,
            25_000_000_000_000_000
        );
    }

    #[tokio::test]
    #[should_panic]
    async fn test_withdraw_tokens_from_partnership_wallet() {
        let program_id = id();
        let mut program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        program_test.set_compute_max_units(500000);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let token_program = spl_token::id();
        let signer = payer.pubkey();

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
        import_ethereum_token_state_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (
            contract_state,
            _,
            vesting_state,
            _,
            mint,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            partnership_account,
            _,
            _,
            _,
            _,
            _,
        ) = get_pda_accounts();

        let data = instruction::WithdrawTokensFromPartnershipWallet {
            amount_to_withdraw: 1,
        }
        .data();

        let deposit_wallet =
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint)
                .await
                .unwrap();

        let accs = accounts::WithdrawTokensFromPartnershipWalletContext {
            vesting_state,
            deposit_wallet,
            signer,
            contract_state,
            partnership_account,
            token_program,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }

    #[tokio::test]
    async fn test_withdraw_tokens_from_partnership_wallet_after_one_month() {
        let program_id = id();
        let mut program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        program_test.set_compute_max_units(500000);

        let token_program = spl_token::id();
        let mut program_test_context = program_test.start_with_context().await;

        //  Sunday, 5 March 2023 01:01:01
        let time_in_timestamp = 1677978061;
        set_time(&mut program_test_context, time_in_timestamp).await;

        let mut banks_client = program_test_context.banks_client.clone();
        let payer = Keypair::from_base58_string(&program_test_context.payer.to_base58_string());
        let recent_blockhash = program_test_context.last_blockhash;

        let signer = payer.pubkey();

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
        import_ethereum_token_state_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        //  Thursday, 11 May 2023 01:01:01
        let time_in_timestamp = 1683766861;
        set_time(&mut program_test_context, time_in_timestamp).await;

        let (
            contract_state,
            _,
            vesting_state,
            _,
            mint,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            partnership_account,
            _,
            _,
            _,
            _,
            _,
        ) = get_pda_accounts();

        let data = instruction::WithdrawTokensFromPartnershipWallet {
            amount_to_withdraw: 1,
        }
        .data();

        let deposit_wallet =
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint)
                .await
                .unwrap();

        let deposit_wallet_balance_before_withdraw_tokens_from_partnership_wallet_context =
            get_token_balance(&mut banks_client, &deposit_wallet).await;
        assert_eq!(
            deposit_wallet_balance_before_withdraw_tokens_from_partnership_wallet_context,
            0
        );

        let accs = accounts::WithdrawTokensFromPartnershipWalletContext {
            vesting_state,
            deposit_wallet,
            signer,
            contract_state,
            partnership_account,
            token_program,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let deposit_wallet_balance_after_withdraw_tokens_from_partnership_wallet_context =
            get_token_balance(&mut banks_client, &deposit_wallet).await;
        assert_eq!(
            deposit_wallet_balance_after_withdraw_tokens_from_partnership_wallet_context,
            1
        );
    }

    #[tokio::test]
    #[should_panic]
    async fn test_withdraw_tokens_from_marketing_wallet() {
        let program_id = id();
        let mut program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        program_test.set_compute_max_units(500000);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let token_program = spl_token::id();
        let signer = payer.pubkey();

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
        import_ethereum_token_state_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (
            contract_state,
            _,
            vesting_state,
            _,
            mint,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            marketing_account,
            _,
            _,
            _,
        ) = get_pda_accounts();

        let data = instruction::WithdrawTokensFromMarketingWallet {
            amount_to_withdraw: 1,
        }
        .data();

        let deposit_wallet =
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint)
                .await
                .unwrap();

        let accs = accounts::WithdrawTokensFromMarketingWalletContext {
            vesting_state,
            deposit_wallet,
            signer,
            contract_state,
            marketing_account,
            token_program,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }

    #[tokio::test]
    async fn test_withdraw_tokens_from_marketing_wallet_after_one_year() {
        let program_id = id();
        let mut program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        program_test.set_compute_max_units(500000);

        let token_program = spl_token::id();
        let mut program_test_context = program_test.start_with_context().await;

        //  Sunday, 5 March 2023 01:01:01
        let time_in_timestamp = 1677978061;
        set_time(&mut program_test_context, time_in_timestamp).await;

        let mut banks_client = program_test_context.banks_client.clone();
        let payer = Keypair::from_base58_string(&program_test_context.payer.to_base58_string());
        let recent_blockhash = program_test_context.last_blockhash;

        let signer = payer.pubkey();

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
        import_ethereum_token_state_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        //  Thursday, 11 May 2023 01:01:01
        let time_in_timestamp = 1709600470;
        set_time(&mut program_test_context, time_in_timestamp).await;

        let (
            contract_state,
            _,
            vesting_state,
            _,
            mint,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            marketing_account,
            _,
            _,
            _,
        ) = get_pda_accounts();

        let data = instruction::WithdrawTokensFromMarketingWallet {
            amount_to_withdraw: 1,
        }
        .data();

        let deposit_wallet =
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint)
                .await
                .unwrap();

        let deposit_wallet_balance_before_withdraw_tokens_from_marketing_wallet_context =
            get_token_balance(&mut banks_client, &deposit_wallet).await;
        assert_eq!(
            deposit_wallet_balance_before_withdraw_tokens_from_marketing_wallet_context,
            0
        );

        let accs = accounts::WithdrawTokensFromMarketingWalletContext {
            vesting_state,
            deposit_wallet,
            signer,
            contract_state,
            marketing_account,
            token_program,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let deposit_wallet_balance_after_withdraw_tokens_from_marketing_wallet_context =
            get_token_balance(&mut banks_client, &deposit_wallet).await;
        assert_eq!(
            deposit_wallet_balance_after_withdraw_tokens_from_marketing_wallet_context,
            1
        );
    }

    #[tokio::test]
    async fn test_withdraw_tokens_from_liquidity_wallet() {
        let program_id = id();
        let mut program_test = ProgramTest::new("leancoin", program_id, processor!(entry));
        program_test.set_compute_max_units(500000);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let token_program = spl_token::id();
        let signer = payer.pubkey();

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
        import_ethereum_token_state_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (
            contract_state,
            _,
            vesting_state,
            _,
            mint,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            liquidity_account,
            _,
        ) = get_pda_accounts();

        let data = instruction::WithdrawTokensFromLiquidityWallet {
            amount_to_withdraw: 1,
        }
        .data();

        let deposit_wallet =
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint)
                .await
                .unwrap();

        let deposit_wallet_balance_before_withdraw_tokens_from_liquidity_wallet_context =
            get_token_balance(&mut banks_client, &deposit_wallet).await;
        assert_eq!(
            deposit_wallet_balance_before_withdraw_tokens_from_liquidity_wallet_context,
            0
        );

        let accs = accounts::WithdrawTokensFromLiquidityWalletContext {
            vesting_state,
            deposit_wallet,
            signer,
            contract_state,
            liquidity_account,
            token_program,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let deposit_wallet_balance_after_withdraw_tokens_from_liquidity_wallet_context =
            get_token_balance(&mut banks_client, &deposit_wallet).await;
        assert_eq!(
            deposit_wallet_balance_after_withdraw_tokens_from_liquidity_wallet_context,
            1
        );
    }

    async fn create_token_account(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: Hash,
        mint: Pubkey,
    ) -> Result<Pubkey> {
        let rent = Rent::default();
        let new_keypair = Keypair::new();
        let transaction = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &new_keypair.pubkey(),
                    rent.minimum_balance(Account::LEN),
                    Account::LEN.try_into().unwrap(),
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &new_keypair.pubkey(),
                    &mint,
                    &payer.pubkey(),
                )
                .unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer, &new_keypair],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        Ok(new_keypair.pubkey())
    }

    fn get_accounts_to_mapping() -> Vec<AccountInfoFromEthereum> {
        let (
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            burning_account,
            _,
            community_account,
            _,
            partnership_account,
            _,
            marketing_account,
            _,
            liquidity_account,
            _,
        ) = get_pda_accounts();

        let burn_balance = 1800000000000000000; // 18% of total supply
        let community_balance = 1000000000000000000; // 10% of total supply
        let partnership_balance = 2000000000000000000; // 20% of total supply
        let marketing_balance = 1500000000000000000; // 15% of total supply
        let liquidity_balance = 1000000000000000000; // 10% of total supply
        let swap_balance = 1230000000000000000; // 12.3% of total supply

        vec![
            AccountInfoFromEthereum {
                wallet_name: String::from("Burning"),
                account_public_key: burning_account,
                account_balance: burn_balance,
            },
            AccountInfoFromEthereum {
                wallet_name: String::from("community"),
                account_public_key: community_account,
                account_balance: community_balance,
            },
            AccountInfoFromEthereum {
                wallet_name: String::from("partnership"),
                account_public_key: partnership_account,
                account_balance: partnership_balance,
            },
            AccountInfoFromEthereum {
                wallet_name: String::from("marketing"),
                account_public_key: marketing_account,
                account_balance: marketing_balance,
            },
            AccountInfoFromEthereum {
                wallet_name: String::from("liquidity"),
                account_public_key: liquidity_account,
                account_balance: liquidity_balance,
            },
            AccountInfoFromEthereum {
                wallet_name: String::from("swap"),
                account_public_key: Pubkey::new_unique(),
                account_balance: swap_balance,
            },
        ]
    }

    fn get_pda_accounts() -> (
        Pubkey,
        u8,
        Pubkey,
        u8,
        Pubkey,
        u8,
        Pubkey,
        u8,
        Pubkey,
        u8,
        Pubkey,
        u8,
        Pubkey,
        u8,
        Pubkey,
        u8,
        Pubkey,
        u8,
    ) {
        let program_id = id();

        let (contract_state, contract_state_nonce) =
            Pubkey::find_program_address(&[b"contract_state"], &program_id);
        let (vesting_state, vesting_state_nonce) =
            Pubkey::find_program_address(&[b"vesting_state"], &program_id);
        let (mint, mint_nonce) = Pubkey::find_program_address(&[b"mint"], &program_id);
        let (program_account, program_account_nonce) =
            Pubkey::find_program_address(&[b"program_account"], &program_id);
        let (burning_account, burning_nonce) =
            Pubkey::find_program_address(&[b"burning_account"], &program_id);
        let (community_account, community_nonce) =
            Pubkey::find_program_address(&[b"community_account"], &program_id);
        let (partnership_account, partnership_nonce) =
            Pubkey::find_program_address(&[b"partnership_account"], &program_id);
        let (marketing_account, marketing_nonce) =
            Pubkey::find_program_address(&[b"marketing_account"], &program_id);
        let (liquidity_account, liquidity_nonce) =
            Pubkey::find_program_address(&[b"liquidity_account"], &program_id);

        (
            contract_state,
            contract_state_nonce,
            vesting_state,
            vesting_state_nonce,
            mint,
            mint_nonce,
            program_account,
            program_account_nonce,
            burning_account,
            burning_nonce,
            community_account,
            community_nonce,
            partnership_account,
            partnership_nonce,
            marketing_account,
            marketing_nonce,
            liquidity_account,
            liquidity_nonce,
        )
    }

    async fn set_time(ctx: &mut ProgramTestContext, time: i64) {
        let clock_sysvar: Clock = ctx.banks_client.get_sysvar().await.unwrap();
        let mut new_clock = clock_sysvar.clone();
        new_clock.epoch = new_clock.epoch + 30;
        new_clock.unix_timestamp = time;

        ctx.set_sysvar(&new_clock);
    }
}
