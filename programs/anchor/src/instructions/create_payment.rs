use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::Token;
use crate::{error::EcomError, states::{escrow::{Escrow, EscrowStatus}, payment::{Payment, PaymentMethod, PaymentStatus}}};
use anchor_lang::solana_program::hash::{self};


#[derive(Accounts)]
pub struct CreatePayment<'info>{
    #[account(mut)]
    pub signer:Signer<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [b"payment",signer.key().as_ref()],
        bump,
        space = 8 + Payment::INIT_SPACE
    )]
    pub payments:Account<'info,Payment>,
    pub system_program:Program<'info,System>
}


#[derive(Accounts)]
pub struct CloseAccount<'info>{
    #[account(mut)]
    pub signer:Signer<'info>,

    #[account(
        mut,
        close = signer,  
        seeds = [b"payment",signer.key().as_ref()],
        bump,
    )]
    pub payments:Account<'info,Payment>,
}

#[derive(Accounts)]
pub struct CloseEscrowAccount<'info>{
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        close = signer,
        seeds = [b"escrow",signer.key().as_ref()],
        bump,
    )]
    pub escrow:Account<'info,Escrow>,
}

#[derive(Accounts)]
pub struct CreateEscrow<'info>{
    #[account(mut)]
    pub owner:Signer<'info>,
    #[account(
        init,
        payer = owner,
        seeds = [b"escrow",owner.key().as_ref()],
        bump,
        space = 8 + Escrow::INIT_SPACE,
    )]
    pub escrow: Account<'info,Escrow>,

    #[account(
        mut,
        seeds = [b"payment",owner.key().as_ref()],
        bump,
    )]
    pub payment:Account<'info,Payment>,

    ///CHECK: User Token Account
    #[account(mut)]
    pub user_ata: AccountInfo<'info>,
    ///CHECK: Escrow Token Account
    #[account(mut)]
    pub escrow_ata: AccountInfo<'info>,
    ///CHECK: Buyer Token Account
    #[account(mut)]
    pub buyer_ata: AccountInfo<'info>,
    ///CHECK: Seller Token Account
    #[account(mut)]
    pub seller_ata: AccountInfo<'info>,


    pub token_program:Program<'info,Token>,
    pub system_program:Program<'info,System>
}


#[derive(Accounts)]
pub struct DepositeEscrow<'info>{
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"escrow",owner.key().as_ref()],
        bump,
    )]
    pub escrow: Account<'info,Escrow>,
    #[account(
        mut,
        seeds = [b"payment",owner.key().as_ref()],
        bump,
    )]
    pub payment:Account<'info,Payment>,
    ///CHECK: Native SOL Vault Account
    #[account(
        mut,
        seeds = [b"vault",owner.key().as_ref()],
        bump,
    )]    
    pub vault_account: AccountInfo<'info>,
    
    ///CHECK: Native SOL Escrow Accounts
    #[account(mut)]
    pub escrow_account: AccountInfo<'info>,

    ///CHECK: Native SOL User Accounts
    #[account(mut)]
    pub user_account: AccountInfo<'info>,
    pub system_program:Program<'info,System>,
}

#[derive(Accounts)]
pub struct WithdrawlEscrow<'info>{
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"escrow",owner.key().as_ref()],
        bump,
    )]
    pub escrow: Account<'info,Escrow>,
    #[account(
        mut,
        seeds = [b"payment",owner.key().as_ref()],
        bump,
    )]
    pub payment:Account<'info,Payment>,
    ///CHECK: Native SOL Vault Account
    #[account(
        mut,
        seeds = [b"vault",owner.key().as_ref()],
        bump,
    )]    
    pub vault_account: AccountInfo<'info>,

    ///CHECK: Native SOL Escrow Account
    #[account(mut)]
    pub escrow_account: AccountInfo<'info>,

    ///CHECK: Native SOL Seller Account
    #[account(mut)]
    pub seller_account: AccountInfo<'info>,
    pub system_program:Program<'info,System>,
}



