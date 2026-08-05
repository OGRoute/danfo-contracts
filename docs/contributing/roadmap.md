# Roadmap

What needs doing, sized for [Drips Wave](drips-wave.md) complexity. Each item
below is a real gap in the code as it stands — not a placeholder.

Items marked **open** have no assignee. Comment on the tracking issue before
starting.

## Correctness and safety

| Task | Complexity | Why |
| --- | --- | --- |
| Validate `rewards.init` `token` against `registry.get_config().token` | Medium | Today the two are only aligned by the deploy script. A hand-initialised pool can accept funds in one asset while the registry slashes into it in another — see the warning in [danfo-rewards](../reference/rewards.md#types) |
| Multisig or timelocked admin | High | A single admin key can `upgrade` either contract to arbitrary wasm. That is total compromise, including custodied stakes |
| Timelock on `upgrade` | High | Announce a wasm hash, wait, then apply, so users can exit a change they dislike |
| Property tests for the accept rule | Medium | `approvals + rejections >= min_votes && approvals > rejections` is currently covered by example tests only. A `proptest` sweep over vote counts would pin the boundary behaviour |
| Fuzz `page`/`recent` bounds | Medium | The clamps are tested at `u32::MAX` and at the end of the list, but not across arbitrary `(start, limit, count)` triples |

## Protocol

| Task | Complexity | Why |
| --- | --- | --- |
| Staked attestation | High | Voting costs nothing today, so `min_votes` sock puppets can approve their own correction. The hardest open design question in the protocol — [discuss first](../protocol/threat-model.md#open-problems) |
| Reputation-weighted quorum | High | Scale `min_votes` with the value of the route being edited, or with the attestors' accepted-correction history |
| Route-indexed reads | Medium | There is no way to fetch corrections for one `route_id` without scanning every id, so the indexer must replay the whole chain. Needs a `RouteIndex(String) -> Vec<u32>` key written on `submit` |
| Batch `finalize` | Medium | Settling a day's backlog is currently one transaction per correction |
| Contributor-scoped reads | Trivial | `reputation` gives counts but there is no `by_contributor(who, start, limit)` |

## Developer experience

| Task | Complexity | Why |
| --- | --- | --- |
| `justfile` or `Makefile` wrapping the ordered build | Trivial | The registry-before-rewards build order trips up every newcomer exactly once |
| Wasm size budget in CI | Trivial | Fail the build if either optimised wasm grows past a threshold; Soroban charges by size |
| `cargo-audit` job | Trivial | Advisory scanning, with the known `ed25519-dalek` pin allow-listed |
| Deploy script: `--dry-run` | Trivial | Print the invocations without sending them |
| Deploy script: mainnet guard | Trivial | Refuse `STELLAR_NETWORK=mainnet` while the contracts are unaudited |
| Testnet contract ids in the docs | Trivial | A published deployment anyone can invoke against beats a script they must run |

## Documentation

| Task | Complexity | Why |
| --- | --- | --- |
| Worked end-to-end CLI walkthrough | Trivial | Every `stellar contract invoke` for one correction, submit through claim, with real output |
| Indexer integration guide | Medium | How to reconstruct state from the [event stream](../reference/events.md) — the intended integration path, currently only sketched |
| Diagram the slash→pool→reward loop | Trivial | The economic argument in [Stake economics](../protocol/economics.md) is prose that wants a picture |

## Out of scope

Kept here so nobody spends a sprint on them:

* **Storing full correction payloads on-chain.** The chain stores a
  `payload_hash`; payload availability is deliberately an off-chain problem.
* **A token of our own.** Stakes and rewards use a Stellar Asset Contract —
  XLM on testnet, NGNC in production.
* **Governance.** Parameters are admin-tuned. On-chain governance is a
  different project.
