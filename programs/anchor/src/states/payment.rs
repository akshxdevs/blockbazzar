use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Payment{
    pub payment_id: [u8;16],
    pub payment_amount: u64,
    pub product_pubkey:Pubkey,
    pub payment_method:PaymentMethod,
    pub payment_status:PaymentStatus,
    pub time_stamp:i64,
    #[max_len(3)]
    pub tx_signature:Option<String>,
    pub payment_bump:u8,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, InitSpace, PartialEq, Eq)]
pub enum PaymentMethod {
    SOL,
    ETH,
    BTC,
    USDT,
    USDC
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, InitSpace, PartialEq, Eq)]
pub enum PaymentStatus {
    Success,
    Pending,
    Failed,
}