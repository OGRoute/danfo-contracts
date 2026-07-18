#![no_std]
//! danfo-registry — the correction lifecycle for Danfo's community-owned
//! Lagos transit knowledge base.
//!
//! Flow: `submit` (staked) -> `attest` (challenge window) -> `finalize`
//! (accept: stake refunded / reject: stake slashed to `slash_recipient`).

use soroban_sdk::contract;

#[contract]
pub struct Registry;
