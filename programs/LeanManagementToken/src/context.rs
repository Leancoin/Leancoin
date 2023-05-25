use anchor_lang::{
    error,
    prelude::{
        account, require_keys_neq, Account, AccountInfo, Accounts, Key, Program, Pubkey, Rent,
        Signer, SolanaSysvar, System, ToAccountInfo,
    },
    solana_program::system_program,
    Id, Space,
};
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata;

use crate::account::{ContractState, VestingState};

use crate::{
    BURNING_ACCOUNT_SEED, COMMUNITY_ACCOUNT_SEED, CONTRACT_STATE_SEED, LIQUIDITY_ACCOUNT_SEED,
    MARKETING_ACCOUNT_SEED, MINT_SEED, PARTNERSHIP_ACCOUNT_SEED, PROGRAM_ACCOUNT_SEED,
    VESTING_STATE_SEED,
};

/// The discriminator is defined by the first 8 bytes of the SHA256 hash of the account's Rust identifier.
/// It includes the name of struct type and lets Anchor know what type of account it should deserialize the data as.
const DISCRIMINATOR_LEN: usize = 8;

/// Context for the initialize instruction.
///
/// This context is used to initialize the contract state and the vesting state.
///
/// The contract state is initialized with the following accounts:
///
/// - `contract_state` - the account that contains the contract state,
/// - `vesting_state` - the account that contains the vesting state,
/// - `mint` - the mint account,
/// - `program_account` - the account that contains the tokens that will be distributed to the users,
/// - `burning_account` - the account that contains the tokens that will be burned.
///
/// The vesting state is initialized with the following accounts:
///
/// - `community_wallet` - the account that contains the tokens that will be distributed to the community wallet,
/// - `partnership_wallet` - the account that contains the tokens that will be distributed to the partnership wallet,
/// - `marketing_wallet` - the account that contains the tokens that will be distributed to the marketing wallet,
/// - `liquidity_wallet` - the account that contains the tokens that will be distributed to the liquidity wallet.
///
/// The context includes also:
/// - `token_program` - the Solana token program account,
/// - `system_program` - the Solana system program account,
/// - `signer` - the signer of the transaction which executes initialize instruction, the signer becomes contract's owner.
#[derive(Accounts)]
pub struct InitializeContext<'info> {
    #[account(
        init,
        payer = signer,
        space = DISCRIMINATOR_LEN + ContractState::INIT_SPACE,
        seeds = [CONTRACT_STATE_SEED.as_bytes()],
        bump
    )]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(
        init,
        payer = signer,
        space = DISCRIMINATOR_LEN + VestingState::INIT_SPACE,
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

/// Context for the import_ethereum_token_state instruction.
///
/// This context is used to update the contract state and the vesting state using some data from the Ethereum contract.
///
/// The contract state is updated using the following accounts:
///
/// - `contract_state` - the account that contains the contract state,
/// - `mint` - the mint account,
/// - `program_account` - the account that contains the tokens that will be distributed to the users.
///
/// The vesting state is updated using the following accounts:
///
/// - `vesting_state` - the account that contains the vesting state.
///
/// The context includes also:
/// - `token_program` - the Solana token program account,
/// - `signer` - the signer of the transaction which must be the contract's owner.
#[derive(Accounts)]
pub struct ImportEthereumTokenStateContext<'info> {
    #[account(
        mut,
        seeds = [CONTRACT_STATE_SEED.as_bytes()],
        bump = contract_state.contract_state_nonce,
    )]
    pub contract_state: Box<Account<'info, ContractState>>,

    #[account(
        mut,
        seeds = [VESTING_STATE_SEED.as_bytes()],
        bump = vesting_state.vesting_state_nonce,
    )]
    pub vesting_state: Box<Account<'info, VestingState>>,

    #[account(
        mut,
        seeds = [MINT_SEED.as_bytes()],
        bump = contract_state.mint_nonce,
    )]
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [PROGRAM_ACCOUNT_SEED.as_bytes()],
        bump = contract_state.program_account_nonce,
    )]
    pub program_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub signer: Signer<'info>,
}

