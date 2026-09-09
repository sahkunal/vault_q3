use crate::{
    error::ErrorCode, events::Withdrawn, helpers::transfer_out_of_vault, state::VaultState,
    STATE_SEED, VAULT_SEED,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);

        let balance = self.vault.lamports();
        require!(balance >= amount, ErrorCode::InsufficientFunds);

        transfer_out_of_vault(
            &self.system_program,
            &self.vault,
            self.user.to_account_info(),
            self.vault_state.key(),
            self.vault_state.vault_bump,
            amount,
        )?;

        emit!(Withdrawn {
            user: self.user.key(),
            amount,
            new_balance: balance - amount,
        });

        Ok(())
    }
}