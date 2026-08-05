# danfo-rewards

A sponsor-funded pool paying a fixed reward per accepted correction. Claims
are pull-based and idempotent.

Source: [`contracts/rewards/src/lib.rs`](https://github.com/OGRoute/danfo-contracts/blob/main/contracts/rewards/src/lib.rs)

{% hint style="info" %}
This crate calls `contractimport!` on the registry's compiled wasm, so
`cargo build -p danfo-registry --release --target wasm32-unknown-unknown` must
run before anything that compiles this crate.
{% endhint %}

## Admin

### `init(config: RewardsConfig)`

One-shot initialisation.

* **Auth** — the incoming `config.admin`, so the deployment cannot be
  front-run.
* **Errors** — `AlreadyInitialized` (`#2`), `InvalidConfig` (`#6`) if
  `reward_amount <= 0`.
* **Emits** — `("init", admin)`.

### `set_reward(amount: i128)`

Change the per-correction payout. Takes effect on the next `claim`; already
claimed corrections are unaffected.

* **Auth** — admin.
* **Errors** — `NotInitialized` (`#1`), `InvalidConfig` (`#6`) if
  `amount <= 0`.
* **Emits** — `("config", amount)`.

### `upgrade(new_wasm_hash: BytesN<32>)`

Replace this contract's wasm in place. The pool's contract id and balance are
unaffected.

* **Auth** — admin.
* **Errors** — `NotInitialized` (`#1`).
* **Emits** — `("upgrade", ())`.

## Pool

### `fund(sponsor, amount)`

Transfer `amount` from `sponsor` into the pool.

* **Auth** — `sponsor`.
* **Errors** — `NotInitialized` (`#1`), `InvalidAmount` (`#7`) if
  `amount <= 0` — a negative transfer would otherwise read as a withdrawal.
* **Emits** — `("fund", sponsor), amount`.

{% hint style="warning" %}
Funding is one-way. There is no withdrawal path, deliberately: a sponsor who
could pull the pool back could strand contributors mid-cycle.
{% endhint %}

### `claim(id)`

Pay `reward_amount` for accepted correction `id` to its recorded contributor.

The contract reads `registry.get(id)` cross-contract and refuses unless the
status is `Accepted`. It never trusts the caller for the status or the
recipient.

* **Auth** — none. Anyone may crank it; the payout only ever reaches
  `correction.contributor` as read from the registry.
* **Errors** — `NotInitialized` (`#1`), `NotAccepted` (`#3`),
  `AlreadyClaimed` (`#4`), `InsufficientPool` (`#5`). Also propagates the
  registry's `BadId` (`#3`) for an id that does not exist.
* **Emits** — `("claim",), (id, contributor, amount)`.

The `Claimed(id)` flag is written **before** the token transfer, so a
re-entrant token contract cannot drain the pool through recursive claims.

## Reads

| Function | Returns | Notes |
| --- | --- | --- |
| `get_config()` | `RewardsConfig` | Live reward amount and linked registry |
| `pool()` | `i128` | Current balance |
| `is_claimed(id)` | `bool` | Whether that id has already been paid |
| `total_paid()` | `i128` | Lifetime payouts |

## Types

### `RewardsConfig`

```rust
pub struct RewardsConfig {
    pub admin: Address,
    pub token: Address,        // must match the registry's stake token
    pub registry: Address,     // danfo-registry contract id
    pub reward_amount: i128,   // > 0
}
```

{% hint style="danger" %}
`token` is not validated against the registry's token on `init` — the deploy
script sets both from the same variable. If you initialise this contract by
hand, passing a different SAC than the registry uses will let the pool accept
funds in one asset while the registry slashes into it in another. Making this
check on-chain is [an open task](../contributing/roadmap.md).
{% endhint %}

## Storage keys

```rust
pub enum DataKey {
    Config,          // instance
    Claimed(u32),    // persistent — idempotency flag per correction id
    TotalPaid,       // persistent
}
```