/// Context for the burn instruction.
///
/// This context is used to burn tokens from burning_account.
///
/// The context includes:
/// - `burning_account` - the account that holds tokens to be burned,
/// - `mint` - the mint account used to mint tokens that should be burned,
/// - `contract_state` - the account that contains the contract state,
/// - `token_program` - the Solana token program account.
#[derive(Accounts)]
pub struct BurnContext<'info> {
    #[account(
        mut,
        seeds = [MINT_SEED.as_bytes()],
        bump = contract_state.mint_nonce,
    )]
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [CONTRACT_STATE_SEED.as_bytes()],
        bump = contract_state.contract_state_nonce,
    )]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(
        mut,
        seeds = [BURNING_ACCOUNT_SEED.as_bytes()],
        bump = contract_state.burning_account_nonce,
    )]
    pub burning_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

/// Context for the change_authority instruction.
///
/// This context is used to set new authority on contract state.
///
/// The context includes:
/// - `contract_state` - the account that contains the contract state,
/// - `signer` - the signer of the transaction which must be the contract's owner.
#[derive(Accounts)]
pub struct ChangeAuthorityContext<'info> {
    #[account(
        mut,
        seeds = [CONTRACT_STATE_SEED.as_bytes()],
        bump = contract_state.contract_state_nonce,
    )]
    pub contract_state: Box<Account<'info, ContractState>>,
    pub signer: Signer<'info>,
}

/// Context for the set token metadata instruction.
///
/// This context is used to set the token metadata.
///
/// The context includes:
///
/// - contract_state - the account containing the contract state,
/// - mint - the mint account,
/// - signer - the signer of the transaction, who must be the contract's owner,
/// - metadata_pda - the metadata PDA account,
/// - system_program - the Solana system program account,
/// - token_program - the Solana token program account,
/// - metadata_program - the Metaplex metadata program account,
///
/// There are also check comments within the context:
/// - metadata_pda and metadata_program are checked by the inner instruction.
#[derive(Accounts)]
pub struct SetTokenMetadataContext<'info> {
    #[account(
        mut,
        seeds = [CONTRACT_STATE_SEED.as_bytes()],
        bump = contract_state.contract_state_nonce,
    )]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(
        mut,
        seeds = [MINT_SEED.as_bytes()],
        bump = contract_state.mint_nonce,
    )]
    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: The metadata program account. It is considered safe because it is checked by the inner instruction, ensuring it is the correct account.
    #[account(mut, address = Pubkey::find_program_address(&[b"metadata", &mpl_token_metadata::id().to_bytes(), &mint.key().to_bytes()], &mpl_token_metadata::id()).0)]
    pub metadata_pda: AccountInfo<'info>,

    /// CHECK: The metadata program account. It is considered safe because it is checked by the inner instruction, ensuring it is the correct account.
    #[account(address = mpl_token_metadata::id())]
    pub metadata_program: AccountInfo<'info>,

    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Context for the withdraw_tokens_from_community_wallet instruction.
///
/// This context is used to withdraw tokens from the community wallet.
///
/// The context includes:
/// - `contract_state` - the account that contains the contract state,
/// - `vesting_state` - the account that contains the vesting state,
/// - `community_account` - the community wallet account which is the source of tokens to be transferred,
/// - `deposit_wallet` - the destination account receiving tokens transferred from community_account,
/// - `signer` - the signer of the transaction which must be the contract's owner.
/// - `token_program` - the Solana token program account.
#[derive(Accounts)]
pub struct WithdrawTokensFromCommunityWalletContext<'info> {
    #[account(
        mut,
        seeds = [CONTRACT_STATE_SEED.as_bytes()],
        bump = contract_state.contract_state_nonce,
    )]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(
        mut,
        seeds = [VESTING_STATE_SEED.as_bytes()],
        bump = vesting_state.vesting_state_nonce,
    )]
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

