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
use types::{
    MAX_CHALLENGE_WINDOW, MAX_PAGE, MAX_ROUTE_ID_LEN, MAX_SUMMARY_LEN, TTL_EXTEND, TTL_THRESHOLD,
};

#[contract]
pub struct Registry;

/// Read config or panic `NotInitialized`.
fn config(env: &Env) -> Config {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .unwrap_or_else(|| panic_with_error!(env, Error::NotInitialized))
}

/// Reject a config that would make the lifecycle unsound: a non-positive
/// stake (nothing at risk), zero `min_votes` (acceptance without review), or
/// a window that is zero or long enough to strand stakes.
fn validate(env: &Env, cfg: &Config) {
    if cfg.stake_amount <= 0
        || cfg.min_votes == 0
        || cfg.challenge_window == 0
        || cfg.challenge_window > MAX_CHALLENGE_WINDOW
    {
        panic_with_error!(env, Error::InvalidConfig);
    }
}

/// Extend the TTL of a persistent key after writing it.
fn bump(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, TTL_THRESHOLD, TTL_EXTEND);
}

/// Load correction `id` or panic `BadId`.
fn load(env: &Env, id: u32) -> Correction {
    env.storage()
        .persistent()
        .get(&DataKey::Correction(id))
        .unwrap_or_else(|| panic_with_error!(env, Error::BadId))
}

#[contractimpl]
impl Registry {
    /// One-shot initialization. Auth: the incoming `admin`, so a deploy
    /// cannot be front-run into someone else's control. Errors
    /// `AlreadyInitialized` on a second call, `InvalidConfig` on bad fields.
    pub fn init(env: Env, config: Config) {
        if env.storage().instance().has(&DataKey::Config) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        config.admin.require_auth();
        validate(&env, &config);
        env.storage().instance().set(&DataKey::Config, &config);
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
        env.events().publish((symbol_short!("init"),), config.admin);
    }

    /// Replace the configuration. Auth: current admin.
    /// Used after deploy to point `slash_recipient` at the rewards pool.
    /// `token` cannot change — pending stakes are denominated in it, so a
    /// swap would refund or slash in an asset the contract never received.
    pub fn set_config(env: Env, new_config: Config) {
        let current = config(&env);
        current.admin.require_auth();
        if new_config.token != current.token {
            panic_with_error!(&env, Error::TokenImmutable);
        }
        validate(&env, &new_config);
        env.storage().instance().set(&DataKey::Config, &new_config);
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
        env.events()
            .publish((symbol_short!("config"),), new_config.admin);
    }

    /// Replace this contract's wasm. Auth: admin. The migration path off
    /// unaudited testnet code without moving contract ids or stake custody.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        config(&env).admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        env.events().publish((symbol_short!("upgrade"),), ());
    }

    /// Submit a correction, locking `stake_amount` of the config token.
    /// Auth: `contributor`. Returns the new 0-based correction id.
    /// Errors `InvalidInput` if `route_id` is empty or either string is over
    /// its length cap.
    pub fn submit(
        env: Env,
        contributor: Address,
        route_id: String,
        kind: Kind,
        payload_hash: BytesN<32>,
        summary: String,
    ) -> u32 {
        contributor.require_auth();
        if route_id.is_empty()
            || route_id.len() > MAX_ROUTE_ID_LEN
            || summary.is_empty()
            || summary.len() > MAX_SUMMARY_LEN
        {
            panic_with_error!(&env, Error::InvalidInput);
        }
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

        env.events()
            .publish((symbol_short!("submit"), contributor), (id, route_id, kind));
        id
    }

    /// Approve or reject a pending correction during its challenge window.
    /// Auth: `voter`. One vote per address; contributors cannot vote on
    /// their own corrections.
    pub fn attest(env: Env, voter: Address, id: u32, approve: bool) {
        voter.require_auth();
        let s = env.storage().persistent();

        let ckey = DataKey::Correction(id);
        let mut correction = load(&env, id);
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
        let mut correction = load(&env, id);
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
        load(&env, id)
    }

    /// The active configuration, so clients can read the live stake, window
    /// and quorum instead of hardcoding launch defaults.
    pub fn get_config(env: Env) -> Config {
        config(&env)
    }

    /// Total number of corrections ever submitted.
    pub fn total(env: Env) -> u32 {
        env.storage().persistent().get(&DataKey::Count).unwrap_or(0)
    }

    /// The most recent `n` corrections, newest first. `n` is clamped to
    /// `MAX_PAGE` so a read is never sized by untrusted input.
    pub fn recent(env: Env, n: u32) -> Vec<Correction> {
        let count = Self::total(env.clone());
        let n = n.min(MAX_PAGE).min(count);
        let mut out = Vec::new(&env);
        let mut i = 0u32;
        while i < n {
            // Every id below count was written by submit; a miss is unreachable.
            out.push_back(load(&env, count - 1 - i));
            i += 1;
        }
        out
    }

    /// A window of corrections in submission order, starting at `start`.
    /// `limit` is clamped to `MAX_PAGE`. Returns empty past the end, so an
    /// indexer can page forward from its last cursor without bounds checks.
    pub fn page(env: Env, start: u32, limit: u32) -> Vec<Correction> {
        let count = Self::total(env.clone());
        let mut out = Vec::new(&env);
        if start >= count {
            return out;
        }
        let end = start.saturating_add(limit.min(MAX_PAGE)).min(count);
        let mut id = start;
        while id < end {
            out.push_back(load(&env, id));
            id += 1;
        }
        out
    }

    /// Whether `voter` has already attested to correction `id`.
    pub fn has_voted(env: Env, id: u32, voter: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Voted(id, voter))
            .unwrap_or(false)
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