impl<'info> CreatePayment<'info>{
    pub fn create_payment(
        &mut self,
        payment_amount: u64,
        product_pubkey:Pubkey,
        tx_signature:Option<String>,
        payment_bump:u8,
    ) -> Result<()> {
        let clock = Clock::get()?;

        let seed_data = [
            self.signer.key().as_ref(),
            &clock.unix_timestamp.to_le_bytes(),
        ].concat();
        let hash = hash::hash(&seed_data);
        let payment_id:[u8;16] = hash.to_bytes()[..16]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        self.payments.set_inner(
            Payment { 
                payment_id: payment_id, 
                payment_amount, 
                product_pubkey, 
                payment_method: PaymentMethod::SOL, 
                payment_status: PaymentStatus::Pending, 
                time_stamp: clock.unix_timestamp, 
                tx_signature, 
                payment_bump, 
            }
        );
        Ok(())
    }
}

impl<'info> CloseAccount<'info> {
    pub fn close_payment(
        &mut self
    )->Result<()>{
        msg!("Payment account {} closed successfully", self.payments.key());
        Ok(())
    }
}

impl <'info> CreateEscrow<'info> {
    pub fn create_escrow(
        &mut self,
        buyer_pubkey:Pubkey,
        seller_pubkey:Pubkey,
        amount:u64,
        escrow_bump:u8,
    )->Result<()> {
        let clock = Clock::get()?;

        require!(
            self.payment.payment_status == PaymentStatus::Pending,
            EcomError::InvalidPayment
        );

        self.escrow.set_inner(Escrow { 
            owner: self.owner.key(), 
            buyer_pubkey, 
            seller_pubkey, 
            amount, 
            release_fund: false, 
            time_stamp: clock.unix_timestamp, 
            update_timestamp :clock.unix_timestamp, 
            escrow_status:EscrowStatus::SwapPending, 
            escrow_bump,
        });

        Ok(())
    }
}

impl <'info> CloseEscrowAccount<'info> {
    pub fn close_escrow(
        &mut self
    )->Result<()> {
        msg!("Escrow account {} closed successfully",self.escrow.key());
        Ok(())
    }
}

impl <'info> DepositeEscrow<'info> {
    pub fn deposite_escrow(
        &mut self,
        _escrow_bump:u8,
    )-> Result<()> {
        let payment = &mut self.payment;
        let escrow = &mut self.escrow;
        let amount = payment.payment_amount;
        
        require!(
            payment.payment_status == PaymentStatus::Pending,
            EcomError::EscrowError
        );

        let cpi_accounts = system_program::Transfer {
            from:self.user_account.to_account_info(),
            to:self.vault_account.to_account_info(),
        };
        let cpi_programs = self.system_program.to_account_info();
        let cpi_ctx = CpiContext::new(
            cpi_programs,
            cpi_accounts,
        );
        system_program::transfer(cpi_ctx, amount)?;

        escrow.escrow_status = EscrowStatus::FundsReceived;
        escrow.release_fund = true;
        Ok(())
    }
}

impl <'info> WithdrawlEscrow<'info> {
    pub fn withdrawl_escrow(
        &mut self,
        vault_bump:u8
    )-> Result<()> {
        let payment = &mut self.payment;
        let escrow = &mut self.escrow;
        let amount = payment.payment_amount;

        require!(
            payment.payment_status == PaymentStatus::Pending
            && payment.payment_method == PaymentMethod::SOL,
            EcomError::InvalidPayment
        );
        require!(escrow.release_fund == true,EcomError::FundsNotFound);
        
        let cpi_accounts = system_program::Transfer{
            from:self.vault_account.to_account_info(),
            to:self.seller_account.to_account_info(),
        };
        let cpi_programs: AccountInfo<'_> = self.system_program.to_account_info();
        let owner_key = self.owner.key();
        let seeds: &[&[u8]] = &[
            b"vault",
            owner_key.as_ref(),
            &[vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            cpi_programs,
            cpi_accounts,
            signer_seeds,
        );
        system_program::transfer(cpi_ctx, amount)?;

        payment.payment_status = PaymentStatus::Success;
        escrow.escrow_status = EscrowStatus::SwapSuccess;
        escrow.release_fund = false;
        Ok(())
    }
}