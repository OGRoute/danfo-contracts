---
description: >-
  Soroban smart contracts for Danfo — a community-owned transit knowledge
  protocol for Lagos, on Stellar.
---

# Danfo Contracts

Lagos moves on *danfo* minibuses. There are roughly 75,000 of them, and no
official map, no published fare table, and no authority that keeps either
current. Fares change with the fuel price. Routes change when a road closes.
The only people who know are the riders and drivers on the bus that morning.

Danfo lets those people maintain the data themselves — and puts economic weight
behind every edit, so the data is worth trusting.

## The idea in one paragraph

Anyone can propose a correction to the transit knowledge base, but proposing
one locks a stake. Peers review it during a challenge window. Accepted
corrections return the stake and pay a reward from a sponsor-funded pool.
Rejected ones lose the stake — straight into that same reward pool. Spam
therefore funds the people doing the real work.

## The two contracts

| Contract | Responsibility |
| --- | --- |
| [`danfo-registry`](reference/registry.md) | The correction lifecycle: staked `submit` → `attest` → `finalize` (refund or slash), plus reads and per-address reputation |
| [`danfo-rewards`](reference/rewards.md) | A sponsor-funded pool paying a fixed reward per accepted correction. Pull-based and idempotent |

The two are deliberately separated. `danfo-rewards` never trusts its caller: it
verifies acceptance with a cross-contract read of `danfo-registry.get(id)`, so
anyone may crank a claim and the payout still only ever reaches the
contributor recorded on-chain at submission time.

## Where to go next

* **Just want to run it?** → [Quickstart](getting-started/quickstart.md)
* **Want to understand the protocol?** → [Correction lifecycle](protocol/lifecycle.md)
* **Want the function-by-function API?** → [danfo-registry](reference/registry.md)
* **Want to contribute?** → [How to contribute](contributing/how-to-contribute.md)

## Status

{% hint style="warning" %}
These contracts are **unaudited** and deployed to Stellar **testnet only**. Do
not use them to hold mainnet funds. See
[SECURITY.md](https://github.com/OGRoute/danfo-contracts/blob/main/SECURITY.md)
for the disclosure process.
{% endhint %}

The application layer — the multilingual AI transit agent, the corrections
feed, and the indexer — lives in a separate repository, `danfo-app`. This
repository is the on-chain half and stands alone: everything here builds,
tests, and deploys without it.

## Licence

[MIT](https://github.com/OGRoute/danfo-contracts/blob/main/LICENSE).
