# danfo-contracts

[![CI](https://github.com/OGRoute/danfo-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/OGRoute/danfo-contracts/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-GitBook-3884FF.svg)](docs/README.md)
[![Soroban SDK](https://img.shields.io/badge/Soroban%20SDK-22-black.svg)](https://developers.stellar.org/docs/build/smart-contracts/overview)

Soroban smart contracts for **Danfo** — a community-owned transit knowledge
protocol for Lagos on [Stellar](https://stellar.org).

📖 **[Documentation](docs/README.md)** · 🌊 **[Contributing via Drips Wave](docs/contributing/drips-wave.md)** · 🗺️ **[Roadmap](docs/contributing/roadmap.md)**

---

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

So spam funds the people doing the real work. That loop is the whole design —
[stake economics](docs/protocol/economics.md) does the arithmetic.

## Contracts

| Contract | Responsibility |
|---|---|
| [`contracts/registry`](contracts/registry) | Correction lifecycle: staked submit → attest → finalize (refund/slash), plus reads and per-address reputation — [API reference](docs/reference/registry.md) |
| [`contracts/rewards`](contracts/rewards) | Sponsor-funded pool paying a fixed reward per accepted correction; pull-based, idempotent claims — [API reference](docs/reference/rewards.md) |

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

34 tests — 23 in the registry, 11 in the rewards pool, including a
cross-contract integration test that drives one correction from submission to
payout. Full setup in the [quickstart](docs/getting-started/quickstart.md).

Known pin: `ed25519-dalek` is held at `2.x` in `Cargo.lock`;
`soroban-env-host 22.x` does not compile against dalek `3.0`.

## Deploy (testnet)

```bash
./scripts/deploy.sh
```

Deploys both contracts in dependency order, initializes them, points the
registry's `slash_recipient` at the rewards pool, and prints the contract ids
to paste into the application layer's environment. Tunables and verification
commands: [deploying to testnet](docs/getting-started/deploying.md).

Launch defaults: 10 XLM stake, 24 h challenge window, 2 minimum attestations,
5 XLM reward — all admin-tunable via `set_config` / `set_reward`, and all
readable on-chain via `get_config`.

## Contributing

Newcomers start with issues labelled
[`trivial`](https://github.com/OGRoute/danfo-contracts/labels/trivial); the
open design questions are labelled
[`needs-design`](https://github.com/OGRoute/danfo-contracts/labels/needs-design)
and want a discussion before code.

- [How to contribute](docs/contributing/how-to-contribute.md) — the loop, and what review looks for
- [Drips Wave](docs/contributing/drips-wave.md) — how issues here are scoped, sized, and rewarded
- [Roadmap](docs/contributing/roadmap.md) — what needs doing, with complexity
- [Code of Conduct](CODE_OF_CONDUCT.md)

## Related

The application layer — the multilingual AI transit agent, the corrections
feed, and the indexer — lives in a separate repository, `danfo-app`. This
repository is the on-chain half and stands alone: everything here builds,
tests, and deploys without it.

## Status

Unaudited testnet software. Do not use with mainnet funds. The
[threat model](docs/protocol/threat-model.md) sets out what these contracts
defend against, what they do not, and the open problems. Vulnerability
disclosure: [SECURITY.md](SECURITY.md).

## License

[MIT](LICENSE)
