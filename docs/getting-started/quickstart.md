# Quickstart

Clone, build, and run the full test suite in about five minutes.

## Prerequisites

| Tool | Why |
| --- | --- |
| Rust (stable) | `rust-toolchain.toml` pins the channel and the wasm target for you |
| `wasm32-unknown-unknown` target | Soroban contracts compile to wasm |
| [Stellar CLI](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup) | Only needed to deploy — not to build or test |

```bash
rustup target add wasm32-unknown-unknown
```

## Build and test

{% hint style="danger" %}
**Build order matters.** `danfo-rewards` calls `contractimport!` on the
registry's compiled wasm, so the registry must be built *before* anything that
compiles the rewards crate. A bare `cargo test --workspace` on a clean
checkout will fail with a missing-file error until you have run the first
command below once.
{% endhint %}

```bash
git clone https://github.com/OGRoute/danfo-contracts
cd danfo-contracts

# 1. registry wasm first — rewards imports its interface
cargo build -p danfo-registry --release --target wasm32-unknown-unknown

# 2. now the workspace tests compile
cargo test --workspace

# 3. and the rewards wasm
cargo build -p danfo-rewards --release --target wasm32-unknown-unknown
```

You should see 34 passing tests — 23 in the registry, 11 in the rewards pool.

## Try the lifecycle in a test

The fastest way to see the protocol work end to end is the integration test
that drives a correction from submission to payout. It lives in
[`contracts/rewards/src/test.rs`](https://github.com/OGRoute/danfo-contracts/blob/main/contracts/rewards/src/test.rs):

```bash
cargo test -p danfo-rewards claim_pays_contributor_once -- --nocapture
```

That test mints tokens, submits a correction, collects two approving
attestations, advances the ledger clock past the challenge window, finalizes,
and claims the reward — the entire flow described in
[Correction lifecycle](../protocol/lifecycle.md).

## Deploy it

Ready to put it on testnet? → [Deploying to testnet](deploying.md)