/// Context for the withdraw_tokens_from_partnership_wallet instruction.
///
/// This context is used to withdraw tokens from the partnership wallet.
///
/// The context includes:
/// - `contract_state` - the account that contains the contract state,
/// - `vesting_state` - the account that contains the vesting state,
/// - `partnership_account` - the partnership wallet account which is the source of tokens to be transferred,
/// - `deposit_wallet` - the destination account receiving tokens transferred from partnership_account,
/// - `signer` - the signer of the transaction which must be the contract's owner.
/// - `token_program` - the Solana token program account.
#[derive(Accounts)]
pub struct WithdrawTokensFromPartnershipWalletContext<'info> {
    #[account(
        mut,
        seeds = [CONTRACT_STATE_SEED.as_bytes()],
        bump = contract_state.contract_state_nonce,
    )]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(
        mut,
        seeds = [VESTING_STATE_SEED.as_bytes()],
        bump = vesting_state.vesting_state_nonce,
    )]
    pub vesting_state: Box<Account<'info, VestingState>>,

    #[account(
        mut,
        seeds = [PARTNERSHIP_ACCOUNT_SEED.as_bytes()],
        bump = vesting_state.partnership_wallet_nonce,
    )]
    pub partnership_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub deposit_wallet: Box<Account<'info, TokenAccount>>,

    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

/// Context for the withdraw_tokens_from_marketing_wallet instruction.
///
/// This context is used to withdraw tokens from the marketing wallet.
///
/// The context includes:
/// - `contract_state` - the account that contains the contract state,
/// - `vesting_state` - the account that contains the vesting state,
/// - `marketing_account` - the marketing wallet account which is the source of tokens to be transferred,
/// - `deposit_wallet` - the destination account receiving tokens transferred from marketing_account,
/// - `signer` - the signer of the transaction which must be the contract's owner.
/// - `token_program` - the Solana token program account.
#[derive(Accounts)]
pub struct WithdrawTokensFromMarketingWalletContext<'info> {
    #[account(
        mut,
        seeds = [CONTRACT_STATE_SEED.as_bytes()],
        bump = contract_state.contract_state_nonce,
    )]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(
        mut,
        seeds = [VESTING_STATE_SEED.as_bytes()],
        bump = vesting_state.vesting_state_nonce,
    )]
    pub vesting_state: Box<Account<'info, VestingState>>,

    #[account(
        mut,
        seeds = [MARKETING_ACCOUNT_SEED.as_bytes()],
        bump = vesting_state.marketing_wallet_nonce,
    )]
    pub marketing_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub deposit_wallet: Box<Account<'info, TokenAccount>>,

    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

/// Context for the withdraw_tokens_from_liquidity_wallet instruction.
///
/// This context is used to withdraw tokens from the liquidity wallet.
///
/// The context includes:
/// - `contract_state` - the account that contains the contract state,
/// - `vesting_state` - the account that contains the vesting state,
/// - `liquidity_account` - the community wallet account which is the source of tokens to be transferred,
/// - `deposit_wallet` - the destination account receiving tokens transferred from liquidity_account,
/// - `signer` - the signer of the transaction which must be the contract's owner.
/// - `token_program` - the Solana token program account.
#[derive(Accounts)]
pub struct WithdrawTokensFromLiquidityWalletContext<'info> {
    #[account(
        mut,
        seeds = [CONTRACT_STATE_SEED.as_bytes()],
        bump = contract_state.contract_state_nonce,
    )]
    pub contract_state: Box<Account<'info, ContractState>>,
    #[account(
        mut,
        seeds = [VESTING_STATE_SEED.as_bytes()],
        bump = vesting_state.vesting_state_nonce,
    )]
    pub vesting_state: Box<Account<'info, VestingState>>,

    #[account(
        mut,
        seeds = [LIQUIDITY_ACCOUNT_SEED.as_bytes()],
        bump = vesting_state.liquidity_wallet_nonce,
    )]
    pub liquidity_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub deposit_wallet: Box<Account<'info, TokenAccount>>,

    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

/// Generic vesting wallet context which is a trait to be implemented by all vesting wallet contexts where:
/// - `vested_account` refers to the account (wallet) who is the source of vested tokens that can be transferred, e.g. community account, partnership account, marketing account or liquidity account,
/// - `deposit_wallet` refers to the destination account who receives the tokens from `vested_account`,
/// - `token_program` refers to native Solana token program account.
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