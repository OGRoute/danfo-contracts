# danfo-registry

The correction lifecycle: staked `submit` → `attest` → `finalize`.

Source: [`contracts/registry/src/lib.rs`](https://github.com/OGRoute/danfo-contracts/blob/main/contracts/registry/src/lib.rs)

## Admin

### `init(config: Config)`

One-shot initialisation.

* **Auth** — the incoming `config.admin`, so a deployed-but-uninitialised
  contract cannot be front-run.
* **Errors** — `AlreadyInitialized` (`#2`), `InvalidConfig` (`#8`).
* **Emits** — `("init", admin)`.

### `set_config(new_config: Config)`

Replace the configuration. Used after deploy to point `slash_recipient` at the
rewards pool.

* **Auth** — current admin.
* **Errors** — `NotInitialized` (`#1`), `TokenImmutable` (`#10`) if `token`
  differs from the current one, `InvalidConfig` (`#8`).
* **Emits** — `("config", new_admin)`.

### `upgrade(new_wasm_hash: BytesN<32>)`

Replace this contract's wasm in place. Contract id and stake custody are
unaffected.

* **Auth** — admin.
* **Errors** — `NotInitialized` (`#1`).
* **Emits** — `("upgrade", ())`.

## Lifecycle

### `submit(contributor, route_id, kind, payload_hash, summary) -> u32`

Lock `stake_amount` and record a correction. Returns its 0-based id.

| Parameter | Type | Notes |
| --- | --- | --- |
| `contributor` | `Address` | Must authorise; receives the refund on acceptance |
| `route_id` | `String` | Route slug, 1–64 bytes, e.g. `cms-oshodi` |
| `kind` | `Kind` | `Fare` \| `Route` \| `Closure` |
| `payload_hash` | `BytesN<32>` | sha256 of the off-chain JSON payload |
| `summary` | `String` | 1–280 bytes, human-readable |

* **Auth** — `contributor`.
* **Errors** — `NotInitialized` (`#1`), `InvalidInput` (`#9`) if either string
  is empty or over its cap. The length checks run **before** the stake
  transfer.
* **Emits** — `("submit", contributor), (id, route_id, kind)`.

### `attest(voter, id, approve)`

Approve or reject a pending correction during its challenge window.

* **Auth** — `voter`.
* **Errors** — `BadId` (`#3`), `NotPending` (`#5`), `SelfVote` (`#6`),
  `AlreadyVoted` (`#4`).
* **Emits** — `("attest", voter), (id, approve)`.

### `finalize(id) -> Status`

Settle a correction after its window. Accept refunds the stake; reject sends
it to `slash_recipient`.

Acceptance requires `approvals + rejections >= min_votes` **and**
`approvals > rejections` — a tie rejects, and so does silence.

* **Auth** — none. Anyone may crank it; funds only move to addresses recorded
  at submission.
* **Errors** — `NotInitialized` (`#1`), `BadId` (`#3`), `NotPending` (`#5`),
  `WindowNotElapsed` (`#7`).
* **Emits** — `("final",), (id, status)`.

## Reads

None of these take auth or cost state.

| Function | Returns | Notes |
| --- | --- | --- |
| `get(id)` | `Correction` | `BadId` (`#3`) if absent |
| `get_config()` | `Config` | Live parameters — read these instead of hardcoding launch defaults |
| `total()` | `u32` | Corrections ever submitted |
| `recent(n)` | `Vec<Correction>` | Newest first. `n` clamped to `MAX_PAGE` (50) |
| `page(start, limit)` | `Vec<Correction>` | Submission order from `start`. `limit` clamped to 50. Empty past the end, so an indexer can poll from its cursor without bounds checks |
| `has_voted(id, voter)` | `bool` | Whether that address already attested |
| `reputation(who)` | `(u32, u32)` | `(submitted, accepted)` |

## Types

### `Correction`

```rust
pub struct Correction {
    pub contributor: Address,
    pub route_id: String,       // 1..=64 bytes
    pub kind: Kind,
    pub payload_hash: BytesN<32>,
    pub summary: String,        // 1..=280 bytes
    pub stake: i128,            // locked at submission
    pub status: Status,
    pub submitted_at: u64,
    pub finalize_after: u64,    // submitted_at + challenge_window
    pub approvals: u32,
    pub rejections: u32,
}
```

`stake` is stored per-correction rather than read from config at finalize
time, so changing `stake_amount` never alters what an in-flight correction is
owed.

### `Config`

```rust
pub struct Config {
    pub admin: Address,
    pub token: Address,          // SAC; immutable after init
    pub stake_amount: i128,      // > 0
    pub challenge_window: u64,   // > 0, <= 30 days, in seconds
    pub min_votes: u32,          // > 0
    pub slash_recipient: Address,
}
```

### `Kind` and `Status`

```rust
pub enum Kind   { Fare = 0, Route = 1, Closure = 2 }
pub enum Status { Pending = 0, Accepted = 1, Rejected = 2 }
```

## Constants

| Constant | Value | Meaning |
| --- | --- | --- |
| `MAX_ROUTE_ID_LEN` | 64 | Bytes |
| `MAX_SUMMARY_LEN` | 280 | Bytes |
| `MAX_PAGE` | 50 | Largest page any read returns |
| `MAX_CHALLENGE_WINDOW` | 2,592,000 | 30 days in seconds |
| `TTL_THRESHOLD` | 518,400 | ~30 days in ledgers |
| `TTL_EXTEND` | 1,555,200 | ~90 days in ledgers |

## Storage keys

```rust
pub enum DataKey {
    Config,                     // instance
    Count,                      // persistent — next correction id
    Correction(u32),            // persistent
    Voted(u32, Address),        // persistent — one vote per address per id
    AcceptedCount(Address),     // persistent — reputation
    SubmittedCount(Address),    // persistent — reputation
}
```
