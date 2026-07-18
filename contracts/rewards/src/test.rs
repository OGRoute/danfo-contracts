#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, BytesN, Env, String,
};

const STAKE: i128 = 100;
const REWARD: i128 = 50;
const WINDOW: u64 = 3600;

struct Setup<'a> {
    rewards: RewardsClient<'a>,
    registry: registry::Client<'a>,
    token: TokenClient<'a>,
    mint: StellarAssetClient<'a>,
    sponsor: Address,
    alice: Address,
}

fn setup(env: &Env) -> Setup<'_> {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let sponsor = Address::generate(env);
    let alice = Address::generate(env);

    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token = TokenClient::new(env, &sac.address());
    let mint = StellarAssetClient::new(env, &sac.address());

    let registry_id = env.register(registry::WASM, ());
    let registry = registry::Client::new(env, &registry_id);

    let rewards_id = env.register(Rewards, ());
    let rewards = RewardsClient::new(env, &rewards_id);

    registry.init(&registry::Config {
        admin: admin.clone(),
        token: sac.address(),
        stake_amount: STAKE,
        challenge_window: WINDOW,
        min_votes: 2,
        slash_recipient: rewards_id.clone(),
    });
    rewards.init(&RewardsConfig {
        admin: admin.clone(),
        token: sac.address(),
        registry: registry_id,
        reward_amount: REWARD,
    });

    mint.mint(&sponsor, &10_000);
    Setup {
        rewards,
        registry,
        token,
        mint,
        sponsor,
        alice,
    }
}

/// Submit correction id 0 from alice and drive it to Accepted.
fn accept_one(env: &Env, s: &Setup) {
    s.mint.mint(&s.alice, &1_000);
    s.registry.submit(
        &s.alice,
        &String::from_str(env, "cms-oshodi"),
        &registry::Kind::Fare,
        &BytesN::from_array(env, &[7u8; 32]),
        &String::from_str(env, "Fare is now 500 naira"),
    );
    let bob = Address::generate(env);
    let carol = Address::generate(env);
    s.registry.attest(&bob, &0, &true);
    s.registry.attest(&carol, &0, &true);
    env.ledger().with_mut(|li| li.timestamp += WINDOW + 1);
    s.registry.finalize(&0);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn init_twice_panics() {
    let env = Env::default();
    let s = setup(&env);
    s.rewards.init(&RewardsConfig {
        admin: s.sponsor.clone(),
        token: s.token.address.clone(),
        registry: s.registry.address.clone(),
        reward_amount: REWARD,
    });
}

#[test]
fn fund_increases_pool() {
    let env = Env::default();
    let s = setup(&env);
    assert_eq!(s.rewards.pool(), 0);
    s.rewards.fund(&s.sponsor, &500);
    assert_eq!(s.rewards.pool(), 500);
    assert_eq!(s.token.balance(&s.sponsor), 9_500);
}

#[test]
fn claim_pays_contributor_once() {
    let env = Env::default();
    let s = setup(&env);
    s.rewards.fund(&s.sponsor, &500);
    accept_one(&env, &s);

    let before = s.token.balance(&s.alice);
    s.rewards.claim(&0);
    assert_eq!(s.token.balance(&s.alice), before + REWARD);
    assert!(s.rewards.is_claimed(&0));
    assert_eq!(s.rewards.total_paid(), REWARD);
    assert_eq!(s.rewards.pool(), 500 - REWARD);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn claim_pending_panics() {
    let env = Env::default();
    let s = setup(&env);
    s.rewards.fund(&s.sponsor, &500);
    s.mint.mint(&s.alice, &1_000);
    s.registry.submit(
        &s.alice,
        &String::from_str(&env, "cms-oshodi"),
        &registry::Kind::Fare,
        &BytesN::from_array(&env, &[7u8; 32]),
        &String::from_str(&env, "Fare is now 500 naira"),
    );
    s.rewards.claim(&0);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn claim_twice_panics() {
    let env = Env::default();
    let s = setup(&env);
    s.rewards.fund(&s.sponsor, &500);
    accept_one(&env, &s);
    s.rewards.claim(&0);
    s.rewards.claim(&0);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn claim_empty_pool_panics() {
    let env = Env::default();
    let s = setup(&env);
    accept_one(&env, &s);
    s.rewards.claim(&0);
}

#[test]
fn rejected_stake_slashes_into_pool() {
    let env = Env::default();
    let s = setup(&env);
    s.mint.mint(&s.alice, &1_000);
    s.registry.submit(
        &s.alice,
        &String::from_str(&env, "cms-oshodi"),
        &registry::Kind::Fare,
        &BytesN::from_array(&env, &[7u8; 32]),
        &String::from_str(&env, "Spam"),
    );
    env.ledger().with_mut(|li| li.timestamp += WINDOW + 1);
    s.registry.finalize(&0); // no votes -> rejected, stake slashed to rewards
    assert_eq!(s.rewards.pool(), STAKE);
}
