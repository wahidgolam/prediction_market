import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import type { HelloAnchor } from "../target/types/hello_anchor";

// Configure the client to use the local cluster
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.HelloAnchor as anchor.Program<HelloAnchor>;

// Client
console.log("My address:", program.provider.publicKey.toString());
const balance = await program.provider.connection.getBalance(program.provider.publicKey);
console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL \n`);

console.log(program.programId.toString());

const id = "Ac2ag21ved7vgs8vs9ds8vs9d8vu2"
//await initMarket(id);

await takePosition(id, false, 1);

async function initMarket(id){
  const metadata = {
    id: id
  };
  const [market_pda] = await anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(metadata.id)],
    program.programId
  );

  const context = {
    market: market_pda,
    signer: program.provider.publicKey,
    rent: web3.SYSVAR_RENT_PUBKEY,
    systemProgram: web3.SystemProgram.programId,
  };

  let latestBlockhash = await program.provider.connection.getLatestBlockhash('finalized');


  const tx = await program.methods.createMarket(
    metadata
  )
  .accounts(context)
  .rpc()
  //.catch(e => console.error(e));

  await program.provider.connection.confirmTransaction({
    signature: tx,
    blockhash: latestBlockhash.blockhash,
    lastValidBlockHeight: latestBlockhash.lastValidBlockHeight
  });

  console.log(`https://explorer.solana.com/tx/${tx}?cluster=devnet`);

  const data = await program.account.market.fetch(market_pda);
  const reserve = (
      await program.provider.connection.getBalance(market_pda)
    );
  console.log(`ID: `,data.id.toString());
  console.log(`Reserve: `, reserve);

}

async function takePosition(id, position: boolean, number){
  const metadata2 = {
    marketId: id,
    positionTypeString: (position)?"yes":"no",
    positionType: position,
    number: number
  };

  const [market_pda] = await anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(metadata2.marketId)],
    program.programId
  );

    const [position_pda] = await anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(metadata2.marketId), program.provider.publicKey.toBuffer(), Buffer.from(metadata2.positionTypeString)],
    program.programId
  );

  const context = {
    market: market_pda,
    position: position_pda,
    signer: program.provider.publicKey,
    rent: web3.SYSVAR_RENT_PUBKEY,
    systemProgram: web3.SystemProgram.programId,
  };

  let latestBlockhash = await program.provider.connection.getLatestBlockhash('finalized');


  const tx = await program.methods.takePosition(
    metadata2
  )
  .accounts(context)
  .rpc()
  // //.catch(e => console.error(e));

  await program.provider.connection.confirmTransaction({
    signature: tx,
    blockhash: latestBlockhash.blockhash,
    lastValidBlockHeight: latestBlockhash.lastValidBlockHeight
  });

  console.log(`https://explorer.solana.com/tx/${tx}?cluster=devnet`);

  const data = await program.account.market.fetch(market_pda);
  const reserve = (
      await program.provider.connection.getBalance(market_pda)
    );
  console.log(`Market ID: `,data.id.toString());
  console.log(`Total Supply of Yes tokens: `,data.supplyY.toString());
  console.log(`Total Supply of No tokens: : `,data.supplyN.toString());
  console.log(`Last Price of Yes token: `,data.priceY.toString());
  console.log(`Last Price of No token: `,data.priceN.toString());
  console.log(`Reserve: `, reserve);

  const data2 = await program.account.position.fetch(position_pda);
  console.log(`Position Taker: `,data2.taker.toString());
  console.log(`Position Type: `,data2.positionType.toString());
  console.log(`Position Number of Tokens: `,data2.number.toString());
  console.log(`Position Change in Reserve: `,data2.delReserve.toString());

}
