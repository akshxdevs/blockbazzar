use anchor_lang::{prelude::*, system_program::{self, Transfer, transfer}};
use crate::{error::EcomError, states::{escrow::{Escrow, EscrowStatus}, payment::{Payment, PaymentMethod, PaymentStatus}, vault::VaultState}};
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

    #[account(
        init, 
        payer = owner, 
        space = VaultState::INIT_SPACE, 
        seeds = [b"state", owner.key().as_ref()], 
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        seeds = [b"vault", vault_state.key().as_ref()], 
        bump, 
    )]
    pub vault: SystemAccount<'info>, //
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
    
    #[account(
        seeds = [b"state", owner.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    
    ///CHECK: Native SOL Escrow Accounts
    #[account(mut)]
    pub escrow_account: AccountInfo<'info>,

    ///CHECK: Native SOL User Accounts
    #[account(mut)]
    pub user: AccountInfo<'info>,
    pub system_program:Program<'info,System>,
}

#[derive(Accounts)]
pub struct WithdrawlEscrow<'info>{
    #[account(mut)]
    pub owner: Signer<'info>,

    ///CHECK: Native SOL Seller Account
    #[account(mut)]
    pub seller_account: AccountInfo<'info>,

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
    
    #[account(
        seeds = [b"state", owner.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program:Program<'info,System>,
}

#[derive(Accounts)]
pub struct CloseAll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // payment (close → refund rent)
    #[account(
        mut,
        close = signer,
        seeds = [b"payment", signer.key().as_ref()],
        bump,
    )]
    pub payment: Account<'info, Payment>,

    // escrow (close → refund rent)
    #[account(
        mut,
        close = signer,
        seeds = [b"escrow", signer.key().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    // vault_state (close → refund rent)
    #[account(
        mut,
        close = signer,
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    // vault PDA – drain all lamports then let the system close it
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
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

impl <'info> CreateEscrow<'info> {
    pub fn create_escrow(
        &mut self,
        buyer_pubkey:Pubkey,
        seller_pubkey:Pubkey,
        amount:u64,
        escrow_bump:u8,
        vault:u8,
        vault_state:u8
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
            escrow_bump
        });
        self.vault_state.state_bump = vault_state;
        self.vault_state.vault_bump = vault;
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
        let cpi_programs = self.system_program.to_account_info();
        let cpi_accounts = system_program::Transfer {
            from:self.user.to_account_info(),
            to:self.vault.to_account_info(),
        };
        let cpi_ctx: CpiContext<'_, '_, '_, '_, system_program::Transfer<'_>> = CpiContext::new(cpi_programs,cpi_accounts);
        system_program::transfer(cpi_ctx, amount)?;

        escrow.escrow_status = EscrowStatus::FundsReceived;
        escrow.release_fund = true;
        Ok(())
    }
}

impl <'info> WithdrawlEscrow<'info> {
    pub fn withdrawl_escrow(&mut self) -> Result<()> {
        let amount = self.payment.payment_amount;
        require!(
            self.payment.payment_status == PaymentStatus::Pending
                && self.payment.payment_method == PaymentMethod::SOL,
            EcomError::InvalidPayment
        );
        require!(self.escrow.release_fund, EcomError::FundsNotFound);

        let state_value_key = self.vault_state.key();
        let seeds: &[&[u8]] = &[
            b"vault",
            state_value_key.as_ref(),
            &[self.vault_state.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.seller_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        transfer(cpi_ctx, amount)?;

        self.payment.payment_status = PaymentStatus::Success;
        self.escrow.escrow_status = EscrowStatus::SwapSuccess;
        self.escrow.release_fund = false;

        Ok(())
    }
}

impl<'info> CloseAll<'info> {
    pub fn close_all(&mut self) -> Result<()> {
        // 1. Drain vault → signer
        let lamports = self.vault.lamports();
        if lamports > 0 {
            let vault_state_key = self.vault_state.key();
            let seeds = &[
                b"vault",
                vault_state_key.as_ref(),
                &[self.vault_state.vault_bump],
            ];
            let signer_seeds = &[&seeds[..]];

            let cpi_accounts = Transfer {
                from: self.vault.to_account_info(),
                to: self.signer.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            let _ =transfer(cpi_ctx, lamports)?;
        }

        msg!(
            "Closed payment={}, escrow={}, vault_state={}, vault drained {} lamports",
            self.payment.key(),
            self.escrow.key(),
            self.vault_state.key(),
            lamports
        );
        Ok(())
    }
}