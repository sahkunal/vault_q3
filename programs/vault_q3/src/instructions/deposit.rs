use crate::{error::ErrorCode, events::Deposited, state::VaultState, STATE_SEED, VAULT_SEED};
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [VAULT_SEED, vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        seeds = [STATE_SEED, user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);

        let new_balance = self
            .vault
            .lamports()
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;

        if let Some(max_amount) = self.vault_state.max_amount {
            require!(new_balance <= max_amount, ErrorCode::DepositExceedsMax);
        }

        let cpi_context = CpiContext::new(
            self.system_program.to_account_info(),
            Transfer {
                from: self.user.to_account_info(),
                to: self.vault.to_account_info(),
            },
        );
        transfer(cpi_context, amount)?;

        emit!(Deposited {
            user: self.user.key(),
            amount,
            new_balance,
        });

        Ok(())
    }
}