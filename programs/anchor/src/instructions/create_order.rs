use anchor_lang::prelude::*;
use crate::{error::EcomError, states::order::{Order, OrderStatus, OrderTracking}};
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

impl<'info> CreateOrder<'info> {
    pub fn create_order(
        &mut self,
        payment_id:String,
        order_bump:u8,
    ) -> Result<()> {
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
    pub fn update_order(&mut self, status_str: String) -> Result<()> {
        match status_str.as_str() {
            "intransit" => self.order.order_tracking = OrderTracking::InTransit,
            "shipped" => self.order.order_tracking = OrderTracking::Shipped,
            "outfordelivery" => self.order.order_tracking = OrderTracking::OutForDelivery,
            "delivered" => self.order.order_tracking = OrderTracking::Delivered,
            _ => return err!(EcomError::UnexpectedError), 
        }
        Ok(())
    }
}
