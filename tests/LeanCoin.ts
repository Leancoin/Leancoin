import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
    Connection,
    Keypair,
    PublicKey,
    Transaction,
    ComputeBudgetProgram,
} from "@solana/web3.js";

import bs58 from "bs58";
import { BN } from "bn.js";
import { Leancoin } from "../target/types/leancoin";
import { assert, expect } from "chai";
import * as dotenv from "dotenv";
import { findProgramAddress } from "./utils/pda";
import { getOrCreateAssociatedTokenAccount } from "./utils/accounts";
import { isBetween1and5 } from './utils/time';

dotenv.config();

describe("LeanCoin", () => {
    const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.Leancoin as Program<Leancoin>;

    let contract_state_address: anchor.web3.PublicKey = null;
    let contract_state_bump: number = null;

    let vesting_state_address: anchor.web3.PublicKey = null;
    let vesting_state_bump: number = null;

    let authority: anchor.web3.PublicKey = null;
    let authority_bump: number = null;

    let mint: anchor.web3.PublicKey = null;
    let mint_bump: number = null;

    let program_account_address: anchor.web3.PublicKey = null;
    let program_account_bump: number = null;

    let burning_account_address: anchor.web3.PublicKey = null;
    let burning_account_bump: number = null;

    let community_account_address: anchor.web3.PublicKey = null;
    let community_account_bump: number = null;

    let partnership_account_address: anchor.web3.PublicKey = null;
    let partnership_account_bump: number = null;

    let marketing_account_address: anchor.web3.PublicKey = null;
    let marketing_account_bump: number = null;

    let liquidity_account_address: anchor.web3.PublicKey = null;
    let liquidity_account_bump: number = null;

    let swap_account_address: anchor.web3.PublicKey = null;

    let swap_keypair: anchor.web3.Keypair = Keypair.generate();

    let rem_accounts: any = [];
    let user_info_ethereum_token_state_mapping = [];

    let amount_token_to_mint = new BN(0);
    let amount_token_to_burn = new BN(0);

    const payer = Keypair.fromSecretKey(bs58.decode("4VZjxpHWNaQMB6hZrzFTBJmmcb16ZCT3dVgq66DfbE4FfuJahzqhjWEnqnbqXfGejqufoQYZdxsNDHxmTcCoYj72"));
    const connection = new Connection("http://localhost:8899", "confirmed");

    describe("Initializes state", async () => {
        it("Initialize test state", async () => {
            [mint, mint_bump] = findProgramAddress("mint");

            [program_account_address, program_account_bump] =
                findProgramAddress("program_account");

            [burning_account_address, burning_account_bump] =
                findProgramAddress("burning_account");

            [contract_state_address, contract_state_bump] =
                findProgramAddress("contract_state");

            [community_account_address, community_account_bump] =
                findProgramAddress("community_account");

            [partnership_account_address, partnership_account_bump] =
                findProgramAddress("partnership_account");

            [marketing_account_address, marketing_account_bump] =
                findProgramAddress("marketing_account");

            [liquidity_account_address, liquidity_account_bump] =
                findProgramAddress("liquidity_account");

            [vesting_state_address, vesting_state_bump] =
                findProgramAddress("vesting_state");

            [authority, authority_bump] = findProgramAddress("authority");
        });

        it("should initialize the contract", async () => {
            const tx = await program.methods
                .initialize(
                    contract_state_bump,
                    vesting_state_bump,
                    mint_bump,
                    program_account_bump,
                    burning_account_bump,
                    community_account_bump,
                    liquidity_account_bump,
                    marketing_account_bump,
                    partnership_account_bump,
                )
                .accounts({
                    contractState: contract_state_address,
                    vestingState: vesting_state_address,
                    communityAccount: community_account_address,
                    liquidityAccount: liquidity_account_address,
                    marketingAccount: marketing_account_address,
                    partnershipAccount: partnership_account_address,
                    mint: mint,
                    programAccount: program_account_address,
                    burningAccount: burning_account_address,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    signer: provider.wallet.publicKey,
                    systemProgram: anchor.web3.SystemProgram.programId,
                })
                .rpc();
        });

        it("should show state", async () => {
            const vestingState = await program.account.vestingState.fetch(
                vesting_state_address,
            );
            expect(vestingState).to.exist;
        });

        it("Initialize token accounts", async () => {
            amount_token_to_mint = new BN("10000000000000000000");
            amount_token_to_burn = new BN("1470000000000000000");
            const amounts = {
                burn: new BN("1800000000000000000"), // 18% of total supply
                community: new BN("1000000000000000000"), // 10% of total supply
                partnership: new BN("2000000000000000000"), // 20% of total supply
                marketing: new BN("1500000000000000000"), // 15% of total supply
                liquidity: new BN("1000000000000000000"), // 10% of total supply
                swap: new BN("1230000000000000000"), // 12.3% of total supply
            };

            const createAccount = (name, address) => {
                user_info_ethereum_token_state_mapping.push({
                    walletName: name,
                    accountPublicKey: address,
                    accountBalance: amounts[name],
                });
                rem_accounts.push({
                    pubkey: address,
                    isWritable: true,
                    isSigner: false,
                });
            };

            // Burning account
            createAccount("burn", burning_account_address);

            // Community account
            createAccount("community", community_account_address);

            // Partnership account
            createAccount("partnership", partnership_account_address);

            // Marketing account
            createAccount("marketing", marketing_account_address);

            // Liquidity account
            createAccount("liquidity", liquidity_account_address);

            await connection.requestAirdrop(payer.publicKey, 1e9);
            await connection.requestAirdrop(swap_keypair.publicKey, 1e9);

            // Swap account
            let swapWalletAssociatedTokenAccount =
                await getOrCreateAssociatedTokenAccount(
                    provider,
                    mint,
                    provider.wallet.publicKey,
                    connection,
                );

            createAccount("swap", swapWalletAssociatedTokenAccount);

            swap_account_address = swapWalletAssociatedTokenAccount;
        });

        it("should import Ethereum token state", async () => {
            const tx = await program.methods
                .importEthereumTokenState(
                    user_info_ethereum_token_state_mapping,
                    amount_token_to_mint,
                    amount_token_to_burn,
                )
                .remainingAccounts(rem_accounts)
                .accounts({
                    contractState: contract_state_address,
                    vestingState: vesting_state_address,
                    mint: mint,
                    programAccount: program_account_address,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    signer: provider.wallet.publicKey,
                })
                .transaction();

            const additionalComputeBudgetInstruction =
                ComputeBudgetProgram.setComputeUnitLimit({
                    units: 500_000,
                });
            const transaction = new Transaction()
                .add(additionalComputeBudgetInstruction)
                .add(tx);

            await provider.sendAndConfirm(transaction, [], {
                commitment: "confirmed",
            });
        });

        it("secound ethereum_token_state_mapping!", async () => {
            try {
                const tx = await program.methods
                    .importEthereumTokenState(
                        user_info_ethereum_token_state_mapping,
                        amount_token_to_mint,
                        amount_token_to_burn,
                    )
                    .remainingAccounts(rem_accounts)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        mint: mint,
                        programAccount: program_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);

                await provider.sendAndConfirm(transaction, [], {
                    commitment: "confirmed",
                });
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0x1772",
                );
            }
        });

        it("Balance on accounts", async () => {
            let burning_account_balance =
                await connection.getTokenAccountBalance(
                    burning_account_address,
                );
            assert(
                burning_account_balance.value.amount == "1800000000000000000",
            );
            let liquidity_account_balance =
                await connection.getTokenAccountBalance(
                    liquidity_account_address,
                );
            assert(
                liquidity_account_balance.value.amount == "1000000000000000000",
            );
            let community_account_balance =
                await connection.getTokenAccountBalance(
                    community_account_address,
                );
            assert(
                community_account_balance.value.amount == "1000000000000000000",
            );
            let partnership_account_balance =
                await connection.getTokenAccountBalance(
                    partnership_account_address,
                );
            assert(
                partnership_account_balance.value.amount ==
                    "2000000000000000000",
            );
            let marketing_account_balance =
                await connection.getTokenAccountBalance(
                    marketing_account_address,
                );
            assert(
                marketing_account_balance.value.amount == "1500000000000000000",
            );
            let swap_account_balance = await connection.getTokenAccountBalance(
                swap_account_address,
            );
            assert(swap_account_balance.value.amount == "1230000000000000000");
        });

        it("Fail miss contract_state_address!", async () => {
            let fake_contract_state_address = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                const tx = await program.methods
                    .importEthereumTokenState(
                        user_info_ethereum_token_state_mapping,
                        amount_token_to_mint,
                        amount_token_to_burn,
                    )
                    .remainingAccounts(rem_accounts)
                    .accounts({
                        contractState: fake_contract_state_address,
                        vestingState: vesting_state_address,
                        mint: mint,
                        programAccount: program_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, [], {
                    commitment: "confirmed",
                });
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss mint!", async () => {
            let fake_mint = new PublicKey("11111111111111111111111111111111");

            try {
                const tx = await program.methods
                    .importEthereumTokenState(
                        user_info_ethereum_token_state_mapping,
                        amount_token_to_mint,
                        amount_token_to_burn,
                    )
                    .remainingAccounts(rem_accounts)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        mint: fake_mint,
                        programAccount: program_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss distribution_account_address!", async () => {
            let fake_program_account_address = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                const tx = await program.methods
                    .importEthereumTokenState(
                        user_info_ethereum_token_state_mapping,
                        amount_token_to_mint,
                        amount_token_to_burn,
                    )
                    .remainingAccounts(rem_accounts)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        mint: mint,
                        programAccount: fake_program_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss token_program!", async () => {
            let fake_token_program = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                const tx = await program.methods
                    .importEthereumTokenState(
                        user_info_ethereum_token_state_mapping,
                        amount_token_to_mint,
                        amount_token_to_burn,
                    )
                    .remainingAccounts(rem_accounts)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        mint: mint,
                        programAccount: program_account_address,
                        tokenProgram: fake_token_program,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbc0",
                );
            }
        });

        it("Fail miss system_program!", async () => {
            let fake_system_program = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                const tx = await program.methods
                    .importEthereumTokenState(
                        user_info_ethereum_token_state_mapping,
                        amount_token_to_mint,
                        amount_token_to_burn,
                    )
                    .remainingAccounts(rem_accounts)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        mint: mint,
                        programAccount: program_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0x1772",
                );
            }
        });

        it("Fail miss signer!", async () => {
            let fake_signer = new PublicKey("11111111111111111111111111111111");

            try {
                const tx = await program.methods
                    .importEthereumTokenState(
                        user_info_ethereum_token_state_mapping,
                        amount_token_to_mint,
                        amount_token_to_burn,
                    )
                    .remainingAccounts(rem_accounts)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        mint: mint,
                        programAccount: program_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: fake_signer,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(err.message, "Signature verification failed");
            }
        });

        it("Fail miss contract_state!", async () => {
            let fake_contract_state = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                const tx = await program.methods
                    .importEthereumTokenState(
                        user_info_ethereum_token_state_mapping,
                        amount_token_to_mint,
                        amount_token_to_burn,
                    )
                    .remainingAccounts(rem_accounts)
                    .accounts({
                        contractState: fake_contract_state,
                        vestingState: vesting_state_address,
                        mint: mint,
                        programAccount: program_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Burn Tokens!", async () => {
			if(isBetween1and5()) {
                let burning_accoount_balance_before = (await provider.connection.getTokenAccountBalance(burning_account_address)).value.amount;

                const tx = await program.methods
                .burn()
                .accounts({
                    mint: mint,
                    contractState: contract_state_address,
                    burningAccount: burning_account_address,
                    tokenProgram: TOKEN_PROGRAM_ID,
                })
                .rpc({ commitment: "confirmed" });

                let burning_accoount_balance_after = (await provider.connection.getTokenAccountBalance(burning_account_address)).value.amount;
                let expected_amount_burning_tokens = BigInt(burning_accoount_balance_before) / BigInt(20);

                assert.equal(BigInt(burning_accoount_balance_after), BigInt(burning_accoount_balance_before) - BigInt(expected_amount_burning_tokens));

			} else {
				console.error("The current date is not between the 1st and the 5th day of the month.")
			}
        });

        it("Fail miss contract_state_address!", async () => {
            let fake_contract_state_address = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                const tx = await program.methods
                    .burn()
                    .accounts({
                        mint: mint,
                        contractState: fake_contract_state_address,
                        burningAccount: burning_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                    })
                    .rpc();
            } catch (error) {
                assert.equal(
                    error.error.errorMessage,
                    "The given account is owned by a different program than expected",
                );
            }
        });

        it("Fail miss program_account_address!", async () => {
            let fake_program_account_address = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                const tx = await program.methods
                    .burn()
                    .accounts({
                        mint: mint,
                        contractState: contract_state_address,
                        burningAccount: fake_program_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                    })
                    .rpc();
            } catch (error) {
                assert.equal(
                    error.error.errorMessage,
                    "The given account is owned by a different program than expected",
                );
            }
        });

        it("Fail miss tokenProgram!", async () => {
            let fake_tokenProgram = new PublicKey(
                "11111111111111111111111111111111",
            );
            try {
                const tx = await program.methods
                    .burn()
                    .accounts({
                        mint: mint,
                        contractState: contract_state_address,
                        burningAccount: burning_account_address,
                        tokenProgram: fake_tokenProgram,
                    })
                    .rpc();
            } catch (error) {
                assert.equal(
                    error.error.errorMessage,
                    "Program ID was not as expected",
                );
            }
        });

        it("Withdraw Tokens From Community Wallet 0.000001000!", async () => {
            let swap_wallet_before_balance = 
			await connection.getTokenAccountBalance(
				swap_account_address,
			);
			let swap_wallet_before_token_balance = BigInt(swap_wallet_before_balance.value.amount);

            const tx = await program.methods
                .withdrawTokensFromCommunityWallet(new BN(1000))
                .accounts({
                    contractState: contract_state_address,
                    vestingState: vesting_state_address,
                    communityAccount: community_account_address,
                    depositWallet: swap_account_address,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    signer: provider.wallet.publicKey,
                })
                .transaction();

            const additionalComputeBudgetInstruction =
                ComputeBudgetProgram.setComputeUnitLimit({
                    units: 500_000,
                });
            const transaction = new Transaction()
                .add(additionalComputeBudgetInstruction)
                .add(tx);
            await provider.sendAndConfirm(transaction, [], {
                commitment: "confirmed",
            });

			let swap_wallet_after_balance = await connection.getTokenAccountBalance(
				swap_account_address,
			);
			let swap_wallet_after_token_balance = BigInt(swap_wallet_after_balance.value.amount);
			
			assert.equal( swap_wallet_after_token_balance - swap_wallet_before_token_balance, BigInt(1000));
        });

		it("Withdraw Tokens From Community Wallet 1!", async () => {
            let swap_wallet_before_balance = 
			await connection.getTokenAccountBalance(
				swap_account_address,
			);
			let swap_wallet_before_token_balance = BigInt(swap_wallet_before_balance.value.amount);

            const tx = await program.methods
                .withdrawTokensFromCommunityWallet(new BN(1000000000))
                .accounts({
                    contractState: contract_state_address,
                    vestingState: vesting_state_address,
                    communityAccount: community_account_address,
                    depositWallet: swap_account_address,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    signer: provider.wallet.publicKey,
                })
                .transaction();

            const additionalComputeBudgetInstruction =
                ComputeBudgetProgram.setComputeUnitLimit({
                    units: 500_000,
                });
            const transaction = new Transaction()
                .add(additionalComputeBudgetInstruction)
                .add(tx);
            await provider.sendAndConfirm(transaction, [], {
                commitment: "confirmed",
            });

			let swap_wallet_after_balance = await connection.getTokenAccountBalance(
				swap_account_address,
			);
			let swap_wallet_after_token_balance = BigInt(swap_wallet_after_balance.value.amount);
			
			assert.equal( swap_wallet_after_token_balance - swap_wallet_before_token_balance, BigInt(1000000000));
        });

		it("Fail Withdraw Tokens From Community Wallet 1000000000!", async () => {
            try {
                const tx = await program.methods
                    .withdrawTokensFromCommunityWallet(new BN("1000000000000000000"))
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        communityAccount: community_account_address,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0x1776",
                );
            }
        });

        it("Fail miss contract_state!", async () => {
            let fake_contract_state = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);

                const tx = await program.methods
                    .withdrawTokensFromCommunityWallet(amount_to_withdraw)
                    .accounts({
                        contractState: fake_contract_state,
                        vestingState: vesting_state_address,
                        communityAccount: community_account_address,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss vesting_state!", async () => {
            let fake_vesting_state = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);

                const tx = await program.methods
                    .withdrawTokensFromCommunityWallet(amount_to_withdraw)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: fake_vesting_state,
                        communityAccount: community_account_address,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss community_account!", async () => {
            let fake_community_account = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);

                const tx = await program.methods
                    .withdrawTokensFromCommunityWallet(amount_to_withdraw)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        communityAccount: fake_community_account,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });



        it("Withdraw Tokens From Partnership Wallet 0 tokens", async () => {
            let amount_to_withdraw = new BN(0);
            const tx = await program.methods
                .withdrawTokensFromPartnershipWallet(amount_to_withdraw)
                .accounts({
                    contractState: contract_state_address,
                    vestingState: vesting_state_address,
                    partnershipAccount: partnership_account_address,
                    depositWallet: swap_account_address,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    signer: provider.wallet.publicKey,
                })
                .transaction();

            const additionalComputeBudgetInstruction =
                ComputeBudgetProgram.setComputeUnitLimit({
                    units: 500_000,
                });
            const transaction = new Transaction()
                .add(additionalComputeBudgetInstruction)
                .add(tx);
            await provider.sendAndConfirm(transaction, [], {
                commitment: "confirmed",
            });
        });

        it("Withdraw Tokens From Partnership Wallet 0.000000100 tokens!", async () => {
            let amount_to_withdraw = new BN(100);

            try {
                const tx = await program.methods
                    .withdrawTokensFromPartnershipWallet(amount_to_withdraw)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        partnershipAccount: partnership_account_address,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, [], {
                    commitment: "confirmed",
                });
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0x1776",
                );
            }
        });

		it("Fail Withdraw Tokens From Partnership Wallet 1000000000 tokens", async () => {
			try {
			const tx = await program.methods
				.withdrawTokensFromPartnershipWallet( new BN("1000000000000000000"))
				.accounts({
					contractState: contract_state_address,
					vestingState: vesting_state_address,
					partnershipAccount: partnership_account_address,
					depositWallet: swap_account_address,
					tokenProgram: TOKEN_PROGRAM_ID,
					signer: provider.wallet.publicKey,
				})
				.transaction();

			const additionalComputeBudgetInstruction =
				ComputeBudgetProgram.setComputeUnitLimit({
					units: 500_000,
				});
			const transaction = new Transaction()
				.add(additionalComputeBudgetInstruction)
				.add(tx);
			await provider.sendAndConfirm(transaction, [], {
				commitment: "confirmed",
			});
			} catch (err) {
				assert.equal(
					err.message,
					"failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0x1776",
				);
			}
		});

        it("Fail miss contract_state!", async () => {
            let fake_contract_state = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);
                const tx = await program.methods
                    .withdrawTokensFromPartnershipWallet(amount_to_withdraw)
                    .accounts({
                        contractState: fake_contract_state,
                        vestingState: vesting_state_address,
                        partnershipAccount: partnership_account_address,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss vesting_state!", async () => {
            let fake_vesting_state = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);
                const tx = await program.methods
                    .withdrawTokensFromPartnershipWallet(amount_to_withdraw)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: fake_vesting_state,
                        partnershipAccount: partnership_account_address,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss partnership_account!", async () => {
            let fake_partnership_account = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);
                const tx = await program.methods
                    .withdrawTokensFromPartnershipWallet(amount_to_withdraw)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        partnershipAccount: fake_partnership_account,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Withdraw Tokens From Marketing Wallet 0 tokens!", async () => {
            let amount_to_withdraw = new BN(0);
            const tx = await program.methods
                .withdrawTokensFromMarketingWallet(amount_to_withdraw)
                .accounts({
                    contractState: contract_state_address,
                    vestingState: vesting_state_address,
                    marketingAccount: marketing_account_address,
                    depositWallet: swap_account_address,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    signer: provider.wallet.publicKey,
                })
                .transaction();

            const additionalComputeBudgetInstruction =
                ComputeBudgetProgram.setComputeUnitLimit({
                    units: 500_000,
                });
            const transaction = new Transaction()
                .add(additionalComputeBudgetInstruction)
                .add(tx);
            await provider.sendAndConfirm(transaction, []);
        });

		// fail because marketing wallet lockup time not expired
        it("Fail Withdraw Tokens From Marketing Wallet 0.000000100 tokens!", async () => {
            let amount_to_withdraw = new BN(100);
            try {
                const tx = await program.methods
                    .withdrawTokensFromMarketingWallet(amount_to_withdraw)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        marketingAccount: marketing_account_address,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0x1776",
                );
            }
        });

        it("Fail miss contract_state!", async () => {
            let fake_contract_state = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);
                const tx = await program.methods
                    .withdrawTokensFromMarketingWallet(amount_to_withdraw)
                    .accounts({
                        contractState: fake_contract_state,
                        vestingState: vesting_state_address,
                        marketingAccount: marketing_account_address,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss vesting_state!", async () => {
            let fake_vesting_state = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);
                const tx = await program.methods
                    .withdrawTokensFromMarketingWallet(amount_to_withdraw)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: fake_vesting_state,
                        marketingAccount: marketing_account_address,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss marketing_account!", async () => {
            let fake_marketing_account = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);
                const tx = await program.methods
                    .withdrawTokensFromMarketingWallet(amount_to_withdraw)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        marketingAccount: fake_marketing_account,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss marketing_wallet!", async () => {
            let fake_marketing_wallet = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);
                const tx = await program.methods
                    .withdrawTokensFromMarketingWallet(amount_to_withdraw)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        marketingAccount: marketing_account_address,
                        depositWallet: fake_marketing_wallet,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Withdraw Tokens From Liquidity Wallet!", async () => {
            let amount_to_withdraw = new BN(0);
            const tx = await program.methods
                .withdrawTokensFromLiquidityWallet(amount_to_withdraw)
                .accounts({
                    contractState: contract_state_address,
                    vestingState: vesting_state_address,
                    liquidityAccount: liquidity_account_address,
                    depositWallet: swap_account_address,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    signer: provider.wallet.publicKey,
                })
                .transaction();

            const additionalComputeBudgetInstruction =
                ComputeBudgetProgram.setComputeUnitLimit({
                    units: 500_000,
                });
            const transaction = new Transaction()
                .add(additionalComputeBudgetInstruction)
                .add(tx);
            await provider.sendAndConfirm(transaction, []);
        });

        it("Fail miss contract_state!", async () => {
            let fake_contract_state = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);
                const tx = await program.methods
                    .withdrawTokensFromLiquidityWallet(amount_to_withdraw)
                    .accounts({
                        contractState: fake_contract_state,
                        vestingState: vesting_state_address,
                        liquidityAccount: liquidity_account_address,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss vesting_state!", async () => {
            let fake_vesting_state = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);
                const tx = await program.methods
                    .withdrawTokensFromLiquidityWallet(amount_to_withdraw)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: fake_vesting_state,
                        liquidityAccount: liquidity_account_address,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Fail miss liquidity_account!", async () => {
            let fake_liquidity_account = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                let amount_to_withdraw = new BN(0);
                const tx = await program.methods
                    .withdrawTokensFromLiquidityWallet(amount_to_withdraw)
                    .accounts({
                        contractState: contract_state_address,
                        vestingState: vesting_state_address,
                        liquidityAccount: fake_liquidity_account,
                        depositWallet: swap_account_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);
                await provider.sendAndConfirm(transaction, []);
            } catch (err) {
                assert.equal(
                    err.message,
                    "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbbf",
                );
            }
        });

        it("Pass change Authority", async () => {
            let new_authority = new PublicKey(
                "11111111111111111111111111111111",
            );

            let contract_state_account = await program.account.contractState.fetch(
                contract_state_address,
            );

            assert.equal(
                contract_state_account.authority.toBase58(),
                provider.wallet.publicKey.toBase58(),
            );

            await program.methods
                .changeAuthority(new_authority)
                .accounts({
                    contractState: contract_state_address,
                    signer: provider.wallet.publicKey,
                })
                .rpc();

            contract_state_account = await program.account.contractState.fetch(
                contract_state_address,
            );

            assert.equal(
                contract_state_account.authority.toBase58(),
                new_authority.toBase58(),
            );
        });

        it("Fail set change Authority", async () => {
            let new_authority = new PublicKey(
                "11111111111111111111111111111111",
            );

            try {
                await program.methods
                    .changeAuthority(new_authority)
                    .accounts({
                        contractState: contract_state_address,
                        signer: new_authority,
                    })
                    .rpc();

            } catch (err) {
                assert.equal(
                    err.message,
                    "Signature verification failed",
                );
            }
        });
    });
});
