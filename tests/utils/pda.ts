import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { Leancoin } from "../../target/types/leancoin";

export const findProgramAddress = (key: string): [PublicKey, number] => {
    const program = anchor.workspace.Leancoin as anchor.Program<Leancoin>;
    let seed: Buffer[] = [Buffer.from(anchor.utils.bytes.utf8.encode(key))];

    const [_pda, _bump] = PublicKey.findProgramAddressSync(
        seed,
        program.programId,
    );

    return [_pda, _bump];
};
