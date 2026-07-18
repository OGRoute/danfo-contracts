#![no_std]
//! danfo-registry — the correction lifecycle for Danfo's community-owned
//! Lagos transit knowledge base.
//!
//! Flow: `submit` (staked) -> `attest` (challenge window) -> `finalize`
//! (accept: stake refunded / reject: stake slashed to `slash_recipient`).

mod types;

use soroban_sdk::{contract, contractimpl, panic_with_error, symbol_short, token, Env};

pub use types::{Config, Correction, DataKey, Error, Kind, Status};
use types::{TTL_EXTEND, TTL_THRESHOLD};

#[contract]
pub struct Registry;

/// Read config or panic `NotInitialized`.
fn config(env: &Env) -> Config {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .unwrap_or_else(|| panic_with_error!(env, Error::NotInitialized))
}

/// Extend the TTL of a persistent key after writing it.
fn bump(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, TTL_THRESHOLD, TTL_EXTEND);
}

#[contractimpl]
impl Registry {}
