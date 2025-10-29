#![allow(unexpected_cfgs,deprecated)]
use anchor_lang::prelude::*;
mod instructions;
mod states;
mod error;
use crate::instructions::*;
use crate::states::{Category,Division};

declare_id!("FYo4gi69vTJZJMnNxj2mZz2Q9CbUu12rQDVtHNUFQ2o7");

#[program]
pub mod ecom_dapp {
    use super::*;

    pub fn create_product(
        ctx: Context<CreateProduct>,
        product_name:String,
        product_short_description:String,
        price:u32,
        category: Category,
        division:Division,
        seller_name:String,
        product_imgurl:String,    
    ) -> Result<()> {
        ctx.accounts.create_product(
            product_name, 
            product_short_description, 
            price, category, 
            division, 
            seller_name, 
            product_imgurl, 
            ctx.bumps.product,
        )?;
        ctx.accounts.product_list(
            ctx.bumps.product_list,
        )?;
        let product_key = ctx.accounts.product.key();
        ctx.accounts.product_list.products.push(product_key);
        Ok(())
    }

    pub fn add_to_cart(
        ctx: Context<AddToCart>,
        product_name: String,
        quantity: u32,
        seller_pubkey: Pubkey,
        product_imgurl: String,
        price: u32,
    ) -> Result<()> {
        ctx.accounts.add_to_cart(
            product_name,
            quantity as u64,
            seller_pubkey,
            product_imgurl,
            price as u64,
            ctx.bumps.cart,
        )?;

        ctx.accounts.cart_list(
            ctx.bumps.cart_list,
        )?;
        let cart_key = ctx.accounts.cart.key();
        ctx.accounts.cart_list.cart_list.push(cart_key);
        Ok(())
    }

    pub fn create_payment(
        ctx: Context<CreatePayment>,
        payment_amount: u64,
        product_pubkey:Pubkey,
        tx_signature:Option<String>,
    ) -> Result<()> {
       ctx.accounts.create_payment(
        payment_amount, 
        product_pubkey, 
        tx_signature, 
        ctx.bumps.payments,
    )?;
        Ok(())
    }
    pub fn close_payment(
        ctx: Context<CloseAccount>,
    ) -> Result<()> {
        ctx.accounts.close_payment()
    }
    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        buyer_pubkey:Pubkey,
        seller_pubkey:Pubkey,
        amount:u64,
    )-> Result<()> {
        ctx.accounts.create_escrow(
            buyer_pubkey, 
            seller_pubkey, 
            amount, 
            ctx.bumps.escrow
        )?;
        Ok(())
    }

    pub fn deposit_escrow(
        ctx: Context<DepositeEscrow>,
        _product_id: u32,
    ) -> Result<()> {
        ctx.accounts.deposite_escrow(
            ctx.bumps.escrow,
        )?;
        Ok(())
    }

    pub fn withdraw_escrow(
        ctx: Context<WithdrawlEscrow>,
        _product_id: u32,
    ) -> Result<()> {
        ctx.accounts.withdrawl_escrow(
            ctx.bumps.escrow,
        )?;
        Ok(())
    }
    pub fn create_order(
        ctx: Context<CreateOrder>,
        payment_id:String,
    )->Result<()> {
        ctx.accounts.create_order(
            payment_id,
            ctx.bumps.order
        )?;
        Ok(())
    }

    pub fn update_order(
        ctx: Context<UpdateOrder>,
        orderstatus_str:String
    )->Result<()> {
        ctx.accounts.update_order(
            orderstatus_str,
        )?;
        Ok(())
    }


}
