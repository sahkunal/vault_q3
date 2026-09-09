

use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_sdk::{
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
    transaction::Transaction,
};
use vault_q3::accounts as vault_accounts;
use vault_q3::instruction as vault_ix;
use vault_q3::{STATE_SEED, VAULT_SEED};

const PROGRAM_ID: Pubkey = vault_q3::ID;


fn account_closed(svm: &LiteSVM, pubkey: &Pubkey) -> bool {
    svm.get_account(pubkey)
        .map(|a| a.lamports == 0)
        .unwrap_or(true)
}

struct Setup {
    svm: LiteSVM,
    user: Keypair,
    vault_state: Pubkey,
    vault: Pubkey,
}


fn setup() -> Setup {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(PROGRAM_ID, "../../target/deploy/vault_q3.so")
        .expect("failed to load vault_q3.so — did you run `anchor build`?");

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let (vault_state, _) =
        Pubkey::find_program_address(&[STATE_SEED, user.pubkey().as_ref()], &PROGRAM_ID);
    let (vault, _) =
        Pubkey::find_program_address(&[VAULT_SEED, vault_state.as_ref()], &PROGRAM_ID);

    Setup {
        svm,
        user,
        vault_state,
        vault,
    }
}

fn send(svm: &mut LiteSVM, payer: &Keypair, ix: Instruction) -> Result<(), String> {
    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[payer], blockhash);
    svm.send_transaction(tx).map(|_| ()).map_err(|e| format!("{e:?}"))
}

fn initialize_vault(setup: &mut Setup, max_amount: Option<u64>) {
    let accounts = vault_accounts::Initialize {
        user: setup.user.pubkey(),
        vault_state: setup.vault_state,
        vault: setup.vault,
        system_program: system_program::ID,
    };

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: vault_ix::Initialize { max_amount }.data(),
    };

    send(&mut setup.svm, &setup.user, ix).expect("initialize failed");
}

fn deposit(setup: &mut Setup, amount: u64) -> Result<(), String> {
    let accounts = vault_accounts::Deposit {
        user: setup.user.pubkey(),
        vault: setup.vault,
        vault_state: setup.vault_state,
        system_program: system_program::ID,
    };

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: vault_ix::Deposit { amount }.data(),
    };

    send(&mut setup.svm, &setup.user, ix)
}

fn withdraw(setup: &mut Setup, amount: u64) -> Result<(), String> {
    let accounts = vault_accounts::Withdraw {
        user: setup.user.pubkey(),
        vault: setup.vault,
        vault_state: setup.vault_state,
        system_program: system_program::ID,
    };

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: vault_ix::Withdraw { amount }.data(),
    };

    send(&mut setup.svm, &setup.user, ix)
}

fn close_vault(setup: &mut Setup) -> Result<(), String> {
    let accounts = vault_accounts::Close {
        user: setup.user.pubkey(),
        vault: setup.vault,
        vault_state: setup.vault_state,
        system_program: system_program::ID,
    };

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: vault_ix::Close {}.data(),
    };

    send(&mut setup.svm, &setup.user, ix)
}

#[test]
fn initialize_creates_vault_state() {
    let mut setup = setup();
    initialize_vault(&mut setup, None);

    let account = setup.svm.get_account(&setup.vault_state);
    assert!(account.is_some(), "vault_state account should exist after initialize");
}

#[test]
fn initialize_rejects_zero_max_amount() {
    let mut setup = setup();

    let accounts = vault_accounts::Initialize {
        user: setup.user.pubkey(),
        vault_state: setup.vault_state,
        vault: setup.vault,
        system_program: system_program::ID,
    };

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: vault_ix::Initialize {
            max_amount: Some(0),
        }
        .data(),
    };

    let result = send(&mut setup.svm, &setup.user, ix);
    assert!(result.is_err(), "max_amount of 0 should be rejected (InvalidAmount)");
}

