# Errors

Both contracts raise typed contract errors. A caller sees
`Error(Contract, #N)`, where `N` is the discriminant below.

{% hint style="warning" %}
The discriminants are part of the public interface. Clients match on the
integer, so **never renumber or reuse one** — add new variants at the end.
{% endhint %}

## danfo-registry

| # | Variant | Raised when |
| --- | --- | --- |
| 1 | `NotInitialized` | Any function that reads config, before `init` |
| 2 | `AlreadyInitialized` | `init` called a second time |
| 3 | `BadId` | `get`, `attest`, or `finalize` on an id that was never submitted |
| 4 | `AlreadyVoted` | `attest` from an address that already voted on that id |
| 5 | `NotPending` | `attest` or `finalize` on a correction already settled |
| 6 | `SelfVote` | `attest` by the correction's own contributor |
| 7 | `WindowNotElapsed` | `finalize` before `finalize_after` |
| 8 | `InvalidConfig` | `init` or `set_config` with `stake_amount <= 0`, `min_votes == 0`, `challenge_window == 0`, or `challenge_window > 30 days` |
| 9 | `InvalidInput` | `submit` with an empty or over-length `route_id` (>64 bytes) or `summary` (>280 bytes) |
| 10 | `TokenImmutable` | `set_config` attempting to change `token` |

## danfo-rewards

| # | Variant | Raised when |
| --- | --- | --- |
| 1 | `NotInitialized` | Any function that reads config, before `init` |
| 2 | `AlreadyInitialized` | `init` called a second time |
| 3 | `NotAccepted` | `claim` for a correction that is not `Accepted` |
| 4 | `AlreadyClaimed` | `claim` for an id already paid |
| 5 | `InsufficientPool` | `claim` when the pool balance is below `reward_amount` |
| 6 | `InvalidConfig` | `init` or `set_reward` with `reward_amount <= 0` |
| 7 | `InvalidAmount` | `fund` with `amount <= 0` |

{% hint style="info" %}
`claim` invokes the registry, so a registry error surfaces to the caller
unchanged. `Error(Contract, #3)` from a `claim` may therefore be either the
rewards contract's `NotAccepted` **or** the registry's `BadId`. Distinguish
them by calling `registry.get(id)` directly.
{% endhint %}

## Testing an error path

Every error above has a test. The house rule is that a new error variant
lands with its own `#[should_panic]` case:

```rust
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn submit_overlong_route_panics() {
    let env = Env::default();
    let s = setup(&env);
    let alice = Address::generate(&env);
    s.mint.mint(&alice, &1_000);
    s.client.submit(
        &alice,
        &String::from_str(&env, OVERLONG_ROUTE),
        &Kind::Fare,
        &BytesN::from_array(&env, &[7u8; 32]),
        &String::from_str(&env, "Fare is now 500 naira"),
    );
}
```

To assert on an error **without** unwinding — for example to check that a
failed call left balances untouched — use the generated `try_` variant:

```rust
let bad = s.client.try_submit(/* ... */);
assert!(bad.is_err());
assert_eq!(s.token.balance(&alice), 1_000); // stake was never taken
```
