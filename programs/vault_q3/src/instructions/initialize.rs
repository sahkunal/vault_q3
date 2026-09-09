use crate::{error::ErrorCode, events::VaultInitialized, state::VaultState, STATE_SEED, VAULT_SEED};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [STATE_SEED, user.key().as_ref()],
        bump,
        space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        seeds = [VAULT_SEED, vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps, max_amount: Option<u64>) -> Result<()> {
        if let Some(max) = max_amount {
            require!(max > 0, ErrorCode::InvalidAmount);
        }

        self.vault_state.set_inner(VaultState {
            vault_bump: bumps.vault,
            state_bump: bumps.vault_state,
            max_amount,
        });

        emit!(VaultInitialized {
            user: self.user.key(),
            max_amount,
        });

        Ok(())
    }
}