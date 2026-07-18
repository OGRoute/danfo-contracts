# Contributing to danfo-contracts

Thanks for helping build community-owned transit data for Lagos.

## Setup

```bash
rustup target add wasm32-unknown-unknown
cargo build -p danfo-registry --release --target wasm32-unknown-unknown  # first, always
cargo test --workspace
```

The registry wasm build must run before anything that compiles
`danfo-rewards` (it `contractimport!`s the registry interface).

## Rules

- `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`
  must pass — CI enforces both.
- No `unwrap()`/`expect()` in contract code (tests are fine). No floats;
  all amounts are `i128` base units.
- Every public function keeps its `///` doc: what it does, auth, errors.
- Every state change emits an event; every persistent write extends TTL.
- Add tests for every new error path (`#[should_panic(expected = ...)]`).

## Workflow

1. Pick an issue (or open one describing the change first).
2. Branch from `main`, one logical change per commit.
3. Conventional commits: `type(scope): description` — scopes: `registry`,
   `rewards`, `ci`, `deploy`, `docs`.
4. Open a PR; CI must be green.

## What not to do

- Don't add speculative functions with no user-flow mapping.
- Don't change storage types of deployed contracts without a migration plan.
- Don't commit secrets, `.stellar/`, or `target/`.
