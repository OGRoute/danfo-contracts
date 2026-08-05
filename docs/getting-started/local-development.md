# Local development

## Layout

```
danfo-contracts/
├── contracts/
│   ├── registry/          danfo-registry — correction lifecycle
│   │   └── src/
│   │       ├── lib.rs     contract entry points
│   │       ├── types.rs   storage types, config, errors, constants
│   │       └── test.rs    23 tests
│   └── rewards/           danfo-rewards — sponsor-funded payout pool
│       └── src/
│           ├── lib.rs     contract entry points + registry interface import
│           └── test.rs    11 tests, incl. cross-contract integration
├── docs/                  this documentation (GitBook-synced)
├── scripts/deploy.sh      sequential testnet deploy + wiring
└── .github/workflows/     fmt, clippy, test, ordered wasm builds
```

## The checks CI runs

Run these before pushing; CI runs exactly the same three, in this order.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`clippy` is `-D warnings`, so a warning is a build failure. That is
deliberate — see [How to contribute](../contributing/how-to-contribute.md).

## House rules

These are enforced in review, and most of them are enforced by CI too.

* **No `unwrap()` or `expect()` in contract code.** Use `unwrap_or_else(||
  panic_with_error!(...))` with a typed error so the caller gets
  `Error(Contract, #N)` instead of an opaque trap. Tests may use `unwrap()`
  freely.
* **No floats.** Every amount is an `i128` in the token's base units
  (stroops for XLM).
* **Every public function carries a `///` doc** stating what it does, who must
  authorise it, and which errors it can raise.
* **Every state change publishes an event.** The indexer in `danfo-app` reads
  the chain through events, not through polling reads.
* **Every persistent write extends its TTL.** Soroban state expires; a write
  without a bump is a bug that only shows up 30 days later.
* **Every error path gets a test** with
  `#[should_panic(expected = "Error(Contract, #N)")]`.

## Adding a stored field

Changing a `#[contracttype]` that is already deployed changes the on-chain
encoding of every existing entry. Deployed contracts hold real testnet stakes,
so:

1. Say so explicitly in the PR description.
2. Describe the migration — usually a new `DataKey` variant written alongside
   the old one, then a backfill.
3. Do not reuse or renumber an existing `Error` discriminant. Clients match on
   the integer.

Both contracts expose an admin-gated `upgrade(new_wasm_hash)` so a migration
can ship without moving contract ids or disturbing stake custody.

## Known dependency pin

`ed25519-dalek` is held at `2.x` in `Cargo.lock`. `soroban-env-host 22.x` does
not compile against dalek `3.0`. Dependabot is configured to skip that major,
and the ignore rule carries this note — remove it once the host crate moves.

## Troubleshooting

**`couldn't read .../danfo_registry.wasm`** — you skipped the registry build.
Run the first command in the [Quickstart](quickstart.md).

**Tests pass locally but fail in CI** — CI builds the registry wasm fresh. If
you changed the registry's public interface, the stale wasm in your `target/`
may be what your local rewards tests are importing. `cargo clean -p
danfo-registry` and rebuild.
