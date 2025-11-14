use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Cart{
    pub product_id: [u8;16],
    #[max_len(50)]
    pub product_name: String,
    pub quantity: u64,
    pub seller_pubkey: Pubkey,
    #[max_len(150)]
    pub product_imgurl: String,
    pub stock_status: Stock,
    #[max_len(40)]
    pub amount: Vec<u64>,
    pub cart_bump:u8,
}

#[account]
#[derive(InitSpace)]
pub struct CartList{
    #[max_len(40)]
    pub cart_list: Vec<Pubkey>,
    pub total_amount:u64, 
    pub cart_list_bump:u8,
}
#[event]
pub struct CartCreated {
    pub seller: Pubkey,
    pub quantity: u64,
    pub product_name: String,
    pub amount: u64,
}
#[derive(AnchorDeserialize,AnchorSerialize,Clone,InitSpace, PartialEq, Eq)]
pub enum Stock {
    OutOfStock,
    InStock,
    Restoring
}