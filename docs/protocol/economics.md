# Stake economics

## The loop

```
spam submitted → rejected → stake slashed → reward pool
                                                 ↓
good correction submitted → accepted → stake refunded + reward paid
```

The design goal is a single sentence: **make the cost of being wrong land on
the person who was wrong, and route it to the people who were right.**

## Launch parameters

| Parameter | Default | Where it lives |
| --- | --- | --- |
| Stake | 10 XLM | `registry.Config.stake_amount` |
| Challenge window | 24 hours | `registry.Config.challenge_window` |
| Minimum attestations | 2 | `registry.Config.min_votes` |
| Reward per accepted correction | 5 XLM | `rewards.RewardsConfig.reward_amount` |

Read them live with `registry.get_config()` and `rewards.get_config()`. Never
hardcode them in a client — they are admin-tunable by design, because the
right stake in Lagos is an empirical question, not one that can be settled
before launch.

## Why the reward is smaller than the stake

A contributor who submits a good correction risks 10 XLM to earn 5. That looks
backwards until you notice the stake is *refunded*: their expected value is
`+5 XLM` when they are right and `-10 XLM` when they are wrong. Being right
has to be more than twice as likely as being wrong before submitting pays,
which is roughly the bar we want for someone claiming to know today's fare.

A spammer's arithmetic is worse. Spam has no realistic path to two approving
attestations, so every submission is `-10 XLM` with near-certainty, and the
loss lands directly in the pool that pays the people who reported honestly.

## Bounds on the parameters

`init` and `set_config` both reject configurations that would break the loop:

| Rule | Why |
| --- | --- |
| `stake_amount > 0` | A zero stake means nothing is at risk, and spam becomes free |
| `min_votes > 0` | A zero quorum means an unreviewed correction can be accepted |
| `challenge_window > 0` | A zero window means `finalize` is callable in the same ledger as `submit`, before anyone could review |
| `challenge_window <= 30 days` | Bounds both operator error and stakes being stranded for an unbounded period |

Violations raise `InvalidConfig` (`#8`).

## Why the token is immutable

`set_config` will change every field *except* `token`, which raises
`TokenImmutable` (`#10`).

The reason is custody. Stakes are transferred in at `submit` time in whatever
token the config named then, and they sit in the contract until `finalize`.
If an admin could swap the token mid-flight, `finalize` would try to refund or
slash in an asset the contract never received — draining an unrelated balance
if it happened to hold one, or reverting and stranding the stake if it did
not. Freezing the token removes the whole class of problem, at the cost of
requiring a redeploy to migrate assets.

## Pool solvency

The pool is not automatically solvent. `claim` checks the balance and raises
`InsufficientPool` (`#5`) rather than paying partially or going into debt.

Two inflows keep it topped up:

* **Sponsors** call `fund(sponsor, amount)`. This is the launch capital.
* **Slashing** delivers every rejected stake, once `slash_recipient` points at
  the pool — step 7 of [the deploy script](../getting-started/deploying.md).

At steady state, with a rejection rate `r`, a stake `s`, and a reward `w`, the
pool is self-sustaining when `r · s ≥ (1 − r) · w`. With the launch numbers
(`s = 10`, `w = 5`) that is `r ≥ 1/3`. Below a one-in-three rejection rate the
pool needs ongoing sponsorship — which is the expected and desirable state,
since it means most contributions are good.

## Currency

Stakes and rewards are denominated in a Stellar Asset Contract. Testnet uses
native XLM. The intended production asset is **NGNC**, Stellar's naira
stablecoin, which is redeemable to a Nigerian bank account through SEP-24 —
so a rider in Lagos can convert a reward into spendable naira without touching
an exchange.
