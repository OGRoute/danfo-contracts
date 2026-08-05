# Deploying to testnet

```bash
./scripts/deploy.sh
```

That is the whole thing. The script is idempotent about network and identity
setup, so it is safe to re-run.

## What the script does

Order matters here, because the two contracts reference each other:

1. Ensures the `testnet` network and a funded identity exist.
2. Builds both wasms — **registry first**, since rewards imports its interface.
3. Optimises both with `stellar contract optimize`.
4. Resolves the native XLM Stellar Asset Contract id.
5. Deploys and `init`s the registry, with `slash_recipient` temporarily
   pointed at the admin — the rewards pool does not exist yet.
6. Deploys and `init`s the rewards pool, pointing it at the registry.
7. Calls `registry.set_config` to move `slash_recipient` onto the rewards
   pool. **This is the step that closes the economic loop**: from here,
   slashed spam tops up the reward pool.

It prints the three ids to paste into the application layer's environment.

{% hint style="info" %}
Both `init` functions require the incoming `admin` to authorise the call, so a
deployed-but-uninitialised contract cannot be front-run into someone else's
control. The script satisfies this by invoking with `--source "$IDENT"`, which
is the same key it puts in the config.
{% endhint %}

## Tunables

All read from the environment, with launch defaults:

| Variable | Default | Meaning |
| --- | --- | --- |
| `STELLAR_NETWORK` | `testnet` | Target network |
| `STELLAR_IDENTITY` | `danfo` | Stellar CLI identity used as admin |
| `STELLAR_RPC_URL` | `https://soroban-testnet.stellar.org` | RPC endpoint |
| `STAKE_AMOUNT` | `100000000` | 10 XLM in stroops |
| `CHALLENGE_WINDOW` | `86400` | 24 hours, in seconds |
| `MIN_VOTES` | `2` | Attestations required before a correction can be accepted |
| `REWARD_AMOUNT` | `50000000` | 5 XLM in stroops |

```bash
CHALLENGE_WINDOW=3600 MIN_VOTES=3 ./scripts/deploy.sh
```

Every one of these stays tunable after deploy through `registry.set_config`
and `rewards.set_reward`, both admin-only. The only field that is **not**
mutable is the registry's `token` — see [Stake economics](../protocol/economics.md#why-the-token-is-immutable).

## Funding the pool

A freshly deployed pool is empty, so `claim` fails with `InsufficientPool`
(`#5`) until a sponsor funds it:

```bash
stellar contract invoke --id "$REWARDS" --source sponsor --network testnet -- \
  fund --sponsor "$(stellar keys address sponsor)" --amount 1000000000
```

Slashed stakes flow in automatically once step 7 above has run, but a launch
still needs seed capital — early corrections are accepted before much spam has
been rejected.

## Verifying a deployment

```bash
# Config the contracts are actually running with
stellar contract invoke --id "$REGISTRY" --network testnet -- get_config
stellar contract invoke --id "$REWARDS"  --network testnet -- get_config

# Pool balance and lifetime payouts
stellar contract invoke --id "$REWARDS" --network testnet -- pool
stellar contract invoke --id "$REWARDS" --network testnet -- total_paid
```

`get_config` on both contracts exists precisely so clients read live values
rather than hardcoding the launch defaults above.

## Upgrading

Both contracts expose `upgrade(new_wasm_hash)`, admin-authorised:

```bash
HASH=$(stellar contract install --wasm "$NEW_WASM" --source danfo --network testnet)
stellar contract invoke --id "$REGISTRY" --source danfo --network testnet -- \
  upgrade --new_wasm_hash "$HASH"
```

Contract ids and stake custody are unaffected. Read
[Local development → Adding a stored field](local-development.md#adding-a-stored-field)
before upgrading across a storage-layout change.
