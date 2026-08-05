#!/usr/bin/env bash
# Create the labels and the starter issue backlog for a Drips Wave cycle.
#
# A Wave repository is only useful to the programme if it has scoped, sized
# work an outside contributor can pick up cold. This script creates that
# backlog from docs/contributing/roadmap.md so the repo is reviewable the
# moment it is applied.
#
# Prereqs: gh CLI, authenticated with write access to $REPO.
# Usage:   ./scripts/seed-issues.sh            # create labels + issues
#          ./scripts/seed-issues.sh --labels   # labels only
#          ./scripts/seed-issues.sh --dry-run  # print, send nothing
set -euo pipefail
cd "$(dirname "$0")/.."

REPO="${REPO:-OGRoute/danfo-contracts}"
DRY=0
LABELS_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY=1 ;;
    --labels)  LABELS_ONLY=1 ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

label() { # name colour description
  if [ "$DRY" = 1 ]; then echo "label: $1"; return; fi
  gh label create "$1" --repo "$REPO" --color "$2" --description "$3" --force >/dev/null
  echo "  label $1"
}

issue() { # title labels body
  if [ "$DRY" = 1 ]; then echo "issue: [$2] $1"; return; fi
  gh issue create --repo "$REPO" --title "$1" --label "$2" --body "$3" >/dev/null
  echo "  issue $1"
}

echo "[1/2] labels…"
label "wave"        "1d76db" "Scoped for a Drips Wave cycle"
label "trivial"     "c2e0c6" "Wave complexity: Trivial (100 points)"
label "medium"      "fbca04" "Wave complexity: Medium (150 points)"
label "high"        "d93f0b" "Wave complexity: High (200 points)"
label "registry"    "5319e7" "contracts/registry"
label "rewards"     "5319e7" "contracts/rewards"
label "tooling"     "bfd4f2" "CI, scripts, developer experience"
label "docs"        "0075ca" "Documentation"
label "security"    "b60205" "Affects funds, auth, or the threat model"
label "needs-design" "e99695" "Discuss the approach before writing code"

[ "$LABELS_ONLY" = 1 ] && { echo "done (labels only)"; exit 0; }

echo "[2/2] issues…"

DOD='
### Definition of done
- [ ] Behaviour implemented
- [ ] Doc comment states what it does, its auth, and its errors
- [ ] Test covers the happy path
- [ ] Test covers each new error path with `#[should_panic(expected = "Error(Contract, #N)")]`
- [ ] `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings` pass
- [ ] Docs under `docs/` updated if the public API changed
'

# ---------------------------------------------------------------- trivial ---

