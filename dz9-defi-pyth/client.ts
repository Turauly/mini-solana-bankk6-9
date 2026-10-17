// ДЗ 9: DeFi-интеграция в Tip Jar — оракул Pyth (SOL/USD) on-chain
const owner = pg.wallet.keypair;
const conn = pg.connection;
const opts = { commitment: "confirmed" as const };
const link = (s: string) => `https://explorer.solana.com/tx/${s}?cluster=devnet`;
const SOL = web3.LAMPORTS_PER_SOL;
// Аккаунт PriceUpdateV2 для SOL/USD (Pyth push oracle, одинаковый адрес в devnet и mainnet)
const PYTH_SOL_USD = new web3.PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");

const [jarPda] = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("jar"), owner.publicKey.toBuffer()],
  pg.PROGRAM_ID
);

// 1. Читаем цену Pyth на клиенте (тот же разбор, что и в программе)
const readPyth = async () => {
  const acc = await conn.getAccountInfo(PYTH_SOL_USD, "confirmed");
  const d = acc!.data;
  let o = 8 + 32;
  o += d[o] === 0 ? 2 : 1;
  const feedId = d.subarray(o, o + 32).toString("hex");
  o += 32;
  const price = Number(d.readBigInt64LE(o));
  const conf = Number(d.readBigUInt64LE(o + 8));
  const expo = d.readInt32LE(o + 16);
  const publishTime = Number(d.readBigInt64LE(o + 20));
  return { owner: acc!.owner.toBase58(), feedId, price, conf, expo, publishTime };
};
const p = await readPyth();
const solUsd = p.price * 10 ** p.expo;
console.log("Pyth account owner:", p.owner);
console.log("feed id:", p.feedId);
console.log(`SOL/USD = $${solUsd.toFixed(4)} ± $${(p.conf * 10 ** p.expo).toFixed(4)}, publish_time ${new Date(p.publishTime * 1000).toISOString()}`);

// 2. Донат $2: программа сама читает Pyth и пересчитывает в lamports
const before = await conn.getBalance(jarPda, "confirmed");
const tx = await pg.program.methods
  .donateUsd(new BN(200), "Донат $2 по курсу Pyth")
  .accounts({
    jar: jarPda,
    donor: owner.publicKey,
    priceUpdate: PYTH_SOL_USD,
    systemProgram: web3.SystemProgram.programId,
  } as any)
  .rpc(opts);
const after = await conn.getBalance(jarPda, "confirmed");
console.log("donate_usd $2:", link(tx));
console.log(`Получено копилкой: ${after - before} lamports = ${((after - before) / SOL).toFixed(6)} SOL`);
console.log(`Проверка: $2 / $${solUsd.toFixed(4)} = ${(2 / solUsd).toFixed(6)} SOL`);

// 3. Логи программы: цена, которую увидела программа
const t = await conn.getTransaction(tx, { commitment: "confirmed", maxSupportedTransactionVersion: 0 });
t?.meta?.logMessages?.filter((l) => l.includes("Donate USD")).forEach((l) => console.log(l));

// 4. Стоимость копилки в долларах
const jar = await pg.program.account.jar.fetch(jarPda, "confirmed");
console.log(`Копилка: ${jar.donations.toString()} донатов, всего ${(jar.totalDonated.toNumber() / SOL).toFixed(6)} SOL`);
console.log(`Баланс PDA: ${(after / SOL).toFixed(6)} SOL ≈ $${((after / SOL) * solUsd).toFixed(2)}`);
console.log(`Прогресс к цели 1 SOL: ${((jar.totalDonated.toNumber() / jar.goal.toNumber()) * 100).toFixed(1)}%`);

// 5. Защита: подставить чужой аккаунт вместо оракула
try {
  await pg.program.methods
    .donateUsd(new BN(100), "фальшивая цена")
    .accounts({ jar: jarPda, donor: owner.publicKey, priceUpdate: jarPda, systemProgram: web3.SystemProgram.programId } as any)
    .rpc(opts);
} catch (e: any) {
  console.log("Фальшивый оракул отклонён:", e.error?.errorMessage ?? e.message?.slice(0, 120));
}
