pub mod constants;
pub mod error;
pub mod events;
pub mod helpers;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use events::*;
pub use instructions::*;
pub use state::*;
declare_id!("AJK2PUW79eWwvruMonMqDMv8LJcWt4es7PauvZpSpruV");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, max_amount: Option<u64>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps, max_amount)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}