#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, BytesN, Env, String,
};

const STAKE: i128 = 100;
const WINDOW: u64 = 3600;

struct Setup<'a> {
    client: RegistryClient<'a>,
    token: TokenClient<'a>,
    mint: StellarAssetClient<'a>,
    admin: Address,
    slash: Address,
}

fn setup(env: &Env) -> Setup<'_> {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let slash = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token = TokenClient::new(env, &sac.address());
    let mint = StellarAssetClient::new(env, &sac.address());
    let contract_id = env.register(Registry, ());
    let client = RegistryClient::new(env, &contract_id);
    client.init(&Config {
        admin: admin.clone(),
        token: sac.address(),
        stake_amount: STAKE,
        challenge_window: WINDOW,
        min_votes: 2,
        slash_recipient: slash.clone(),
    });
    Setup {
        client,
        token,
        mint,
        admin,
        slash,
    }
}

fn submit_one(env: &Env, s: &Setup, contributor: &Address) -> u32 {
    s.mint.mint(contributor, &1_000);
    s.client.submit(
        contributor,
        &String::from_str(env, "cms-oshodi"),
        &Kind::Fare,
        &BytesN::from_array(env, &[7u8; 32]),
        &String::from_str(env, "Fare is now 500 naira"),
    )
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn init_twice_panics() {
    let env = Env::default();
    let s = setup(&env);
    s.client.init(&Config {
        admin: s.admin.clone(),
        token: s.token.address.clone(),
        stake_amount: STAKE,
        challenge_window: WINDOW,
        min_votes: 2,
        slash_recipient: s.slash.clone(),
    });
}

#[test]
fn submit_stakes_and_records() {
    let env = Env::default();
    let s = setup(&env);
    let alice = Address::generate(&env);

    let id = submit_one(&env, &s, &alice);
    assert_eq!(id, 0);
    assert_eq!(s.token.balance(&alice), 1_000 - STAKE);
    assert_eq!(s.token.balance(&s.client.address), STAKE);
    assert_eq!(s.client.total(), 1);

    let c = s.client.get(&0);
    assert_eq!(c.contributor, alice);
    assert_eq!(c.kind, Kind::Fare);
    assert_eq!(c.status, Status::Pending);
    assert_eq!(c.stake, STAKE);
    assert_eq!(c.finalize_after, c.submitted_at + WINDOW);
    assert_eq!(c.approvals, 0);
    assert_eq!(s.client.reputation(&alice), (1, 0));
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn attest_self_vote_panics() {
    let env = Env::default();
    let s = setup(&env);
    let alice = Address::generate(&env);
    submit_one(&env, &s, &alice);
    s.client.attest(&alice, &0, &true);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn attest_double_vote_panics() {
    let env = Env::default();
    let s = setup(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    submit_one(&env, &s, &alice);
    s.client.attest(&bob, &0, &true);
    s.client.attest(&bob, &0, &false);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn attest_bad_id_panics() {
    let env = Env::default();
    let s = setup(&env);
    let bob = Address::generate(&env);
    s.client.attest(&bob, &99, &true);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn finalize_before_window_panics() {
    let env = Env::default();
    let s = setup(&env);
    let alice = Address::generate(&env);
    submit_one(&env, &s, &alice);
    s.client.finalize(&0);
}

#[test]
fn finalize_accepts_and_refunds() {
    let env = Env::default();
    let s = setup(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    submit_one(&env, &s, &alice);
    s.client.attest(&bob, &0, &true);
    s.client.attest(&carol, &0, &true);
    env.ledger().with_mut(|li| li.timestamp += WINDOW + 1);

    assert_eq!(s.client.finalize(&0), Status::Accepted);
    assert_eq!(s.token.balance(&alice), 1_000);
    assert_eq!(s.token.balance(&s.client.address), 0);
    assert_eq!(s.client.reputation(&alice), (1, 1));
    assert_eq!(s.client.get(&0).status, Status::Accepted);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn finalize_twice_panics() {
    let env = Env::default();
    let s = setup(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    submit_one(&env, &s, &alice);
    s.client.attest(&bob, &0, &true);
    s.client.attest(&carol, &0, &true);
    env.ledger().with_mut(|li| li.timestamp += WINDOW + 1);
    s.client.finalize(&0);
    s.client.finalize(&0);
}

#[test]
fn finalize_below_min_votes_slashes() {
    let env = Env::default();
    let s = setup(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    submit_one(&env, &s, &alice);
    s.client.attest(&bob, &0, &true); // only 1 vote, min is 2
    env.ledger().with_mut(|li| li.timestamp += WINDOW + 1);

    assert_eq!(s.client.finalize(&0), Status::Rejected);
    assert_eq!(s.token.balance(&alice), 1_000 - STAKE);
    assert_eq!(s.token.balance(&s.slash), STAKE);
    assert_eq!(s.client.reputation(&alice), (1, 0));
}

#[test]
fn finalize_tie_rejects() {
    let env = Env::default();
    let s = setup(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    submit_one(&env, &s, &alice);
    s.client.attest(&bob, &0, &true);
    s.client.attest(&carol, &0, &false);
    env.ledger().with_mut(|li| li.timestamp += WINDOW + 1);

    assert_eq!(s.client.finalize(&0), Status::Rejected);
    assert_eq!(s.token.balance(&s.slash), STAKE);
}

#[test]
fn recent_returns_newest_first() {
    let env = Env::default();
    let s = setup(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    submit_one(&env, &s, &alice);
    s.mint.mint(&bob, &1_000);
    s.client.submit(
        &bob,
        &String::from_str(&env, "yaba-ikeja"),
        &Kind::Closure,
        &BytesN::from_array(&env, &[9u8; 32]),
        &String::from_str(&env, "Road closed at Yaba"),
    );

    let recent = s.client.recent(&5);
    assert_eq!(recent.len(), 2);
    assert_eq!(recent.get(0).unwrap().contributor, bob);
    assert_eq!(recent.get(1).unwrap().contributor, alice);
}

#[test]
fn set_config_updates_slash_recipient() {
    let env = Env::default();
    let s = setup(&env);
    let new_slash = Address::generate(&env);
    s.client.set_config(&Config {
        admin: s.admin.clone(),
        token: s.token.address.clone(),
        stake_amount: STAKE,
        challenge_window: WINDOW,
        min_votes: 2,
        slash_recipient: new_slash.clone(),
    });

    let alice = Address::generate(&env);
    submit_one(&env, &s, &alice);
    env.ledger().with_mut(|li| li.timestamp += WINDOW + 1);
    assert_eq!(s.client.finalize(&0), Status::Rejected);
    assert_eq!(s.token.balance(&new_slash), STAKE);
}
