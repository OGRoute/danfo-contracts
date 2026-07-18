# Security Policy

## Status

These contracts are **unaudited** and deployed to Stellar **testnet only**.
Do not use them to hold mainnet funds until an audit is completed and this
notice is updated.

## Scope

- `contracts/registry` — stake custody during the correction lifecycle.
- `contracts/rewards` — pool custody and payout idempotency.

Highest-value findings: any path where funds move to an address other than
the recorded contributor / slash recipient, double-claim or double-finalize
paths, and stake accounting errors.

## Reporting a vulnerability

Do **not** open a public issue for security reports. Email
**qozeemibrahim065@gmail.com** with a description, reproduction steps, and
impact. You will get an acknowledgement within 72 hours. Please allow up to
90 days for a fix before public disclosure.
