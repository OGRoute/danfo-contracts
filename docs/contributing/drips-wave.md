# Drips Wave

This repository participates in the
[Stellar Wave](https://www.drips.network/wave/stellar) programme on
[Drips](https://drips.network). A Wave is a monthly cycle in which
contributors solve labelled issues across Stellar ecosystem repositories and
earn Points that convert to rewards.

## How a cycle works

| Phase | What happens here |
| --- | --- |
| **Scoping** | Maintainers label issues with a complexity and put them in the Wave. Nothing is claimable yet |
| **Sprint** (7 days) | Contributors apply for issues, get assigned, and open pull requests |
| **Reward** | Merged and resolved work is verified, and the pool is split by each contributor's share of Points |

## Points

Complexity is set by the maintainer when the issue is scoped, using the
[Wave task template](https://github.com/OGRoute/danfo-contracts/issues/new?template=task.yml):

| Complexity | Points | What it means in this repo |
| --- | --- | --- |
| **Trivial** | 100 | Under an hour. No protocol judgement — a docs fix, a missing test for an existing error path, a script flag |
| **Medium** | 150 | A few hours. One contract and its tests: a new read, a new guard, a bounded input |
| **High** | 200 | A day or more. Cross-contract, changes storage layout, or shifts an economic parameter |

Points reflect complexity and impact, not lines changed. A twelve-line change
to `finalize` is High; a four-hundred-line docs page is Trivial.

## Getting an issue assigned

1. Find the issue on the
   [Drips Explore page](https://www.drips.network/wave/stellar/issues) or in
   [this repository's issues](https://github.com/OGRoute/danfo-contracts/issues).
2. Apply through Drips. Say what you intend to do, not just that you want it —
   "I'll add the `token` equality check in `rewards::init` plus a
   `#[should_panic]` test" beats "can I take this".
3. **Wait to be assigned before writing code.** Unassigned pull requests are
   closed unmerged, which wastes your sprint.
4. Open a pull request that closes the issue. The maintainer reviews, merges,
   and marks it resolved in Drips.

Expect a review turnaround inside 48 hours during a sprint. If it stalls
longer, say so on the issue.

## What gets rejected

The [Wave terms](https://docs.drips.network/wave/terms-and-rules/) prohibit
low-effort reward farming, and this repository enforces that in review:

* Cosmetic reformatting, or typo churn dressed up as a fix.
* Generated code the author cannot explain. You will be asked why a guard is
  ordered the way it is — using an assistant is fine, not understanding the
  result is not.
* Pull requests against issues assigned to somebody else.
* Multiple accounts, or coordination to claim more work than one contributor
  is due.
* Anything that introduces a vulnerability, however "cleanly" it is written.

These are [Code of Conduct](https://github.com/OGRoute/danfo-contracts/blob/main/CODE_OF_CONDUCT.md)
violations here, not just declined patches.

## Maintainer notes

For whoever is scoping the next cycle:

* Apply the repository during the **Scoping** phase — applications are capped
  at five per Wave per organisation, and KYC is required.
* An issue is ready only when its "Definition of done" is a checklist a
  reviewer can tick without a follow-up question. If it is not, it is a
  discussion, not a Wave issue.
* Prefer issues that a stranger can finish inside one sprint. Anything needing
  a design decision from the maintainer mid-flight will not land in seven days.
* Keep a mix of complexities. A repository of nothing but High issues gets no
  first-time contributors; a repository of nothing but Trivial ones gets no
  durable ones.
