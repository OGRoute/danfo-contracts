#![no_std]
//! danfo-rewards — sponsor-funded reward pool paying a fixed amount per
//! accepted correction in danfo-registry. Claims are pull-based and
//! idempotent per correction id; anyone may crank `claim`, the payout only
//! ever goes to the correction's recorded contributor.

mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, token,
    Address, BytesN, Env,
};

/// Interface to danfo-registry, generated from its built wasm.
/// Build order: `cargo build -p danfo-registry --release --target
/// wasm32-unknown-unknown` must run before this crate compiles.
pub mod registry {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/danfo_registry.wasm"
    );
}

/// ~30 days / ~90 days in ledgers (one ledger ≈ 5s, 17280/day).
const TTL_THRESHOLD: u32 = 30 * 17280;
const TTL_EXTEND: u32 = 90 * 17280;

/// Contract configuration, held in instance storage.
#[contracttype]
#[derive(Clone)]
pub struct RewardsConfig {
    pub admin: Address,
    /// SAC token the pool holds — must match the registry's stake token.
    pub token: Address,
    /// danfo-registry contract id.
    pub registry: Address,
    /// Payout per accepted correction, in token base units.
    pub reward_amount: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Config,
    Claimed(u32),
    TotalPaid,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAccepted = 3,
    AlreadyClaimed = 4,
    InsufficientPool = 5,
    /// `reward_amount` is not positive.
    InvalidConfig = 6,
    /// A funding amount is not positive.
    InvalidAmount = 7,
}

#[contract]
pub struct Rewards;

/// Read config or panic `NotInitialized`.
fn config(env: &Env) -> RewardsConfig {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .unwrap_or_else(|| panic_with_error!(env, Error::NotInitialized))
}

#[contractimpl]
impl Rewards {
    /// One-shot initialization. Auth: the incoming `admin`, so a deploy
    /// cannot be front-run into someone else's control. Errors
    /// `AlreadyInitialized` on a second call, `InvalidConfig` on a
    /// non-positive `reward_amount`.
    pub fn init(env: Env, config: RewardsConfig) {
        if env.storage().instance().has(&DataKey::Config) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        config.admin.require_auth();
        if config.reward_amount <= 0 {
            panic_with_error!(&env, Error::InvalidConfig);
        }
        env.storage().instance().set(&DataKey::Config, &config);
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
        env.events().publish((symbol_short!("init"),), config.admin);
    }

    /// Add funds to the reward pool. Auth: `sponsor`.
    /// Errors `InvalidAmount` on a non-positive amount — a negative transfer
    /// would otherwise read as a withdrawal request.
    pub fn fund(env: Env, sponsor: Address, amount: i128) {
        sponsor.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, Error::InvalidAmount);
        }
        let cfg = config(&env);
        token::Client::new(&env, &cfg.token).transfer(
            &sponsor,
            &env.current_contract_address(),
            &amount,
        );
        env.events()
            .publish((symbol_short!("fund"), sponsor), amount);
    }

    /// Replace this contract's wasm. Auth: admin. The migration path off
    /// unaudited testnet code without moving the pool's contract id.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        config(&env).admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        env.events().publish((symbol_short!("upgrade"),), ());
    }

    /// Change the per-correction payout. Auth: admin.
    /// Errors `InvalidConfig` on a non-positive amount.
    pub fn set_reward(env: Env, amount: i128) {
        let mut cfg = config(&env);
        cfg.admin.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, Error::InvalidConfig);
        }
        cfg.reward_amount = amount;
        env.storage().instance().set(&DataKey::Config, &cfg);
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
        env.events().publish((symbol_short!("config"),), amount);
    }

    /// Pay the reward for an accepted correction to its contributor.
    /// No auth — anyone may crank this. Idempotent per correction id.
    pub fn claim(env: Env, id: u32) {
        let cfg = config(&env);

        let correction = registry::Client::new(&env, &cfg.registry).get(&id);
        if correction.status != registry::Status::Accepted {
            panic_with_error!(&env, Error::NotAccepted);
        }

        let ckey = DataKey::Claimed(id);
        let s = env.storage().persistent();
        if s.get::<_, bool>(&ckey).unwrap_or(false) {
            panic_with_error!(&env, Error::AlreadyClaimed);
        }

        let tok = token::Client::new(&env, &cfg.token);
        if tok.balance(&env.current_contract_address()) < cfg.reward_amount {
            panic_with_error!(&env, Error::InsufficientPool);
        }

        s.set(&ckey, &true);
        s.extend_ttl(&ckey, TTL_THRESHOLD, TTL_EXTEND);
        let paid: i128 = s.get(&DataKey::TotalPaid).unwrap_or(0);
        s.set(&DataKey::TotalPaid, &(paid + cfg.reward_amount));
        s.extend_ttl(&DataKey::TotalPaid, TTL_THRESHOLD, TTL_EXTEND);

        tok.transfer(
            &env.current_contract_address(),
            &correction.contributor,
            &cfg.reward_amount,
        );

        env.events().publish(
            (symbol_short!("claim"),),
            (id, correction.contributor, cfg.reward_amount),
        );
    }

    /// The active configuration, so clients can read the live reward and
    /// linked registry instead of hardcoding launch defaults.
    pub fn get_config(env: Env) -> RewardsConfig {
        config(&env)
    }

    /// Current pool balance.
    pub fn pool(env: Env) -> i128 {
        let cfg = config(&env);
        token::Client::new(&env, &cfg.token).balance(&env.current_contract_address())
    }

    /// Whether a correction id has already been paid.
    pub fn is_claimed(env: Env, id: u32) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Claimed(id))
            .unwrap_or(false)
    }

    /// Total rewards ever paid out.
    pub fn total_paid(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TotalPaid)
            .unwrap_or(0)
    }
}
