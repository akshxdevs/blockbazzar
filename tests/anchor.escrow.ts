import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EcomDapp } from "../target/types/ecom_dapp";
import { LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { expect } from "chai";
import { BN } from "@coral-xyz/anchor";
import { 
  TOKEN_PROGRAM_ID,           
} from "@solana/spl-token";

describe("anchor", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.EcomDapp as Program<EcomDapp>;
  const signer = provider.wallet as anchor.Wallet;

  //test pubkey's
  let owner = new PublicKey("2hZmn6pHMPP8N2QXyhWn2tHXbM4kc3QJ4Agj57HWtYkD");
  owner.toBytes();
  let seller = new PublicKey("6fCZ1ie5PxdcsS3J87cW4BcdtAqMr7VWk9QNo2jskgrt");
  seller.toBytes();
  let buyer = new PublicKey("GXrTGkUU17MGwpMW7fqgh65xWECvtNtXeaBkEFeeb42s");
  buyer.toBytes();

  const PAYMENT_AMOUNT_USD = 200;
  const getSolPrice = async (usdAmount: number): Promise<number> => {
    const res = await fetch(
      "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd"
    );
    const data = (await res.json()) as { solana: { usd: number } };
    
    const solPrice = data.solana.usd;
    const solAmount = usdAmount / solPrice;
    console.log({solAmount});
    
    return Math.round(solAmount * LAMPORTS_PER_SOL);
  };
  let USD_TO_SOL:Number;
  let PAYMENT_AMOUNT:Number;
  before(async()=>{
    USD_TO_SOL = await getSolPrice(PAYMENT_AMOUNT_USD);
    console.log({USD_TO_SOL});
    PAYMENT_AMOUNT = Number(USD_TO_SOL);
    console.log("Payment Amount: ",(Number(PAYMENT_AMOUNT) / LAMPORTS_PER_SOL).toFixed(4)+ " SOL");
  });  

  let mint:anchor.web3.PublicKey;
  let escrowAta:anchor.web3.PublicKey;
  let sellerAta:anchor.web3.PublicKey;
  let buyerAta:anchor.web3.PublicKey;
  let userAta:anchor.web3.PublicKey;
  let payment_tx:string;
  let escrowPda: PublicKey;
  let paymentPda: PublicKey;

  const fundIfNeeded = async (pubkey: PublicKey) => {
      const bal = await provider.connection.getBalance(pubkey);
      if (bal < 0.5 * LAMPORTS_PER_SOL) {
        const sig = await provider.connection.requestAirdrop(
          pubkey,
          2 * LAMPORTS_PER_SOL
        );
        await provider.connection.confirmTransaction(sig);
      }
    };

  before(async () => {
  await Promise.all([
    fundIfNeeded(buyer),
    fundIfNeeded(seller),
  ]);
  });

  function bytesToUuid(bytes: number[]): string {
    if (bytes.length !== 16) throw new Error("Invalid UUID length");

    const hex = bytes.map((b) => b.toString(16).padStart(2, "0")).join("");
    return [
      hex.slice(0, 8),
      hex.slice(8, 12),
      hex.slice(12, 16),
      hex.slice(16, 20),
      hex.slice(20),
    ].join("-");
  }

  // it("creates a mint", async () => {
  //   const mintKp = anchor.web3.Keypair.generate();
  //   const lamports = await provider.connection.getMinimumBalanceForRentExemption(82);

  //   const tx = new Transaction().add(
  //     SystemProgram.createAccount({
  //       fromPubkey: owner,
  //       newAccountPubkey: mintKp.publicKey,
  //       space: 82,
  //       lamports,
  //       programId: TOKEN_PROGRAM_ID,
  //     }),
  //     createInitializeMintInstruction(
  //       mintKp.publicKey,
  //       9,
  //       owner,   
  //       null               
  //     )
  //   );

  //   await provider.sendAndConfirm(tx, [mintKp]);
  //   mint = mintKp.publicKey;
  //   console.log("Mint Address: ",mint);
    
  // });

  // it("creates user ATAs", async () => {
  //   if (owner) {
  //     [escrowPda] = PublicKey.findProgramAddressSync(
  //       [Buffer.from("escrow"), owner.toBuffer()],
  //       program.programId
  //     );
  //   }else{
  //     console.error("Owner error");
      
  //   }
  //   userAta = getAssociatedTokenAddressSync(mint, owner);
  //   buyerAta = getAssociatedTokenAddressSync(mint, buyer);
  //   sellerAta = getAssociatedTokenAddressSync(mint, seller);
  //   escrowAta = getAssociatedTokenAddressSync(mint,escrowPda,true);
  //   const tx = new Transaction().add(
  //     createAssociatedTokenAccountInstruction(
  //       owner,
  //       userAta,
  //       owner,
  //       mint
  //     ),
  //     createAssociatedTokenAccountInstruction(
  //       owner,
  //       buyerAta,
  //       buyer,
  //       mint
  //     ),
  //     createAssociatedTokenAccountInstruction(
  //       owner,
  //       sellerAta,
  //       seller,
  //       mint
  //     ),
  //     createAssociatedTokenAccountInstruction(
  //       owner,
  //       escrowAta,
  //       escrowPda,
  //       mint
  //     )
  //   );

  //   await provider.sendAndConfirm(tx);
  //   console.log("User Ata: ",userAta.toString());
  //   console.log("Escrow Ata: ",escrowAta.toString());
  //   console.log("Buyer Ata: ",buyer.toString());
  //   console.log("Seller Ata: ",sellerAta.toString());
  // });

  // it("mints tokens to owner", async () => {
  //   const amount = new BN(200).mul(new BN(10 ** 9)); 

  //   const tx = new Transaction().add(
  //     createMintToInstruction(mint, userAta, owner, Number(amount), [])
  //   );

  //   await provider.sendAndConfirm(tx);
  //   const user = await provider.connection.getAccountInfo(userAta);
  //   console.log("user balance: ", user.lamports / 1_000_000_000);
    
  // });

  it("creates payment PDA", async () => {
    const amount = new BN(Number(PAYMENT_AMOUNT));
    console.log("Amount: ",amount.toNumber());
    const [newPaymentPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("payment"), owner.toBuffer()],
    program.programId
    );
    console.log("Payment PDA created", newPaymentPda);
    await program.methods
      .createPayment(amount, newPaymentPda, null)
      .accounts({
        signer: owner,
        payments: newPaymentPda,
        systemProgram: SystemProgram.programId,
      }as any)
      .rpc();
      paymentPda = newPaymentPda;
      const paymentDetails = await program.account.payment.fetch(newPaymentPda);
      console.log("Payment Status: ",paymentDetails.paymentStatus);
      console.log("Payment PDA created", paymentPda);
      expect(paymentDetails.paymentStatus).to.deep.equal({pending : {}});
  });


  it("creates escrow", async () => {
    [escrowPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), owner.toBuffer()],
      program.programId
    );
    try {
      const escrowDetails = await program.account.escrow.fetch(escrowPda);
      console.log("Escrow Details: ",escrowDetails.escrowStatus);
      if ("swapSuccess" in escrowDetails.escrowStatus) {
        await program.methods.closeEscrow().accounts({
          signer:owner,
          escrow:escrowPda
        }as any).rpc();
        console.log("Existing escrow account closed successfully..");
      }
    } catch (error) {
      const amount = new BN(Number(PAYMENT_AMOUNT));
      await program.methods
        .createEscrow(buyer, seller, amount)
        .accounts({
          owner: owner,
          escrow: escrowPda,
          payment: paymentPda,
          userAta,
          escrowAta,
          buyerAta,
          sellerAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        }as any)
        .rpc();
        const userDetails = await provider.connection.getBalance(owner);
        console.log("User Details: ",userDetails / LAMPORTS_PER_SOL + " SOL");
        const escrowDetails = await program.account.escrow.fetch(escrowPda);
        console.log("Escrow Details: ",escrowDetails.escrowStatus);
        expect(Number(escrowDetails.amount)).to.equal(USD_TO_SOL);
    }
  });

  it("deposits into escrow", async () => {
    try {
      const payment = await program.account.payment.fetch(paymentPda);
      console.log("Payment account:", payment);
      console.log("Payment status:", payment.paymentStatus);
      console.log("Payment amount:", Number(payment.paymentAmount));
    } catch (err) {
      throw new Error(`Payment PDA not initialized: ${err.message}`);
    }
    console.log("EscrowPda: ",escrowPda);
    console.log("paymentPda: ",paymentPda);
    const [vaultPda,vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), owner.toBuffer()],
      program.programId
    );
    const tx = await program.methods
      .depositEscrow(vaultBump)
      .accounts({
        owner: owner,
        escrow: escrowPda,
        payment: paymentPda,
        vaultAccount:vaultPda,
        escrowAccount:escrowPda,
        userAccount:owner,
        systemProgram: SystemProgram.programId,
      }as any)
      .rpc();

    console.log("Deposit tx:", tx);

    const escrow = await program.account.escrow.fetch(escrowPda);
    expect(escrow.escrowStatus).to.deep.equal({ fundsReceived: {} });
    expect(escrow.releaseFund).to.be.true;

    const userBal = await provider.connection.getBalance(owner);
    const vaultBal = await provider.connection.getBalance(vaultPda);
    
      
    console.log("Account Balances:");
    console.log(`Vault (${escrowPda.toString()}): ${vaultBal} SOL`);
    console.log(`User (${owner.toString()}): ${userBal} SOL`);
    const payment = await program.account.payment.fetch(paymentPda);

    expect(payment.paymentMethod).to.deep.equal({sol : {}})
  });

