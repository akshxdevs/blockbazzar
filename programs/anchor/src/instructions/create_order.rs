use anchor_lang::prelude::*;
use crate::{error::EcomError, states::{order::{Order, OrderStatus, OrderTracking}, payment::{Payment, PaymentStatus}}};
use anchor_lang::solana_program::hash::{self};
#[derive(Accounts)]
pub struct CreateOrder<'info>{
    #[account(mut)]
    pub signer:Signer<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [b"order",signer.key().as_ref()],
        bump,
        space = 9 + Order::INIT_SPACE,
    )]
    pub order:Account<'info,Order>,
    
    #[account(
        mut,
        seeds = [b"payment", signer.key().as_ref()],
        bump,
    )]
    pub payment:Account<'info,Payment>,
    
    pub system_program:Program<'info,System>,
}

#[derive(Accounts)]
pub struct UpdateOrder<'info>{
    #[account(mut)]
    pub signer:Signer<'info>,

    #[account(
        mut,
        seeds = [b"order",signer.key().as_ref()],
        bump,
    )]
    pub order:Account<'info,Order>,
}

#[derive(Accounts)]
pub struct CloseOrder<'info>{
    #[account(mut)]
    pub signer:Signer<'info>,
    #[account(
        mut,
        close = signer,
        seeds = [b"order",signer.key().as_ref()],
        bump 
    )]
    pub order:Account<'info,Order>,
    pub system_program: Program<'info,System>
}

impl<'info> CreateOrder<'info> {
    pub fn create_order(
        &mut self,
        payment_id:String,
        order_bump:u8,
    ) -> Result<()> {
        // Validate payment exists and status is valid
        require!(
            self.payment.payment_status == PaymentStatus::Success || 
            self.payment.payment_status == PaymentStatus::Pending,
            EcomError::InvalidPayment
        );

        // Convert payment_id from [u8;16] to UUID format string for comparison
        // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        let payment_id_bytes = self.payment.payment_id;
        let hex: String = payment_id_bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        
        let payment_id_uuid = format!(
            "{}-{}-{}-{}-{}",
            &hex[0..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..32]
        );

        // Validate that the provided payment_id matches the payment account
        require!(
            payment_id == payment_id_uuid,
            EcomError::UnexpectedError
        );

        let clock = Clock::get()?;

        let seed_data = [
            self.signer.key().as_ref(),
            &clock.unix_timestamp.to_le_bytes(),
        ].concat();
        
        let hash = hash::hash(&seed_data);
        
        let order_id = hash.to_bytes()[..16]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        
        let tracking_id = hash.to_bytes()[..16]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
        
        self.order.set_inner(Order { 
            order_id: order_id,
            payment_id, 
            tracking_id, 
            order_status:OrderStatus::Placed, 
            order_tracking:OrderTracking::Booked, 
            created_at:clock.unix_timestamp, 
            updated_at:clock.unix_timestamp, 
            order_bump,
        });
       Ok(()) 
    }
}

impl<'info> UpdateOrder<'info> {
    pub fn update_tracking_status(&mut self, status_str: String) -> Result<()> {
        let clock = Clock::get()?;
        match status_str.as_str() {
            "intransit" => self.order.order_tracking = OrderTracking::InTransit,
            "shipped" => self.order.order_tracking = OrderTracking::Shipped,
            "outfordelivery" => self.order.order_tracking = OrderTracking::OutForDelivery,
            "delivered" => self.order.order_tracking = OrderTracking::Delivered,
            _ => return err!(EcomError::UnexpectedError), 
        }
        self.order.updated_at = clock.unix_timestamp;
        Ok(())
    }
}

impl <'info>CloseOrder<'info> {
    pub fn close_order(&mut self)->Result<()>{
        msg!("Order Closed Successfullt, {}",self.order.key());
        Ok(())
    }
}
