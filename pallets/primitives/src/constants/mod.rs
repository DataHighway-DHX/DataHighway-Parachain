#![cfg_attr(not(feature = "std"), no_std)]

/// Money matters.
pub mod currency {
    use crate::types::Balance;

    // this should match tokenDecimals defined in the chain_spec.rs
    pub const UNITS: Balance = 1_000_000_000_000_000_000;
    pub const DOLLARS: Balance = UNITS;
    pub const CENTS: Balance = DOLLARS / 100;
    pub const MILLICENTS: Balance = CENTS / 1_000;

    pub const MILLIUNITS: Balance = UNITS / 1_000;
    pub const MICROUNITS: Balance = MILLIUNITS / 1_000;

    /// The existential deposit. Set to 1/10 of the Connected Relay Chain.
    // Note: Kusama relay chain's `ExistentialDeposit` is 1 * CENTS,
    // so 1/10 of that is 100 millicents
    pub const EXISTENTIAL_DEPOSIT: Balance = 100 * MILLICENTS;

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
    }
}

/// Time.
pub mod time {
    pub use crate::types::{
        BlockNumber,
        Moment,
    };

    // Note: On Standalone chain we used 4320, but we must use the
    // substrate-parachain-template default value for the Parachain
    pub const MILLISECS_PER_BLOCK: Moment = 12000;
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;

    // 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 1 * HOURS;
    pub const EPOCH_DURATION_IN_SLOTS: u64 = {
        const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

        (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
    };
}
