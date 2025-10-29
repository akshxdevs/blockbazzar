use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Product {
    pub product_id:  [u8; 16] ,
    #[max_len(50)]
    pub product_name: String,
    pub category: Category,
    pub division: Division,
    pub quantity: u32,
    pub seller_pubkey: Pubkey,
    #[max_len(50)]
    pub seller_name: String,
    #[max_len(300)]
    pub product_short_description: String,
    #[max_len(150)]
    pub product_imgurl: String,
    pub price: u32,
    pub rating: f32,
    pub stock_status: Stock,
    pub creation_bump:u8,
}

#[account]
#[derive(InitSpace)]
pub struct ProductsList{
    #[max_len(40)]
    pub products:Vec<Pubkey>,
    pub product_list_bump:u8,
}
#[event]
pub struct ProductCreated {
    pub product_pubkey: Pubkey,
    pub seller: Pubkey,
    pub product_name: String,
    pub price: u32,
    pub category: Category,
    pub division: Division,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum Category {
    Electronics,
    BeautyAndPersonalCare,
    SnacksAndDrinks,
    HouseholdEssentials,
    GroceryAndKitchen,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum Division {
    Mobile,
    Laptop,
    Headphone,
    SmartWatch,
    ComputerPeripherals,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum Stock {
    OutOfStock,
    InStock,
    Restoring
}

