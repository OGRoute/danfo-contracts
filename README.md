# danfo-contracts

Soroban smart contracts for **Danfo** — a community-owned transit knowledge
protocol for Lagos on [Stellar](https://stellar.org).

Lagos runs on informal *danfo* minibuses with no official map or fare data.
Danfo lets riders and drivers maintain that data themselves, with real
economic weight behind every edit:

1. **Stake** — submitting a correction locks a small stake
   (`danfo-registry.submit`).
2. **Attest** — peers approve or reject it during a challenge window
   (`attest`).
3. **Settle** — after the window, anyone can `finalize`: accepted corrections
   refund the stake and become part of the knowledge base; rejected spam is
   slashed **into the reward pool**.
4. **Earn** — accepted corrections claim a payout from the sponsor-funded
   `danfo-rewards` pool (`claim`), in XLM or NGNC (Stellar's naira
   stablecoin — redeemable to a Nigerian bank account via SEP-24).

The application layer (multilingual AI transit agent, corrections feed,
indexer) lives in [danfo-app](../danfo-app).

## Contracts

| Contract | Responsibility |
|---|---|
| [`contracts/registry`](contracts/registry) | Correction lifecycle: staked submit → attest → finalize (refund/slash), plus reads and per-address reputation |
| [`contracts/rewards`](contracts/rewards) | Sponsor-funded pool paying a fixed reward per accepted correction; pull-based, idempotent claims |

`danfo-rewards` verifies acceptance by a cross-contract read of
`danfo-registry.get(id)` — it never trusts the caller.

## Build & test

```bash
rustup target add wasm32-unknown-unknown

# registry wasm must build first: rewards imports its interface
cargo build -p danfo-registry --release --target wasm32-unknown-unknown
cargo test --workspace
cargo build -p danfo-rewards --release --target wasm32-unknown-unknown
```

Known pin: `ed25519-dalek` is held at `2.x` in `Cargo.lock`;
`soroban-env-host 22.x` does not compile against dalek `3.0`.

## Deploy (testnet)

```bash
./scripts/deploy.sh
```

Deploys both contracts in dependency order, initializes them, points the
registry's `slash_recipient` at the rewards pool, and prints the contract ids
to paste into danfo-app's environment.

Launch defaults: 10 XLM stake, 24 h challenge window, 2 minimum attestations,
5 XLM reward — all admin-tunable via `set_config` / `set_reward`.

## Status

Unaudited testnet software. Do not use with mainnet funds. See
[SECURITY.md](SECURITY.md).

## License

[MIT](LICENSE)
