# Threat model

What these contracts defend against, what they do not, and where the open
problems are. If you are looking for something meaningful to work on, the
"Open problems" section is the honest list.

## Trust assumptions

| Actor | Trusted with |
| --- | --- |
| Admin | Tuning stake, window, quorum, reward; upgrading either contract's wasm. **Not** trusted with user stakes — no admin path moves a locked stake |
| Sponsor | Nothing. Funding is one-way; there is no withdrawal path |
| Contributor | Nothing. Their stake is custodied by the contract |
| Attestor | Their vote, one per address per correction |
| Anyone | Cranking `finalize` and `claim`. Neither can redirect funds |

The admin is the largest trust concentration and is currently a single key.
See [Open problems](#open-problems).

## Defended

**Deploy front-running.** `init` requires the incoming `admin` to authorise
the call, so an attacker watching for an uninitialised deployment cannot
initialise it into their own control.

**Double-initialisation.** Both contracts raise `AlreadyInitialized` (`#2`) on
a second `init`.

**Double-voting.** A `Voted(id, address)` key is written on first vote;
`AlreadyVoted` (`#4`) on any repeat.

**Self-approval.** `SelfVote` (`#6`) blocks a contributor attesting to their
own correction.

**Double-finalisation.** `finalize` requires `Status::Pending`, so a
correction's stake can only move once. `NotPending` (`#5`) otherwise.

**Double-claiming.** `Claimed(id)` is written **before** the token transfer,
so a re-entrant token contract cannot drain the pool through recursive claims.
`AlreadyClaimed` (`#4` in rewards) on any repeat.

**Claiming against an unaccepted correction.** `claim` reads
`registry.get(id)` cross-contract and requires `Accepted`. It never trusts a
caller-supplied status. `NotAccepted` (`#3`).

**Early finalisation.** `WindowNotElapsed` (`#7`) until `finalize_after`.

**Token substitution mid-flight.** `set_config` cannot change `token` —
`TokenImmutable` (`#10`). See
[Stake economics](economics.md#why-the-token-is-immutable).

**Config that breaks the incentive loop.** Zero stake, zero quorum, zero or
over-long window all raise `InvalidConfig` (`#8`).

**Storage griefing through unbounded strings.** `route_id` is capped at 64
bytes and `summary` at 280, checked before the stake transfer, so a rejected
submission costs only the transaction fee. `InvalidInput` (`#9`).

**Unbounded reads.** `recent` and `page` clamp their size to `MAX_PAGE` (50),
so a read is never sized by untrusted input.

**Arithmetic overflow.** The release profile sets `overflow-checks = true`, so
counter and balance arithmetic traps rather than wrapping. The 30-day cap on
`challenge_window` also bounds `submitted_at + challenge_window`.

**State expiry.** Every persistent write extends its TTL by ~90 days with a
~30-day threshold.

## Not defended

**Attestation is one-address, one-vote.** Nothing is staked on a vote and
there is no penalty for voting wrongly, so an actor who controls `min_votes`
addresses can approve their own correction from sock puppets. At the launch
quorum of 2 this is cheap. Mitigations under consideration: staked
attestation, reputation-weighted votes, or a quorum that scales with the value
of the route being edited.

**Off-chain payload availability.** The chain stores `payload_hash`, not the
payload. If the off-chain JSON disappears, the commitment is unverifiable.
The hash proves integrity, never availability.

**Attestor apathy.** Because silence rejects, an honest correction that nobody
reviews loses its stake. This is the correct failure direction — the bad
outcome is bad data being accepted — but it is a real cost borne by honest
contributors in a low-traffic period.

**Admin key compromise.** A compromised admin can `upgrade` either contract to
arbitrary wasm, which is total compromise including custodied stakes.

**Economic griefing of the pool.** An attacker who can reliably get
corrections accepted drains the pool at `reward_amount` each. The defence is
the stake plus the quorum, i.e. the same defence as against sock puppets.

## Open problems

Tracked as issues in the repository, and genuinely open — these are not
rhetorical:

1. **Staked attestation.** Make voting cost something, and make a losing vote
   cost more. Hardest open design question in the protocol.
2. **Multisig admin.** Replace the single admin key with a Soroban multisig or
   a timelock, especially before the `upgrade` path is used on anything
   holding value.
3. **Timelock on `upgrade`.** Announce a wasm hash, wait, then apply — so
   users can exit a change they dislike.
4. **Route-indexed reads.** There is currently no way to fetch corrections for
   a given `route_id` without scanning every id, which forces the indexer to
   replay the whole chain.
5. **Sybil resistance on reputation.** `reputation(who)` counts submissions
   and acceptances per address; addresses are free.

## Reporting

Do **not** open a public issue for a vulnerability. Follow
[SECURITY.md](https://github.com/OGRoute/danfo-contracts/blob/main/SECURITY.md).
Highest-value findings are any path where funds move to an address other than
the recorded contributor or slash recipient, any double-claim or
double-finalize path, and any stake accounting error.
