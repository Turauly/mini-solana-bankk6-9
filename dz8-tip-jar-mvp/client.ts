// ДЗ 8: MVP "Tip Jar" — копилка донатов на Solana
const owner = pg.wallet.keypair;
const conn = pg.connection;
const opts = { commitment: "confirmed" as const };
const link = (s: string) => `https://explorer.solana.com/tx/${s}?cluster=devnet`;
const SOL = web3.LAMPORTS_PER_SOL;
const [jarPda] = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("jar"), owner.publicKey.toBuffer()],
  pg.PROGRAM_ID
);
const show = async (label: string) => {
  const j = await pg.program.account.jar.fetch(jarPda, "confirmed");
  const bal = await conn.getBalance(jarPda, "confirmed");
  console.log(`--- ${label} ---`);
  console.log(`title: ${j.title} | goal: ${j.goal.toNumber() / SOL} SOL`);
  console.log(`total_donated: ${j.totalDonated.toNumber() / SOL} SOL | donations: ${j.donations.toString()}`);
  console.log(`last_donor: ${j.lastDonor.toBase58()} | last_message: "${j.lastMessage}"`);
  console.log(`баланс PDA: ${bal / SOL} SOL`);
};

console.log("Program ID:", pg.PROGRAM_ID.toBase58());
console.log("Jar PDA:", jarPda.toBase58());

// 1. Создать копилку (один раз)
if ((await conn.getAccountInfo(jarPda, "confirmed")) === null) {
  const tx = await pg.program.methods
    .createJar("Копилка на хакатон Solana", new BN(Math.round(1 * SOL)))
    .accounts({ jar: jarPda, owner: owner.publicKey, systemProgram: web3.SystemProgram.programId } as any)
    .rpc(opts);
  console.log("create_jar:", link(tx));
}

// 2. Донат от владельца-кошелька
const d1 = await pg.program.methods
  .donate(new BN(Math.round(0.05 * SOL)), "Первый донат!")
  .accounts({ jar: jarPda, donor: owner.publicKey, systemProgram: web3.SystemProgram.programId } as any)
  .rpc(opts);
console.log("donate 0.05 SOL:", link(d1));

// 3. Второй пользователь: новый кошелёк получает 0.05 SOL и донатит 0.02 SOL
const friend = web3.Keypair.generate();
const fund = await web3.sendAndConfirmTransaction(
  conn,
  new web3.Transaction().add(
    web3.SystemProgram.transfer({ fromPubkey: owner.publicKey, toPubkey: friend.publicKey, lamports: Math.round(0.05 * SOL) })
  ),
  [owner],
  opts
);
console.log("Друг:", friend.publicKey.toBase58(), "пополнен:", link(fund));
const d2 = await pg.program.methods
  .donate(new BN(Math.round(0.02 * SOL)), "Удачи на хакатоне!")
  .accounts({ jar: jarPda, donor: friend.publicKey, systemProgram: web3.SystemProgram.programId } as any)
  .signers([friend])
  .rpc(opts);
console.log("donate 0.02 SOL от друга:", link(d2));
await show("После донатов");

// 4. Владелец выводит 0.03 SOL
const w = await pg.program.methods
  .withdraw(new BN(Math.round(0.03 * SOL)))
  .accounts({ jar: jarPda, owner: owner.publicKey } as any)
  .rpc(opts);
console.log("withdraw 0.03 SOL:", link(w));
await show("После вывода");

// 5. Проверки безопасности
try {
  await pg.program.methods
    .withdraw(new BN(Math.round(0.01 * SOL)))
    .accounts({ jar: jarPda, owner: friend.publicKey } as any)
    .signers([friend])
    .rpc(opts);
} catch (e: any) {
  console.log("Чужой withdraw отклонён:", e.error?.errorMessage ?? e.message?.slice(0, 120));
}
try {
  await pg.program.methods
    .withdraw(new BN(Math.round(100 * SOL)))
    .accounts({ jar: jarPda, owner: owner.publicKey } as any)
    .rpc(opts);
} catch (e: any) {
  console.log("Слишком большой withdraw:", e.error?.errorMessage ?? e.message?.slice(0, 120));
}
