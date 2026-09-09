use anchor_lang::prelude::*;

#[event]
pub struct VaultInitialized {
    pub user: Pubkey,
    pub max_amount: Option<u64>,
}

#[event]
pub struct Deposited {
    pub user: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
}

#[event]
pub struct Withdrawn {
    pub user: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
}

#[event]
pub struct VaultClosed {
    pub user: Pubkey,
    pub returned: u64,
}