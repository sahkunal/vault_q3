# vault_q3

A simple per-user SOL vault built with [Anchor](https://www.anchor-lang.com/) on Solana. Each user gets their own PDA-derived vault they can deposit into, withdraw from, and close — optionally capped by a maximum balance set at creation.

**Program ID:** `AJK2PUW79eWwvruMonMqDMv8LJcWt4es7PauvZpSpruV`

## How it works

| Instruction | What happens |
|---|---|
| `initialize(max_amount: Option<u64>)` | Creates a `VaultState` account for the caller and derives their `vault` PDA. `max_amount` optionally caps how many lamports the vault can hold; `0` is rejected. |
| `deposit(amount: u64)` | Transfers `amount` lamports from the user into their vault. Rejects `0`, and rejects deposits that would push the balance over `max_amount` (if set). |
| `withdraw(amount: u64)` | Transfers `amount` lamports from the vault back to the user. Rejects `0` and rejects withdrawing more than the current vault balance. |
| `close` | Drains the entire vault balance back to the user and closes the `VaultState` account, refunding its rent. |

### Account structure

```
VaultState (PDA, seeds = ["state", user])
├── vault_bump: u8
├── state_bump: u8
└── max_amount: Option<u64>

Vault (PDA, seeds = ["vault", vault_state])
— a plain SystemAccount holding lamports; never explicitly `init`'d,
  it comes into existence the first time lamports are sent to it.
```

Because both PDAs are derived from the calling `user`'s pubkey, each user can only ever reach their own vault — there's no shared/cross-user access to guard against.

## Prerequisites

- Rust + Cargo
- [Solana CLI](https://docs.solanalabs.com/cli/install) (Agave)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) `0.31.1`
- Node/Yarn (for TS-based deployment scripts, if used)

## A note on the build

Solana's bundled `cargo` (via `cargo-build-sbf` / platform-tools) currently lags behind crates.io on Rust edition support. Several transitive dependencies (`blake3`, `borsh`, `proc-macro-crate`, `indexmap`, `zeroize`, `unicode-segmentation`) have published newer releases that require Rust edition2024 / a newer `rustc` than platform-tools bundles. This repo's `Cargo.lock` pins all of those to compatible versions — **do not run a bare `cargo update`** on this project, or you'll reintroduce the conflict. If you ever need to bump a dependency deliberately, use:

```bash
cargo update -p <crate> --precise <version>
```

## Building

```bash
anchor build
```

## Testing

Tests are written with [litesvm](https://github.com/LiteSVM/litesvm) (an in-process, fast Solana VM) rather than `solana-test-validator`, so no local validator is required.

```bash
anchor build          # must run first — tests load the compiled .so
cargo test -p vault_q3 --test vault_q3
```

Coverage:
- `initialize` creates vault state, rejects `max_amount = 0`
- `deposit` increases the balance, rejects `0`, rejects exceeding `max_amount`
- `withdraw` returns lamports, rejects `0`, rejects exceeding the current balance
- `close` drains the vault and closes the state account, including the case where nothing was ever deposited

## Deploying

```bash
anchor deploy
```

Update the cluster/wallet in `Anchor.toml` as needed before deploying to devnet/mainnet.

## Tech stack

- Anchor `0.31.1`
- `litesvm` for testing
