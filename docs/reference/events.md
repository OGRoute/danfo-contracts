# Events

Every state change publishes an event. This is the intended integration
surface: the indexer in `danfo-app` follows the chain through events rather
than polling reads, so a client can reconstruct full protocol state from the
event stream alone.

Topics are Soroban `Symbol`s, which cap at 9 characters — hence `final`
rather than `finalize`.

## danfo-registry

| Topics | Data | Published by |
| --- | --- | --- |
| `("init",)` | `admin: Address` | `init` |
| `("config",)` | `new_admin: Address` | `set_config` |
| `("upgrade",)` | `()` | `upgrade` |
| `("submit", contributor)` | `(id: u32, route_id: String, kind: Kind)` | `submit` |
| `("attest", voter)` | `(id: u32, approve: bool)` | `attest` |
| `("final",)` | `(id: u32, status: Status)` | `finalize` |

`submit` and `attest` put the acting address in the **topics**, not the data,
so an indexer can subscribe to one contributor's activity without filtering
the whole stream client-side.

## danfo-rewards

| Topics | Data | Published by |
| --- | --- | --- |
| `("init",)` | `admin: Address` | `init` |
| `("config",)` | `reward_amount: i128` | `set_reward` |
| `("upgrade",)` | `()` | `upgrade` |
| `("fund", sponsor)` | `amount: i128` | `fund` |
| `("claim",)` | `(id: u32, contributor: Address, amount: i128)` | `claim` |

## Reconstructing state

A correction's full history is three events:

```
("submit", alice)  (7, "cms-oshodi", Fare)
("attest", bob)    (7, true)
("attest", carol)  (7, true)
("final",)         (7, Accepted)
("claim",)         (7, alice, 50000000)
```

Balances follow from the same stream: a `submit` locks `stake_amount` as of
that ledger, a `final` with `Accepted` refunds it, a `final` with `Rejected`
moves it to `slash_recipient`, and a `claim` pays `reward_amount`.

{% hint style="info" %}
`submit` does not publish the stake amount, so an indexer replaying history
across a `set_config` must track `stake_amount` changes from the `config`
event — or read `get(id).stake`, which is authoritative and stored per
correction.
{% endhint %}

## Watching a live deployment

```bash
stellar events --start-ledger <LEDGER> --id "$REGISTRY" --network testnet
```
