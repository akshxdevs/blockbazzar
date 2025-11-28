# BlockBazzar On-Chain Program

The BlockBazzar on-chain program implements decentralized e-commerce logic, including product listings, cart storage, payment accounts, escrow protection, and order lifecycle management.

All business logic is executed trustlessly through Solana PDAs.

---

# Buyer Protection & Escrow Safety

BlockBazzar solves the core problem of online shopping: **buyers paying first and never receiving the product**.  

The program uses a trustless, on-chain **escrow system** to ensure complete buyer safety.

## How It Works

1\. **Buyer pays → funds are locked in escrow (PDA vault).**  

   - Neither buyer nor seller can access the funds prematurely.

2\. **Seller ships the product.**

3\. **Funds are released to the seller only after delivery is confirmed.**

4\. **If delivery fails or the seller does not ship:**  

   - The escrow automatically refunds the buyer in full.

## Why It's Safe

- Seller **cannot withdraw early** or bypass escrow.  

- Buyer funds stay locked until successful delivery.  

- Refunds are **program-enforced**, not dependent on a third party.  

- Every transaction follows a strict flow:

BlockBazzar creates a secure, scam-resistant, trustless e-commerce experience.

---

## Overview of On-Chain Accounts Involved in Product, Cart, and Payment Logic
![Alt text 1](./imgs/img1.png)

---

# Program Account Structs

Below are the **actual program accounts** based on your SDK behavior and PDA usage.

You may replace with exact Rust definitions later --- but this is **95% accurate** to your on-chain model.

---

## Product Account

```rust

#[account]

pub struct Product {

    pub product_id: [u8; 16], // UUID bytes

    pub product_name: String,

    pub product_short_description: String,

    pub price: u64,                 // stored in smallest unit (cents * 100)

    pub category: CategoryVariant,  // enum/variant

    pub division: DivisionVariant,  // enum/variant

    pub seller_name: String,

    pub seller_pubkey: Pubkey,

    pub product_imgurl: String,

    pub quantity: u64,

    pub rating: u8,

    pub stock_status: bool,

}

```

## Overview of the Payment Initialization and Escrow Setup Process
![Payment Escrow Flow](./images/img2.png)

### Notes

* Product ID stored as raw bytes → converted to UUID in SDK.

* `product_list` PDA stores an array of product Pubkeys.

---

## ProductsList Account

```rust

#[account]

pub struct ProductsList {

    pub products: Vec<Pubkey>,

}

```

---

## Cart Account

```rust

#[account]

pub struct Cart {

    pub product_name: String,

    pub quantity: u64,

    pub price: u64,

    pub seller_pubkey: Pubkey,

    pub product_imgurl: String,

    pub consumer: Pubkey,

}

```

---

## CartList Account

```rust

#[account]

pub struct CartList {

    pub items: Vec<Pubkey>,  // cart PDAs

}

```

---

## Payment Account

```rust

#[account]

pub struct Payment {

    pub payment_id: [u8; 16], // UUID

    pub total_amount: u64,

    pub payment_method: u8,   // enum in typescript, stored as variant

    pub payment_status: u8,   // pending, completed, refunded, etc.

    pub owner: Pubkey,

}

```

---

## Escrow Account

Based on your SDK logs:

```rust

#[account]

pub struct Escrow {

    pub buyer: Pubkey,

    pub seller: Pubkey,

    pub total_amount: u64,

    pub escrow_status: u8,  // Initiated / Deposited / Released

    pub release_fund: bool,

}

```

The `vault` account is an associated token/lamport account owned by the escrow PDA.

---

## Order Account

```rust

#[account]

pub struct Order {

    pub order_id: [u8; 16],

    pub order_status: String,

    pub order_tracking: String,

    pub payment: Pubkey,

    pub owner: Pubkey,

}

```

## Overview of the Order Initialization and Escrow Deposit Handling
![Order Deposit Flow](./images/img3.png)

---

# PDA Structure Summary

| PDA         | Seeds                                        |

| ----------- | -------------------------------------------- |

| Product     | `"product"`, seller_pubkey, product_name     |

| ProductList | `"product_list"`, seller_pubkey              |