#[test]
fn deposit_increases_vault_balance() {
    let mut setup = setup();
    initialize_vault(&mut setup, None);

    let before = setup.svm.get_balance(&setup.vault).unwrap_or(0);
    deposit(&mut setup, LAMPORTS_PER_SOL).expect("deposit failed");
    let after = setup.svm.get_balance(&setup.vault).unwrap();

    assert_eq!(after, before + LAMPORTS_PER_SOL);
}

#[test]
fn deposit_rejects_zero_amount() {
    let mut setup = setup();
    initialize_vault(&mut setup, None);

    let result = deposit(&mut setup, 0);
    assert!(result.is_err(), "zero-amount deposit should be rejected (InvalidAmount)");
}

#[test]
fn deposit_rejects_amount_over_max() {
    let mut setup = setup();
    initialize_vault(&mut setup, Some(LAMPORTS_PER_SOL));

    // First deposit within the cap succeeds.
    deposit(&mut setup, LAMPORTS_PER_SOL / 2).expect("initial deposit should succeed");

    // Second deposit that would push balance over max_amount must fail.
    let result = deposit(&mut setup, LAMPORTS_PER_SOL);
    assert!(result.is_err(), "deposit exceeding max_amount should be rejected (DepositExceedsMax)");
}

#[test]
fn withdraw_returns_lamports_to_user() {
    let mut setup = setup();
    initialize_vault(&mut setup, None);
    deposit(&mut setup, LAMPORTS_PER_SOL).expect("deposit failed");

    let vault_before = setup.svm.get_balance(&setup.vault).unwrap();
    let user_before = setup.svm.get_balance(&setup.user.pubkey()).unwrap();

    withdraw(&mut setup, LAMPORTS_PER_SOL / 2).expect("withdraw failed");

    let vault_after = setup.svm.get_balance(&setup.vault).unwrap();
    let user_after = setup.svm.get_balance(&setup.user.pubkey()).unwrap();

    assert_eq!(vault_after, vault_before - LAMPORTS_PER_SOL / 2);
    // user_after is slightly less than a naive `user_before + amount` due to
    // the tx fee paid by the user for this withdraw transaction.
    assert!(user_after > user_before);
}

#[test]
fn withdraw_rejects_amount_over_balance() {
    let mut setup = setup();
    initialize_vault(&mut setup, None);
    deposit(&mut setup, LAMPORTS_PER_SOL / 10).expect("deposit failed");

    let result = withdraw(&mut setup, LAMPORTS_PER_SOL);
    assert!(result.is_err(), "withdrawing more than the vault balance should be rejected (InsufficientFunds)");
}

#[test]
fn withdraw_rejects_zero_amount() {
    let mut setup = setup();
    initialize_vault(&mut setup, None);
    deposit(&mut setup, LAMPORTS_PER_SOL).expect("deposit failed");

    let result = withdraw(&mut setup, 0);
    assert!(result.is_err(), "zero-amount withdraw should be rejected (InvalidAmount)");
}

#[test]
fn close_drains_vault_and_closes_state() {
    let mut setup = setup();
    initialize_vault(&mut setup, None);
    deposit(&mut setup, LAMPORTS_PER_SOL).expect("deposit failed");

    let user_before = setup.svm.get_balance(&setup.user.pubkey()).unwrap();

    close_vault(&mut setup).expect("close failed");

    let vault_after = setup.svm.get_balance(&setup.vault).unwrap_or(0);
    let vault_state_after = setup.svm.get_account(&setup.vault_state);
    let user_after = setup.svm.get_balance(&setup.user.pubkey()).unwrap();

    assert_eq!(vault_after, 0, "vault should be fully drained");
    assert!(
        account_closed(&setup.svm, &setup.vault_state),
        "vault_state should be closed and removed"
    );
    // Deposited SOL plus the vault_state rent comes back, minus tx fees.
    assert!(user_after > user_before);
}

#[test]
fn close_with_empty_vault_still_closes_state() {
    let mut setup = setup();
    initialize_vault(&mut setup, None);


    close_vault(&mut setup).expect("close on an empty vault should still succeed");

    assert!(account_closed(&setup.svm, &setup.vault_state));
}