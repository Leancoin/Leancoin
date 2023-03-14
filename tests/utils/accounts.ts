import * as anchor from "@coral-xyz/anchor";
import {
    getAssociatedTokenAddress,
    createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { Transaction, Connection } from "@solana/web3.js";

export const getOrCreateAssociatedTokenAccount = async (
    provider: anchor.AnchorProvider,
    mint: anchor.web3.PublicKey,
    owner: anchor.web3.PublicKey,
    connection: Connection,
): Promise<anchor.web3.PublicKey> => {
    const associatedTokenAddress = await getAssociatedTokenAddress(mint, owner);

    let info = await connection.getAccountInfo(associatedTokenAddress);

    if (info == null) {
        let createAssociatedTokenAccountInstruction_ =
            createAssociatedTokenAccountInstruction(
                provider.wallet.publicKey,
                associatedTokenAddress,
                owner,
                mint,
            );

        const tx = new Transaction();
        tx.add(createAssociatedTokenAccountInstruction_);
        await provider.sendAndConfirm(tx, []);
    }

    return associatedTokenAddress;
};
