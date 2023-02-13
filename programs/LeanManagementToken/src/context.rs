use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::account::*;
use crate::*;

const CONTRACT_STATE_DISCRIMINATOR_LEN: usize = 8;
const MINT_NONCE_LEN: usize = 1;
const ETHEREUM_TOKEN_STATE_MAPPING_ALREADY_PERFORMED_LEN: usize = 1;
const PROGRAM_ACCOUNT_NONCE_LEN: usize = 1;
const BURNING_ACCOUNT_NONCE_LEN: usize = 1;
const LAST_BURNING_MONTH_LEN: usize = 1;
const LAST_BURNING_YEAR_LEN: usize = 1;
const AUTHORITY_LEN: usize = 32;

const VESTING_STATE_DISCRIMINATOR_LEN: usize = 8;
const AMOUNT_COMMUNITY_WALLET_LEN: usize = 8;
const AMOUNT_PARTNERSHIP_WALLET_LEN: usize = 8;
const AMOUNT_MARKETING_WALLET_LEN: usize = 8;
const AMOUNT_LIQUIDITY_WALLET_LEN: usize = 8;

const COMMUNITY_WALLET_NONCE_LEN: usize = 1;
const PARTNERSHIP_WALLET_NONCE_LEN: usize = 1;
const MARKETING_WALLET_NONCE_LEN: usize = 1;
const LIQUIDITY_WALLET_NONCE_LEN: usize = 1;
const START_TIMESTAMP_LEN: usize = 8;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeContext<'info> {
    #[account(
        init,
        payer = signer,
        space = InitializeContext::CONTRACT_STATE_LEN,
        seeds = [CONTRACT_STATE_SEED.as_bytes()],
        bump
    )]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(
        init,
        payer = signer,
        space = InitializeContext::VESTING_STATE_LEN,
        seeds = [VESTING_STATE_SEED.as_bytes()],
        bump
    )]
    pub vesting_state: Box<Account<'info, VestingState>>,
    #[account(
        init, 
        payer = signer,
        seeds = [MINT_SEED.as_bytes()],
        bump,
        mint::decimals = 9,
        mint::authority = mint
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = program_account,
        seeds = [PROGRAM_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub program_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = burning_account,
        seeds = [BURNING_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub burning_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = community_account,
        seeds = [COMMUNITY_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub community_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = partnership_account,
        seeds = [PARTNERSHIP_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub partnership_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = marketing_account,
        seeds = [MARKETING_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub marketing_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = liquidity_account,
        seeds = [LIQUIDITY_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub liquidity_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeContext<'info> {
    pub const CONTRACT_STATE_LEN: usize = CONTRACT_STATE_DISCRIMINATOR_LEN
        + MINT_NONCE_LEN
        + ETHEREUM_TOKEN_STATE_MAPPING_ALREADY_PERFORMED_LEN
        + PROGRAM_ACCOUNT_NONCE_LEN
        + BURNING_ACCOUNT_NONCE_LEN
        + LAST_BURNING_MONTH_LEN
        + LAST_BURNING_YEAR_LEN
        + AUTHORITY_LEN;

    pub const VESTING_STATE_LEN: usize = VESTING_STATE_DISCRIMINATOR_LEN
        + AMOUNT_COMMUNITY_WALLET_LEN
        + AMOUNT_PARTNERSHIP_WALLET_LEN
        + AMOUNT_MARKETING_WALLET_LEN
        + AMOUNT_LIQUIDITY_WALLET_LEN
        + COMMUNITY_WALLET_NONCE_LEN
        + PARTNERSHIP_WALLET_NONCE_LEN
        + MARKETING_WALLET_NONCE_LEN
        + LIQUIDITY_WALLET_NONCE_LEN
        + START_TIMESTAMP_LEN;
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ImportEthereumTokenStateContext<'info> {
    #[account(
        mut,
        seeds = [CONTRACT_STATE_SEED.as_bytes()],
        bump,
    )]
    pub contract_state: Box<Account<'info, ContractState>>,

    #[account(
        mut,
        seeds = [VESTING_STATE_SEED.as_bytes()],
        bump,
    )]
    pub vesting_state: Box<Account<'info, VestingState>>,

    #[account(
        mut,
        seeds = [MINT_SEED.as_bytes()],
        bump,
    )]
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [PROGRAM_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub program_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct BurnContext<'info> {
    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(
        mut, 
        seeds = [BURNING_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub burning_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct WithdrawTokensFromCommunityWalletContext<'info> {
    #[account(mut)]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(mut)]
    pub vesting_state: Box<Account<'info, VestingState>>,

    #[account(
        mut, 
        seeds = [COMMUNITY_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub community_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub deposit_wallet: Box<Account<'info, TokenAccount>>,

    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct WithdrawTokensFromPartnershipWalletContext<'info> {
    #[account(mut)]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(mut)]
    pub vesting_state: Box<Account<'info, VestingState>>,

    #[account(
        mut,
        seeds = [PARTNERSHIP_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub partnership_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub deposit_wallet: Box<Account<'info, TokenAccount>>,

    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct WithdrawTokensFromMarketingWalletContext<'info> {
    #[account(mut)]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(mut)]
    pub vesting_state: Box<Account<'info, VestingState>>,

    #[account(
        mut,
        seeds = [MARKETING_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub marketing_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub deposit_wallet: Box<Account<'info, TokenAccount>>,

    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct WithdrawTokensFromLiquidityWalletContext<'info> {
    #[account(mut)]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(mut)]
    pub vesting_state: Box<Account<'info, VestingState>>,

    #[account(
        mut,
        seeds = [LIQUIDITY_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub liquidity_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub deposit_wallet: Box<Account<'info, TokenAccount>>,

    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub trait VestedWalletContext<'info> {
    fn vested_account(&self) -> Box<Account<'info, TokenAccount>>;
    fn vested_account_nonce(&self) -> u8;
    fn vested_account_seed(&self) -> &str;
    fn deposit_wallet(&self) -> Box<Account<'info, TokenAccount>>;
    fn token_program(&self) -> Program<'info, Token>;
}

impl<'info> VestedWalletContext<'info> for WithdrawTokensFromCommunityWalletContext<'info> {
    fn vested_account(&self) -> Box<Account<'info, TokenAccount>> {
        self.community_account.to_owned()
    }

    fn vested_account_nonce(&self) -> u8 {
        self.vesting_state.community_wallet_nonce
    }

    fn vested_account_seed(&self) -> &'info str {
        COMMUNITY_ACCOUNT_SEED
    }

    fn deposit_wallet(&self) -> Box<Account<'info, TokenAccount>> {
        self.deposit_wallet.to_owned()
    }

    fn token_program(&self) -> Program<'info, Token> {
        self.token_program.to_owned()
    }
}

impl<'info> VestedWalletContext<'info> for WithdrawTokensFromPartnershipWalletContext<'info> {
    fn vested_account(&self) -> Box<Account<'info, TokenAccount>> {
        self.partnership_account.to_owned()
    }

    fn vested_account_nonce(&self) -> u8 {
        self.vesting_state.partnership_wallet_nonce
    }

    fn vested_account_seed(&self) -> &'info str {
        PARTNERSHIP_ACCOUNT_SEED
    }

    fn deposit_wallet(&self) -> Box<Account<'info, TokenAccount>> {
        self.deposit_wallet.to_owned()
    }

    fn token_program(&self) -> Program<'info, Token> {
        self.token_program.to_owned()
    }
}

