#![no_std]
//! danfo-rewards — sponsor-funded reward pool paying a fixed amount per
//! accepted correction in danfo-registry. Claims are pull-based and
//! idempotent per correction id.

use soroban_sdk::contract;

#[contract]
pub struct Rewards;
