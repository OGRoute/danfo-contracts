//! Storage types, config, and errors for danfo-registry.

use soroban_sdk::{contracterror, contracttype, Address, BytesN, String};

/// ~30 days / ~90 days in ledgers (one ledger ≈ 5s, 17280/day).
pub const TTL_THRESHOLD: u32 = 30 * 17280;
pub const TTL_EXTEND: u32 = 90 * 17280;

/// What a correction changes in the knowledge base.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Fare = 0,
    Route = 1,
    Closure = 2,
}

/// Lifecycle state of a correction.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Pending = 0,
    Accepted = 1,
    Rejected = 2,
}

/// A community-submitted correction to the route knowledge base.
#[contracttype]
#[derive(Clone)]
pub struct Correction {
    pub contributor: Address,
    /// Route slug, e.g. "cms-oshodi".
    pub route_id: String,
    pub kind: Kind,
    /// sha256 of the off-chain JSON payload this correction anchors.
    pub payload_hash: BytesN<32>,
    /// Short human-readable summary (length enforced off-chain).
    pub summary: String,
    /// Stake locked at submission, in the config token's base units.
    pub stake: i128,
    pub status: Status,
    pub submitted_at: u64,
    /// `submitted_at + challenge_window`; finalize is callable after this.
    pub finalize_after: u64,
    pub approvals: u32,
    pub rejections: u32,
}

/// Contract configuration, held in instance storage.
#[contracttype]
#[derive(Clone)]
pub struct Config {
    pub admin: Address,
    /// SAC token used for stakes (XLM SAC on testnet, NGNC as an option).
    pub token: Address,
    pub stake_amount: i128,
    /// Challenge window in seconds.
    pub challenge_window: u64,
    /// Minimum total attestations for a correction to be acceptable.
    pub min_votes: u32,
    /// Where rejected stakes go (pointed at the rewards pool after deploy).
    pub slash_recipient: Address,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Config,
    Count,
    Correction(u32),
    Voted(u32, Address),
    AcceptedCount(Address),
    SubmittedCount(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    BadId = 3,
    AlreadyVoted = 4,
    NotPending = 5,
    SelfVote = 6,
    WindowNotElapsed = 7,
    NotAdmin = 8,
}
