<!--
Thanks for contributing. Keep one logical change per PR — it keeps review
fast, which is the only thing that makes a 7-day Wave sprint workable.
-->

## What this changes

<!-- One or two sentences. Link the issue: "Closes #12". -->

## Why

<!-- The user-facing or protocol reason. If there isn't one, say so. -->

## Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] Every new public function has a `///` doc naming its auth and errors
- [ ] Every new error path has a `#[should_panic(expected = "Error(Contract, #N)")]` test
- [ ] Every new state change emits an event and extends TTL on persistent writes
- [ ] Docs under `docs/` updated if the public API or protocol behaviour changed

## Storage & compatibility

- [ ] This PR does **not** change the layout of an existing `#[contracttype]`
- [ ] …or it does, and the migration is described below

<!-- If you changed a stored type or an error discriminant, explain the
     migration. Deployed contracts hold real testnet stakes. -->