it("withdraws from escrow", async () => {
    const [vaultPda,vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), owner.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .withdrawEscrow(vaultBump)
      .accounts({
        owner: owner,
        escrow: escrowPda,
        payment: paymentPda,
        vaultAccount:vaultPda,
        escrowAccount:escrowPda,
        sellerAccount:seller,
        systemProgram: SystemProgram.programId,
      }as any)
      .rpc();

    console.log("Withdraw tx:", tx);
    const sellerBal = await provider.connection.getBalance(seller);
    const vaultBal = await provider.connection.getBalance(vaultPda);
    
      
    console.log("Account Balances:");
    console.log(`Vault (${escrowPda.toString()}): ${vaultBal} SOL`);
    console.log(`Seller (${owner.toString()}): ${sellerBal} SOL`);

    const escrow = await program.account.escrow.fetch(escrowPda);

    expect(escrow.escrowStatus).to.deep.equal({ swapSuccess: {} });
    expect(escrow.releaseFund).to.be.false;

    const payment = await program.account.payment.fetch(paymentPda);
    expect(payment.paymentStatus).to.deep.equal({ success: {} });

    // expect(Number(sellerBal.value.amount)).to.equal(PAYMENT_AMOUNT);
    if (expect(payment.paymentStatus).to.have.property("success")) {
      payment.txSignature = payment_tx;
    }else{
      console.log("Payment Not Intialized!");
    }
    console.log("Swap Payment Completed successfully..");
    console.log("payment invoice: ",payment);
  });

  
  it("should place order and show details",async()=>{
    const [orderPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("order"), signer.publicKey.toBuffer()],
      program.programId
    );console.log("Order PDA:", orderPda.toBase58());

    try {
      const existingPayment = await program.account.payment.fetch(paymentPda);
      const existingOrder = await program.account.order.fetch(orderPda);
      
      if (existingOrder && existingPayment) {
        const order = await program.account.order.fetch(orderPda);
        console.log("Order details: ",order);
        console.log("Order ID: ", bytesToUuid(order.orderId));
        console.log("Order Status: ",order.orderStatus);
        console.log("Order Tracking: ",order.orderTracking);
  
        expect(order.orderStatus).to.have.property("placed");
  
      }
    } catch (error) {
      const payment_id = (await program.account.payment.fetch(paymentPda)).paymentId;
      const order_tx = await program.methods.createOrder(
        String(bytesToUuid(payment_id)),
      ).accounts({
        signer: signer.publicKey,
        order: orderPda,
        payment:paymentPda,
        systemProgram: SYSTEM_PROGRAM_ID,
      } as any).rpc();

      console.log("Transaction Signature: ",order_tx);
      console.log("Order status placed Successfully...");
      
      const order = await program.account.order.fetch(orderPda);

      console.log("Order details: ",order);
      console.log("Order ID: ", bytesToUuid(order.orderId));
      console.log("Order Status: ",order.orderStatus);
      console.log("Order Tracking: ",order.orderTracking);

      expect(order.orderStatus).to.have.property("placed");
    }
  });

  it("should update order status to in-transit and show details",async()=>{
    const [orderPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("order"), signer.publicKey.toBuffer()],
      program.programId
    );console.log("Order PDA:", orderPda.toBase58());

    const [paymentPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("payment"), owner.toBuffer()],
    program.programId
    );console.log("Payment PDA: ",paymentPda);

    const existingPayment = await program.account.payment.fetch(paymentPda);
    const existingOrder = await program.account.order.fetch(orderPda);
    
    if (existingPayment && existingOrder) {
      const orderStatusUpdate = "intransit";
      try {
        await program.methods.updateOrder(
          orderStatusUpdate
        ).accounts({
          signer:signer.publicKey,
          order:orderPda
        }as any).rpc()
        console.log("Order status updated Successfully...");
        console.log("Order Tracking: ",existingOrder.orderTracking);
      } catch (error) {
        console.error(error);
      }
  }
  });

  it("should update order status to shipped and show details",async()=>{
    const [orderPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("order"), signer.publicKey.toBuffer()],
      program.programId
    );console.log("Order PDA:", orderPda.toBase58());

    const [paymentPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("payment"), owner.toBuffer()],
    program.programId
    );console.log("Payment PDA: ",paymentPda);

    const existingPayment = await program.account.payment.fetch(paymentPda);
    const existingOrder = await program.account.order.fetch(orderPda);
    
    if (existingPayment && existingOrder) {
      const orderStatusUpdate = "shipped";
      try {
        await program.methods.updateOrder(
          orderStatusUpdate
        ).accounts({
          signer:signer.publicKey,
          order:orderPda
        }as any).rpc()
        console.log("Order status updated Successfully...");
        console.log("Order Tracking: ",existingOrder.orderTracking);
      } catch (error) {
        console.error(error);
      }
  }
  });
  
  it("should place order and show details",async()=>{
    const [orderPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("order"), signer.publicKey.toBuffer()],
      program.programId
    );console.log("Order PDA:", orderPda.toBase58());

    const [paymentPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("payment"), owner.toBuffer()],
    program.programId
    );console.log("Payment PDA: ",paymentPda);

    const existingPayment = await program.account.payment.fetch(paymentPda);
    const existingOrder = await program.account.order.fetch(orderPda);
    
    if (existingOrder && existingPayment) {
      const order = await program.account.order.fetch(orderPda);
      console.log("Order details: ",order);
      console.log("Order ID: ", bytesToUuid(order.orderId));
      console.log("Order Status: ",order.orderStatus);
      console.log("Order Tracking: ",order.orderTracking);

      expect(order.orderTracking).to.have.property("shipped");
    }
  });

  it("Close Payment PDA..",async()=>{
    console.log(`Deleting Account Pda ${paymentPda} of ${owner.toString()}`);
    const closeAccount =  await program.methods.closePayment().accounts({
      signer: owner,
      payments: paymentPda,
    }as any).rpc();
      console.log(`Account(${paymentPda}) Closed Successfully! `,closeAccount);
  });

});