// ДЗ 7: собственный токен на Token-2022 с метаданными (Token Extensions)
import * as spl from "@solana/spl-token";

const conn = pg.connection;
const payer = pg.wallet.keypair;
const opts = { commitment: "confirmed" as const };
const TP = spl.TOKEN_2022_PROGRAM_ID;
const link = (s: string) => `https://explorer.solana.com/tx/${s}?cluster=devnet`;
const DECIMALS = 6;
const UNIT = 10n ** BigInt(DECIMALS);
const recipient = new web3.PublicKey("EE2jNCaCkrhcmmaSDQh8JiygBsvBupMB11GGwUziLN8W"); // мой Phantom

// 1. Mint-аккаунт с расширением MetadataPointer (метаданные хранятся в самом mint)
const mint = web3.Keypair.generate();
const mintLen = spl.getMintLen([spl.ExtensionType.MetadataPointer]);
const rent = await conn.getMinimumBalanceForRentExemption(mintLen);
const createTx = new web3.Transaction().add(
  web3.SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: mint.publicKey,
    space: mintLen,
    lamports: rent,
    programId: TP,
  }),
  spl.createInitializeMetadataPointerInstruction(mint.publicKey, payer.publicKey, mint.publicKey, TP),
  spl.createInitializeMintInstruction(mint.publicKey, DECIMALS, payer.publicKey, null, TP)
);
const s1 = await web3.sendAndConfirmTransaction(conn, createTx, [payer, mint], opts);
console.log("Mint:", mint.publicKey.toBase58());
console.log("1) create mint + MetadataPointer:", link(s1));

// 2. Метаданные: name, symbol, uri + своё поле
const s2 = await spl.tokenMetadataInitializeWithRentTransfer(
  conn, payer, mint.publicKey, payer.publicKey, payer,
  "Mukhammaddin Token", "MUKH", "https://github.com/Turauly/mini-solana-bank",
  [], opts, TP
);
console.log("2) metadata initialize:", link(s2));
const s3 = await spl.tokenMetadataUpdateFieldWithRentTransfer(
  conn, payer, mint.publicKey, payer, "course", "Solana Dev — ДЗ 7", [], opts, TP
);
console.log("3) metadata update field:", link(s3));

// 3. Выпуск (mint) 1 000 000 MUKH на мой Associated Token Account
const myAta = await spl.getOrCreateAssociatedTokenAccount(conn, payer, mint.publicKey, payer.publicKey, false, "confirmed", opts, TP);
const s4 = await spl.mintTo(conn, payer, mint.publicKey, myAta.address, payer, 1_000_000n * UNIT, [], opts, TP);
console.log("My ATA:", myAta.address.toBase58());
console.log("4) mintTo 1 000 000 MUKH:", link(s4));

// 4. Transfer 2 500 MUKH на кошелёк Phantom (ATA создаётся автоматически)
const toAta = await spl.getOrCreateAssociatedTokenAccount(conn, payer, mint.publicKey, recipient, false, "confirmed", opts, TP);
const s5 = await spl.transferChecked(conn, payer, myAta.address, mint.publicKey, toAta.address, payer, 2_500n * UNIT, DECIMALS, [], opts, TP);
console.log("Recipient ATA:", toAta.address.toBase58());
console.log("5) transferChecked 2 500 MUKH:", link(s5));

// 5. Burn 500 MUKH
const s6 = await spl.burnChecked(conn, payer, myAta.address, mint.publicKey, payer, 500n * UNIT, DECIMALS, [], opts, TP);
console.log("6) burn 500 MUKH:", link(s6));

// 6. Чтение состояния из блокчейна
const fmt = (x: bigint) => (Number(x) / Number(UNIT)).toLocaleString("ru-RU");
const mintInfo = await spl.getMint(conn, mint.publicKey, "confirmed", TP);
const meta = await spl.getTokenMetadata(conn, mint.publicKey, "confirmed", TP);
const a1 = await spl.getAccount(conn, myAta.address, "confirmed", TP);
const a2 = await spl.getAccount(conn, toAta.address, "confirmed", TP);
console.log("--- Состояние ---");
console.log("supply:", fmt(mintInfo.supply), "MUKH, decimals:", mintInfo.decimals);
console.log("metadata:", meta?.name, "|", meta?.symbol, "|", meta?.uri, "| extra:", JSON.stringify(meta?.additionalMetadata));
console.log("Мой баланс:", fmt(a1.amount), "MUKH");
console.log("Баланс Phantom:", fmt(a2.amount), "MUKH");
console.log("Mint в Explorer:", `https://explorer.solana.com/address/${mint.publicKey.toBase58()}?cluster=devnet`);