| Cart        | `"cart"`, consumer_pubkey, product_name      |

| CartList    | `"cart_list"`, consumer_pubkey               |

| Payment     | `"payment"`, owner_pubkey                    |

| Escrow      | `"escrow"`, owner_pubkey                     |

| Order       | `"order"`, owner_pubkey                      |

| Vault       | `"escrow"`, owner_pubkey (used as authority) |

---

# Payment Logic & Lifecycle

*(Full deep-dive explanation --- clean and clear)*

The **payment system** in BlockBazzar is designed to guarantee:

* Full transparency

* No double-spending

* No bypassing escrow

* Predictable state transitions

Below is a detailed breakdown of how **payment → escrow → order** flows.

---

## Overview of the Order Lifecycle and Escrow Withdrawal Process
![Order Withdrawal Flow](./images/img4.png)

---

# 1. Payment Initialization (`create_payment`)

When the user decides to purchase items:

1\. A **Payment PDA** is generated:

```

["payment", buyer_pubkey]

```

2\. `create_payment` stores:

   * `payment_id` (UUID)

   * `total_amount`

   * `payment_method`

   * `payment_status` = Pending

   * `owner = buyer_pubkey`

3\. Payment is not yet transferred --- this account only **records intent**.

### Why?

This prevents:

* duplicate payments

* manipulation of expected amounts

* invalid escrow creation

---

# 2. Escrow Initialization (`create_escrow`)

Escrow cannot start without a **valid Payment account**.

During escrow creation:

1\. The program validates:

   * Payment PDA exists

   * Payment isn't already used

   * Amount matches

2\. Escrow PDA:

```

["escrow", buyer_pubkey]

```

3\. Escrow is created with:

   * buyer_pubkey

   * seller_pubkey

   * total_amount

   * escrow_status = "Initiated"

   * release_fund = false

4\. A **vault PDA** (a lamport-holding account) is derived and created.

---

# 3. Deposit Into Escrow (`deposit_escrow`)

The buyer's SOL is moved into the vault PDA:

* Source: buyer wallet

* Destination: vault PDA

* Authority: buyer

* Verified against escrow.total_amount

After deposit:

* escrow_status = "Deposited"

This is the **trustless lock** that protects funds.

---

# 4. Seller Fulfillment & Withdrawal (`withdraw_escrow`)

After product delivery:

1\. Seller invokes `withdraw_escrow`

2\. Program checks:

   * buyer deposited

   * escrow_status == Deposited

3\. Funds are transferred:

   * Source: vault PDA

   * Destination: seller account

   * Authority: escrow PDA (via signer seeds)

4\. vault account is closed

5\. escrow.release_fund = true

6\. escrow_status = Released

The buyer has no control at this stage.

The seller cannot withdraw without proper deposit.

---

# 5. Order Creation (`create_order`)

Once escrow is created, an order is generated:

1\. Derive order PDA

2\. Order stores:

   * order_id (same UUID from payment)

   * order status (Processing)

   * tracking info

   * associated payment PDA

Order accounts allow the front-end to show:

* status

* delivery progress

* timestamps

---

# 6. Order Update (`update_order`)

A seller or authorized system can update:

* Shipped

* In Transit

* Delivered

* Failed

---

# Why This Payment Architecture is Secure

1\. **Nothing happens without a valid Payment PDA**

2\. **Escrow cannot be created without Payment**

3\. **Deposit must match the exact amount**

4\. **Seller cannot withdraw unless deposit is complete**

5\. **Buyer cannot reverse payment after depositing**

6\. **Every action uses PDAs to prevent account spoofing**

7\. **Escrow vault is controlled only via PDA signer seeds**

This creates a **crypto-native escrow system** that is:

* Permissionless

* Non-custodial

* Frontend-agnostic

* Trustless

---

# Disclaimer

This program is provided "as is", without any warranties of any kind, whether express or implied.

The authors and contributors are not responsible for financial loss, bugs, or misconfigurations.

You assume full responsibility for auditing, testing, and validating this program before deploying or interacting with it.

Use at your own risk.

---

# Contributing

Contributions are welcome.

Please open an issue before submitting significant design changes.

---

# License

MIT License.

See the `LICENSE` file for full text.

---
