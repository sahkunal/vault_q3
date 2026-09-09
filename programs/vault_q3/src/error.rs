use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The vault does not have enough lamports to fulfill this withdrawal.")]
    InsufficientFunds,

    #[msg("The deposit would push the vault balance over its configured maximum.")]
    DepositExceedsMax,

    #[msg("Amount must be greater than zero.")]
    InvalidAmount,

    #[msg("Arithmetic overflow.")]
    MathOverflow,
}