impl<'info> VestedWalletContext<'info> for WithdrawTokensFromMarketingWalletContext<'info> {
    fn vested_account(&self) -> Box<Account<'info, TokenAccount>> {
        self.marketing_account.to_owned()
    }

    fn vested_account_nonce(&self) -> u8 {
        self.vesting_state.marketing_wallet_nonce
    }

    fn vested_account_seed(&self) -> &'info str {
        MARKETING_ACCOUNT_SEED
    }

    fn deposit_wallet(&self) -> Box<Account<'info, TokenAccount>> {
        self.deposit_wallet.to_owned()
    }

    fn token_program(&self) -> Program<'info, Token> {
        self.token_program.to_owned()
    }
}

impl<'info> VestedWalletContext<'info> for WithdrawTokensFromLiquidityWalletContext<'info> {
    fn vested_account(&self) -> Box<Account<'info, TokenAccount>> {
        self.liquidity_account.to_owned()
    }

    fn vested_account_nonce(&self) -> u8 {
        self.vesting_state.liquidity_wallet_nonce
    }

    fn vested_account_seed(&self) -> &'info str {
        LIQUIDITY_ACCOUNT_SEED
    }

    fn deposit_wallet(&self) -> Box<Account<'info, TokenAccount>> {
        self.deposit_wallet.to_owned()
    }

    fn token_program(&self) -> Program<'info, Token> {
        self.token_program.to_owned()
    }
}