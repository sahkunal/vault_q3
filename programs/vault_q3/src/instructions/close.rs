use crate::{
    events::VaultClosed, helpers::transfer_out_of_vault, state::VaultState, STATE_SEED, VAULT_SEED,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [VAULT_SEED, vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [STATE_SEED, user.key().as_ref()],
        bump = vault_state.state_bump,
        close = user,
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {
        let balance = self.vault.lamports();

        transfer_out_of_vault(
            &self.system_program,
            &self.vault,
            self.user.to_account_info(),
            self.vault_state.key(),
            self.vault_state.vault_bump,
            balance,
        )?;

        emit!(VaultClosed {
            user: self.user.key(),
            returned: balance,
        });

        Ok(())
    }
}