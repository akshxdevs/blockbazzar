use anchor_lang::prelude::*;
use crate::states::{cart::{Cart, CartCreated, CartList, Stock}, Product};
#[derive(Accounts)]
#[instruction(product_name: String)]
pub struct AddToCart<'info> {
    #[account(mut)]
    pub consumer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = consumer,
        seeds = [
            b"cart", consumer.key().as_ref(), 
            product_name.as_bytes()
        ],
        bump,
        space = 8 + Cart::INIT_SPACE
    )]
    pub cart: Account<'info, Cart>,

    #[account(mut)]
    pub products:Account<'info,Product>,
    #[account(
        init_if_needed,
        payer = consumer,
        seeds= [b"cart_list",consumer.key().as_ref()],
        bump,
        space = 8 + CartList::INIT_SPACE
    )]
    pub cart_list:Account<'info,CartList>,
    pub system_program: Program<'info, System>,
}

impl <'info> AddToCart <'info> {
    pub fn add_to_cart(
        &mut self,
        product_name: String,
        quantity: u64,
        seller_pubkey: Pubkey,
        product_imgurl: String,
        amount: u64,
        cart_bump:u8,
    ) -> Result<()>{
        let product_id = self.products.product_id;
        self.cart.set_inner(Cart 
            { 
                product_id, 
                product_name: product_name.clone(),
                quantity, 
                seller_pubkey, 
                product_imgurl: product_imgurl.clone(),
                amount: vec![amount], 
                stock_status: Stock::InStock,
                cart_bump,
            });

        emit!(CartCreated{
            product_name,
            amount,
            quantity,
            seller: seller_pubkey,
        });
        Ok(())
    }
    
    pub fn cart_list(
        &mut self,
        cart_list_bump:u8,
    )->Result<()>{
        let cart = &mut self.cart;
        let current_cart_total: u64 = cart.amount.iter().copied().sum();
        let quantity = cart.quantity;
        if self.cart_list.cart_list.is_empty() {
            self.cart_list.set_inner(CartList { 
                cart_list: Vec::new(),
                total_amount: current_cart_total * quantity,  
                cart_list_bump 
            });
        } else {
            let new_total = self.cart_list.total_amount.saturating_add(current_cart_total);
            self.cart_list.total_amount = new_total * quantity;
        }
        Ok(())
    }
}
