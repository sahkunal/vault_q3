use crate::VAULT_SEED;
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

pub fn transfer_out_of_vault<'info>(
    system_program: &Program<'info, System>,
    vault: &SystemAccount<'info>,
    destination: AccountInfo<'info>,
    vault_state_key: Pubkey,
    vault_bump: u8,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }

    let bump = [vault_bump];
    let signer_seeds: &[&[&[u8]]] = &[&[VAULT_SEED, vault_state_key.as_ref(), &bump]];

    let cpi_context = CpiContext::new_with_signer(
        system_program.to_account_info(),
        Transfer {
            from: vault.to_account_info(),
            to: destination,
        },
        signer_seeds,
    );

    transfer(cpi_context, amount)
}