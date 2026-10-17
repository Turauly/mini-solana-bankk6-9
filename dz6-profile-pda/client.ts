// Client для on-chain профиля на PDA
const user = pg.wallet.publicKey;
const [profilePda, bump] = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("profile"), user.toBuffer()],
  pg.PROGRAM_ID
);
const opts = { commitment: "confirmed" as const };
const link = (sig: string) =>
  `https://explorer.solana.com/tx/${sig}?cluster=devnet`;
const date = (t: any) => new Date(t.toNumber() * 1000).toISOString();

const show = async (label: string) => {
  const p = await pg.program.account.profile.fetch(profilePda, "confirmed");
  console.log(`--- ${label} ---`);
  console.log("username:", p.username);
  console.log("bio:", p.bio);
  console.log("authority:", p.authority.toBase58());
  console.log("created_at:", date(p.createdAt));
  console.log("updated_at:", date(p.updatedAt));
  console.log("bump:", p.bump);
};

console.log("Program ID:", pg.PROGRAM_ID.toBase58());
console.log("Мой кошелёк:", user.toBase58());
console.log("Profile PDA:", profilePda.toBase58(), "bump:", bump);

// 1. Создание профиля (если его ещё нет)
if ((await pg.connection.getAccountInfo(profilePda, "confirmed")) === null) {
  const tx = await pg.program.methods
    .createProfile("mukhammaddin", "Студент курса Solana-разработки")
    .accounts({
      profile: profilePda,
      user,
      systemProgram: web3.SystemProgram.programId,
    } as any)
    .rpc(opts);
  console.log("create_profile:", link(tx));
}
await show("После создания");

// 2. Обновление профиля
const upd = await pg.program.methods
  .updateProfile("mukhammaddin", "Изучаю Anchor, PDA и архитектуру программ")
  .accounts({ profile: profilePda, authority: user } as any)
  .rpc(opts);
console.log("update_profile:", link(upd));
await show("После обновления");

// 3. Проверка ошибки: слишком длинное имя
try {
  await pg.program.methods
    .updateProfile("x".repeat(40), "test")
    .accounts({ profile: profilePda, authority: user } as any)
    .rpc(opts);
} catch (e: any) {
  console.log("Ожидаемая ошибка:", e.error?.errorMessage ?? e.message);
}

// 4. У другого пользователя будет другой PDA
const other = web3.Keypair.generate().publicKey;
const [otherPda] = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("profile"), other.toBuffer()],
  pg.PROGRAM_ID
);
console.log("Другой кошелёк:", other.toBase58());
console.log("Его Profile PDA:", otherPda.toBase58());