issue "tooling: add a justfile wrapping the ordered build" "wave,trivial,tooling" \
"## Context
\`danfo-rewards\` calls \`contractimport!\` on the registry's compiled wasm, so the
registry must be built first. A bare \`cargo test --workspace\` on a clean
checkout fails until \`cargo build -p danfo-registry --release --target
wasm32-unknown-unknown\` has run once. This trips up every newcomer exactly once.

## What to do
Add a \`justfile\` (or \`Makefile\`) at the repo root with \`build\`, \`test\`,
\`lint\`, and \`all\` recipes that encode the correct order. Update
\`docs/getting-started/quickstart.md\` to lead with it.

## Where to start
\`.github/workflows/ci.yml\` already has the correct order — mirror it.
$DOD"

issue "ci: fail the build if either optimised wasm exceeds a size budget" "wave,trivial,tooling" \
"## Context
Soroban charges by wasm size, so a size regression is a real cost regression
that nothing currently catches.

## What to do
After the existing wasm build steps in CI, run \`stellar contract optimize\` and
compare each output against a committed budget. Fail with a readable message
naming the current size, the budget, and the delta. Start the budget slightly
above today's sizes.

## Where to start
\`.github/workflows/ci.yml\`, after the two \`cargo build --target
wasm32-unknown-unknown\` steps.
$DOD"

issue "ci: add a cargo-audit job" "wave,trivial,tooling,security" \
"## Context
Nothing scans dependencies for advisories today.

## What to do
Add a job running \`cargo audit\`. Allow-list the known \`ed25519-dalek\` pin —
\`soroban-env-host 22.x\` does not compile against dalek 3.0, which is why
\`.github/dependabot.yml\` ignores that major. Keep the reason in the config so
it can be removed when the host crate moves.

## Where to start
\`.github/workflows/ci.yml\`, \`.github/dependabot.yml\`.
$DOD"

issue "deploy: add a --dry-run flag to scripts/deploy.sh" "wave,trivial,tooling" \
"## Context
There is no way to see what \`deploy.sh\` will do without it doing it.

## What to do
Add \`--dry-run\` that prints every \`stellar\` invocation — fully resolved, with
the config JSON — and sends none of them. Useful for reviewing the
\`set_config\` payload in step 6 before it goes out.

## Where to start
\`scripts/deploy.sh\`. \`scripts/seed-issues.sh\` already has the flag-parsing
shape to copy.
$DOD"

issue "deploy: refuse to run against mainnet while the contracts are unaudited" "wave,trivial,tooling,security" \
"## Context
\`STELLAR_NETWORK\` is a plain environment variable, so \`STELLAR_NETWORK=mainnet
./scripts/deploy.sh\` will happily deploy unaudited contracts that custody
stakes. \`SECURITY.md\` says testnet only; the script does not enforce it.

## What to do
Refuse any network other than \`testnet\` and \`futurenet\` unless
\`DANFO_ALLOW_MAINNET=1\` is set, with an error message pointing at SECURITY.md.

## Where to start
\`scripts/deploy.sh\`, near the \`NET=\` assignment.
$DOD"

issue "registry: add a contributor-scoped read" "wave,trivial,registry" \
"## Context
\`reputation(who)\` returns \`(submitted, accepted)\` counts, but there is no way
to list a contributor's actual corrections without scanning every id.

## What to do
Add \`by_contributor(who, start, limit) -> Vec<Correction>\`, clamped to
\`MAX_PAGE\` like \`page\`. A scan over ids is acceptable for a first pass;
say so in the doc comment. If you want it indexed instead, that overlaps with
the route-index issue — coordinate there first.

## Where to start
\`contracts/registry/src/lib.rs\`, alongside \`page\` and \`recent\`.
$DOD"

issue "docs: worked end-to-end CLI walkthrough" "wave,trivial,docs" \
"## Context
The docs describe the lifecycle and the API, but there is no page that walks
one correction from \`submit\` to \`claim\` with real commands and real output.

## What to do
Add \`docs/getting-started/walkthrough.md\`: every \`stellar contract invoke\` for
a single correction — submit, two attests, finalize, claim — with the actual
output pasted in, against a live testnet deployment. Add it to
\`docs/SUMMARY.md\` under Getting started.

## Where to start
\`docs/protocol/lifecycle.md\` has the sequence; this is the executable version.
$DOD"

issue "docs: diagram the slash-to-pool-to-reward loop" "wave,trivial,docs" \
"## Context
The economic argument in \`docs/protocol/economics.md\` is prose. The loop —
spam slashed into the pool that pays honest contributors — is the single most
important idea in the protocol and it wants a picture.

## What to do
Add a mermaid diagram to \`docs/protocol/economics.md\` showing both paths and
where the tokens actually move. \`docs/protocol/lifecycle.md\` has examples of
the mermaid blocks GitBook renders.
$DOD"

# ----------------------------------------------------------------- medium ---

issue "rewards: validate the configured token against the registry's" "wave,medium,rewards,security" \
"## Context
\`rewards.init\` takes a \`token\` and a \`registry\` independently. Nothing checks
that the token matches \`registry.get_config().token\` — today they are only
aligned because \`scripts/deploy.sh\` sets both from the same variable.

A hand-initialised pool with a mismatched token accepts sponsor funds in one
asset while the registry slashes into it in another, and pays claims in the
first. This is documented as a known gap in
\`docs/reference/rewards.md#types\`.

## What to do
In \`init\`, cross-contract read the registry's config and raise a new error
variant if the tokens differ. Add the variant at the end of the enum — do not
renumber existing discriminants. Update \`docs/reference/errors.md\` and remove
the warning hint from \`docs/reference/rewards.md\`.

## Where to start
\`contracts/rewards/src/lib.rs\`. \`claim\` already shows the cross-contract read
pattern via \`registry::Client\`.
$DOD"

issue "registry: property tests for the acceptance rule" "wave,medium,registry" \
"## Context
Acceptance is \`approvals + rejections >= min_votes && approvals > rejections\`.
It is covered by example tests only — quorum-not-met, tie, and clear majority.
The boundaries deserve a sweep.

## What to do
Add a \`proptest\` (dev-dependency) sweeping \`(approvals, rejections,
min_votes)\` and asserting the resulting \`Status\` matches an independently
written oracle. Confirm the two rules the docs promise: a tie rejects, and
silence rejects.

## Where to start
\`contracts/registry/src/test.rs\` — \`finalize_tie_rejects\` and
\`finalize_below_min_votes_slashes\` are the cases to generalise.
$DOD"

issue "registry: fuzz the page and recent bounds" "wave,medium,registry" \
"## Context
\`page(start, limit)\` and \`recent(n)\` clamp to \`MAX_PAGE\` and to the number of
corrections. Tested at \`u32::MAX\` and at the end of the list, but not across
arbitrary \`(start, limit, count)\` triples — including \`start + limit\`
overflowing \`u32\`.

## What to do
Property-test that both reads never panic, never return more than \`MAX_PAGE\`,
never return more than exist, and return items in the documented order.

## Where to start
\`contracts/registry/src/lib.rs\` \`page\`/\`recent\`;
\`page_walks_forward_and_stops_at_end\` in the tests.
$DOD"

issue "registry: index corrections by route_id" "wave,medium,registry" \
"## Context
There is no way to fetch corrections for one \`route_id\` without scanning every
id, so the indexer has to replay the whole chain to answer \"what changed on
cms-oshodi\". This is listed under Open problems in the threat model.

## What to do
Write a \`RouteIndex(String)\` persistent key on \`submit\` holding the ids for
that route, and add \`by_route(route_id, start, limit)\` clamped to
\`MAX_PAGE\`. Extend TTL on the index key like every other persistent write.

Think about the unbounded-growth case for a popular route and say in the doc
comment how it degrades.

## Where to start
\`contracts/registry/src/lib.rs\` \`submit\`; \`DataKey\` in \`types.rs\`.
$DOD"

issue "registry: batch finalize" "wave,medium,registry" \
"## Context
Settling a day's backlog is one transaction per correction.

## What to do
Add \`finalize_batch(ids: Vec<u32>) -> Vec<Status>\`, capped at \`MAX_PAGE\`.
Decide and document whether one bad id fails the batch or is skipped — the
skip semantics are friendlier for a cranking bot, but must be visible in the
return value.

## Where to start
\`contracts/registry/src/lib.rs\` \`finalize\`.
$DOD"

issue "docs: indexer integration guide" "wave,medium,docs" \
"## Context
\`docs/reference/events.md\` lists the events and sketches how to reconstruct
state, but there is no guide for actually building the indexer — which is the
intended integration path for the application layer.

## What to do
Add \`docs/reference/indexing.md\`: cursor management, handling the \`config\`
event when replaying across a \`set_config\`, reorg/retry behaviour, and a
worked example that rebuilds one correction's full history from the stream.
Add it to \`docs/SUMMARY.md\`.

## Where to start
\`docs/reference/events.md\`, and the \`stellar events\` command at the bottom
of it.
$DOD"

# ------------------------------------------------------------------- high ---

issue "protocol: staked attestation" "wave,high,registry,security,needs-design" \
"## Context
Attestation is one address, one vote. Nothing is staked on a vote and there is
no penalty for voting wrongly, so anyone controlling \`min_votes\` addresses can
approve their own correction from sock puppets. At the launch quorum of 2 that
is cheap. This is the hardest open design question in the protocol.

## What to do
**Discuss before writing code.** Bring a proposal covering: what a voter
stakes, what a losing voter forfeits and to whom, how it interacts with the
existing \`finalize\` refund/slash paths, and what it does to the pool solvency
arithmetic in \`docs/protocol/economics.md\`.

Implementation only after the approach is agreed on the issue.

## Where to start
\`docs/protocol/threat-model.md#not-defended\`,
\`docs/protocol/economics.md\`.
$DOD"

issue "protocol: multisig or timelocked admin" "wave,high,security,needs-design" \
"## Context
The admin is a single key that can \`upgrade\` either contract to arbitrary
wasm. That is total compromise, including custodied stakes. It is the largest
trust concentration in the system and it is documented as such.

## What to do
Replace the single admin with a threshold scheme — a Soroban multisig account,
or an on-chain m-of-n in the contract. Cover the migration for the already
deployed testnet instances and update
\`docs/protocol/threat-model.md#trust-assumptions\`.

Bring the approach to the issue before implementing.

## Where to start
\`Config.admin\` in \`contracts/registry/src/types.rs\`; \`RewardsConfig.admin\`
in \`contracts/rewards/src/lib.rs\`; both \`upgrade\` functions.
$DOD"

issue "protocol: timelock the upgrade path" "wave,high,security,needs-design" \
"## Context
\`upgrade(new_wasm_hash)\` applies immediately on admin auth. Users holding
in-flight stakes have no window to exit a change they dislike.

## What to do
Split it: \`propose_upgrade(hash)\` records the hash and a ready-at timestamp
and emits an event; \`apply_upgrade()\` only succeeds after the delay, and only
for the announced hash. Add \`cancel_upgrade()\`. Decide the delay relative to
\`challenge_window\` — it should be long enough that any correction in flight
when a change is announced can settle under the old code.

Pairs naturally with the multisig issue; either can land first.

## Where to start
\`upgrade\` in both \`contracts/registry/src/lib.rs\` and
\`contracts/rewards/src/lib.rs\`.
$DOD"

echo
echo "done. Review at https://github.com/$REPO/issues"
