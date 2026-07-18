#![no_std]
//! danfo-registry — the correction lifecycle for Danfo's community-owned
//! Lagos transit knowledge base.
//!
//! Flow: `submit` (staked) -> `attest` (challenge window) -> `finalize`
//! (accept: stake refunded / reject: stake slashed to `slash_recipient`).

mod test;
mod types;

use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, token, Address, BytesN, Env, String,
    Vec,
};

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
impl Registry {
    /// One-shot initialization. Errors `AlreadyInitialized` on a second call.
    pub fn init(env: Env, config: Config) {
        if env.storage().instance().has(&DataKey::Config) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
        env.events()
            .publish((symbol_short!("init"),), config.admin);
    }

    /// Replace the configuration. Auth: current admin.
    /// Used after deploy to point `slash_recipient` at the rewards pool.
    pub fn set_config(env: Env, new_config: Config) {
        let current = config(&env);
        current.admin.require_auth();
        env.storage().instance().set(&DataKey::Config, &new_config);
        env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
        env.events()
            .publish((symbol_short!("config"),), new_config.admin);
    }

    /// Submit a correction, locking `stake_amount` of the config token.
    /// Auth: `contributor`. Returns the new 0-based correction id.
    pub fn submit(
        env: Env,
        contributor: Address,
        route_id: String,
        kind: Kind,
        payload_hash: BytesN<32>,
        summary: String,
    ) -> u32 {
        contributor.require_auth();
        let cfg = config(&env);

        token::Client::new(&env, &cfg.token).transfer(
            &contributor,
            &env.current_contract_address(),
            &cfg.stake_amount,
        );

        let s = env.storage().persistent();
        let id: u32 = s.get(&DataKey::Count).unwrap_or(0);
        let now = env.ledger().timestamp();

        let correction = Correction {
            contributor: contributor.clone(),
            route_id: route_id.clone(),
            kind,
            payload_hash,
            summary,
            stake: cfg.stake_amount,
            status: Status::Pending,
            submitted_at: now,
            finalize_after: now + cfg.challenge_window,
            approvals: 0,
            rejections: 0,
        };

        let ckey = DataKey::Correction(id);
        s.set(&ckey, &correction);
        bump(&env, &ckey);

        s.set(&DataKey::Count, &(id + 1));
        bump(&env, &DataKey::Count);

        let skey = DataKey::SubmittedCount(contributor.clone());
        let submitted: u32 = s.get(&skey).unwrap_or(0);
        s.set(&skey, &(submitted + 1));
        bump(&env, &skey);

        env.events().publish(
            (symbol_short!("submit"), contributor),
            (id, route_id, kind),
        );
        id
    }

    /// Approve or reject a pending correction during its challenge window.
    /// Auth: `voter`. One vote per address; contributors cannot vote on
    /// their own corrections.
    pub fn attest(env: Env, voter: Address, id: u32, approve: bool) {
        voter.require_auth();
        let s = env.storage().persistent();

        let ckey = DataKey::Correction(id);
        let mut correction: Correction = s
            .get(&ckey)
            .unwrap_or_else(|| panic_with_error!(&env, Error::BadId));
        if correction.status != Status::Pending {
            panic_with_error!(&env, Error::NotPending);
        }
        if correction.contributor == voter {
            panic_with_error!(&env, Error::SelfVote);
        }

        let vkey = DataKey::Voted(id, voter.clone());
        if s.get::<_, bool>(&vkey).unwrap_or(false) {
            panic_with_error!(&env, Error::AlreadyVoted);
        }
        s.set(&vkey, &true);
        bump(&env, &vkey);

        if approve {
            correction.approvals += 1;
        } else {
            correction.rejections += 1;
        }
        s.set(&ckey, &correction);
        bump(&env, &ckey);

        env.events()
            .publish((symbol_short!("attest"), voter), (id, approve));
    }

    /// Settle a correction after its challenge window. No auth — anyone may
    /// crank this; funds only ever move to recorded addresses.
    /// Accept requires `approvals + rejections >= min_votes` and a strict
    /// approval majority; accept refunds the stake, reject slashes it to
    /// `slash_recipient`.
    pub fn finalize(env: Env, id: u32) -> Status {
        let cfg = config(&env);
        let s = env.storage().persistent();

        let ckey = DataKey::Correction(id);
        let mut correction: Correction = s
            .get(&ckey)
            .unwrap_or_else(|| panic_with_error!(&env, Error::BadId));
        if correction.status != Status::Pending {
            panic_with_error!(&env, Error::NotPending);
        }
        if env.ledger().timestamp() < correction.finalize_after {
            panic_with_error!(&env, Error::WindowNotElapsed);
        }

        let accepted = correction.approvals + correction.rejections >= cfg.min_votes
            && correction.approvals > correction.rejections;
        let tok = token::Client::new(&env, &cfg.token);

        if accepted {
            correction.status = Status::Accepted;
            tok.transfer(
                &env.current_contract_address(),
                &correction.contributor,
                &correction.stake,
            );
            let akey = DataKey::AcceptedCount(correction.contributor.clone());
            let accepted_count: u32 = s.get(&akey).unwrap_or(0);
            s.set(&akey, &(accepted_count + 1));
            bump(&env, &akey);
        } else {
            correction.status = Status::Rejected;
            tok.transfer(
                &env.current_contract_address(),
                &cfg.slash_recipient,
                &correction.stake,
            );
        }

        s.set(&ckey, &correction);
        bump(&env, &ckey);

        env.events()
            .publish((symbol_short!("final"),), (id, correction.status));
        correction.status
    }

    /// Fetch a single correction. Errors `BadId` if it does not exist.
    pub fn get(env: Env, id: u32) -> Correction {
        env.storage()
            .persistent()
            .get(&DataKey::Correction(id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::BadId))
    }

    /// Total number of corrections ever submitted.
    pub fn total(env: Env) -> u32 {
        env.storage().persistent().get(&DataKey::Count).unwrap_or(0)
    }

    /// The most recent `n` corrections, newest first.
    pub fn recent(env: Env, n: u32) -> Vec<Correction> {
        let s = env.storage().persistent();
        let count: u32 = s.get(&DataKey::Count).unwrap_or(0);
        let n = n.min(count);
        let mut out = Vec::new(&env);
        let mut i = 0u32;
        while i < n {
            let id = count - 1 - i;
            // Every id below count was written by submit; a miss is unreachable.
            let c: Correction = s
                .get(&DataKey::Correction(id))
                .unwrap_or_else(|| panic_with_error!(&env, Error::BadId));
            out.push_back(c);
            i += 1;
        }
        out
    }

    /// `(submitted, accepted)` counts for an address — simple reputation.
    pub fn reputation(env: Env, who: Address) -> (u32, u32) {
        let s = env.storage().persistent();
        (
            s.get(&DataKey::SubmittedCount(who.clone())).unwrap_or(0),
            s.get(&DataKey::AcceptedCount(who)).unwrap_or(0),
        )
    }
}
