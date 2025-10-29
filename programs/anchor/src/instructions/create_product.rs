use anchor_lang::prelude::*;
use crate::states::{Category, Division, Product, ProductCreated, ProductsList, Stock};
use anchor_lang::solana_program::hash::{self};
use anchor_lang::solana_program::program_error::ProgramError;


#[derive(Accounts)]
#[instruction(product_name: String)]
pub struct CreateProduct<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        init,
        payer = seller,
        seeds = [b"product", seller.key().as_ref(), product_name.as_bytes()],
        bump,
        space = 8 + Product::INIT_SPACE
    )]
    pub product: Account<'info, Product>,

    #[account(
        init_if_needed,
        payer = seller,
        seeds = [b"product_list", seller.key().as_ref()],
        bump,
        space = 8 + ProductsList::INIT_SPACE
    )]
    pub product_list: Account<'info, ProductsList>,
    pub system_program: Program<'info, System>,
}

impl <'info> CreateProduct<'info> {
    pub fn create_product(
        &mut self,
        product_name:String,
        product_short_description:String,
        price:u32,
        category: Category,
        division:Division,
        seller_name:String,
        product_imgurl:String,
        creation_bump:u8,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let seed_data = [
            self.seller.key().as_ref(),
            &now.to_le_bytes(),
            product_name.as_bytes(),
        ].concat();
        let hash = hash::hash(&seed_data);
        let product_id: [u8; 16]  = hash.to_bytes()[..16]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        msg!("Generated Product ID: {:?}", product_id);

        self.product.set_inner(Product { 
            product_id, 
            product_name:product_name.clone(), 
            category:category.clone() , 
            division:division.clone(), 
            quantity: 100, 
            seller_pubkey:self.seller.key().clone(), 
            seller_name, 
            product_short_description, 
            product_imgurl, 
            price, 
            rating: 0.0, 
            stock_status: Stock::InStock,
            creation_bump  
        });
        emit!(ProductCreated{
            product_pubkey:self.product.key(),
            seller:self.seller.key(),
            product_name,
            price,
            category,
            division
        });
        Ok(())
    }
    pub fn product_list(
        &mut self,
        product_list_bump: u8,
    ) -> Result<()> {
        if self.product_list.products.is_empty() && self.product_list.product_list_bump == 0 {
            self.product_list.set_inner(ProductsList { 
                products: Vec::new(), 
                product_list_bump 
            });
        }
        Ok(())
    }
}


