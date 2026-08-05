# Contributing to danfo-contracts

Thanks for helping build community-owned transit data for Lagos.

This page is the short version. The full guide — what review actually looks
for, and why each rule exists — is
[docs/contributing/how-to-contribute.md](docs/contributing/how-to-contribute.md).

## Setup

```bash
rustup target add wasm32-unknown-unknown
cargo build -p danfo-registry --release --target wasm32-unknown-unknown  # first, always
cargo test --workspace
```

The registry wasm build must run before anything that compiles
`danfo-rewards` (it `contractimport!`s the registry interface). On a clean
checkout, `cargo test --workspace` fails until that first command has run
once.

## Pick something

- [Roadmap](docs/contributing/roadmap.md) — what needs doing, sized
- [`trivial`](https://github.com/OGRoute/danfo-contracts/labels/trivial) — no
  protocol judgement required; start here
- [`needs-design`](https://github.com/OGRoute/danfo-contracts/labels/needs-design)
  — bring a proposal to the issue before writing code

Comment on the issue and wait to be assigned. If you are here through
[Drips Wave](docs/contributing/drips-wave.md), assignment happens through
Drips and is required before you start.

## Rules

- `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`
  must pass — CI enforces both.
- No `unwrap()`/`expect()` in contract code (tests are fine). No floats;
  all amounts are `i128` base units.
- Every public function keeps its `///` doc: what it does, auth, errors.
- Every state change emits an event; every persistent write extends TTL.
- Add tests for every new error path (`#[should_panic(expected = ...)]`).
- Update the page under `docs/` that documents anything you changed.

## Workflow

1. Pick an issue (or open one describing the change first).
2. Branch from `main`, one logical change per commit.
3. Conventional commits: `type(scope): description` — scopes: `registry`,
   `rewards`, `ci`, `deploy`, `docs`.
4. Open a PR; CI must be green. The
   [PR template](.github/PULL_REQUEST_TEMPLATE.md) is the review checklist.

## What not to do

- Don't add speculative functions with no user-flow mapping.
- Don't change storage types of deployed contracts without a migration plan.
- Don't renumber `Error` discriminants — clients match on the integer.
- Don't commit secrets, `.stellar/`, or `target/`.
- Don't open a public issue for a security vulnerability — see
  [SECURITY.md](SECURITY.md).

Participation is governed by the [Code of Conduct](CODE_OF_CONDUCT.md).
