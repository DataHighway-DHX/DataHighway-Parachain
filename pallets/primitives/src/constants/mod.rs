#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::Perquintill;
use frame_support::parameter_types;
use crate::types::{Balance, BlockNumber};

/// Money matters.
pub mod currency {
    use super::Balance;

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

pub mod aura {
    pub const MAX_AUTHORITIES: u32 = 100;
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
    pub const YEAR: BlockNumber = DAYS * 365;

    // 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 1 * HOURS;
    pub const EPOCH_DURATION_IN_SLOTS: u64 = {
        const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

        (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
    };
}

pub mod staking {
	use super::*;

	parameter_types! {
		/// Minimum round length is 1 hour
		pub const MinBlocksPerRound: BlockNumber = 1 * time::MINUTES;
		/// Default length of a round/session is 2 hours
		pub const DefaultBlocksPerRound: BlockNumber = 1 * time::MINUTES;
		/// Unstaked balance can be unlocked after 7 days
		pub const StakeDuration: BlockNumber = 1 * time::MINUTES;
		/// Collator exit requests are delayed by 4 hours (2 rounds/sessions)
		pub const ExitQueueDelay: u32 = 2;
		/// Minimum 5 collators selected per round, default at genesis and minimum forever after
		pub const MinCollators: u32 = 1;
		/// At least 4 candidates which cannot leave the network if there are no other candidates.
		pub const MinRequiredCollators: u32 = 1;
		/// We only allow one delegation per round.
		pub const MaxDelegationsPerRound: u32 = 1;
		/// Maximum 35 delegators per collator at launch, might be increased later
		#[derive(Debug, Eq, PartialEq)]
		pub const MaxDelegatorsPerCollator: u32 = 35;
		/// Maximum 1 collator per delegator at launch, will be increased later
		#[derive(Debug, Eq, PartialEq)]
		pub const MaxCollatorsPerDelegator: u32 = 1;
		/// Minimum stake required to be reserved to be a collator is 10_000
		pub const MinCollatorStake: Balance = 5 * currency::DOLLARS;
		/// Minimum stake required to be reserved to be a delegator is 100
		pub const MinDelegatorStake: Balance = 5 * currency::DOLLARS;
		/// Maximum number of collator candidates
		#[derive(Debug, Eq, PartialEq)]
		pub const MaxCollatorCandidates: u32 = aura::MAX_AUTHORITIES;
		/// Maximum number of concurrent requests to unlock unstaked balance
		pub const MaxUnstakeRequests: u32 = 10;
		/// The starting block number for the network rewards
		pub const NetworkRewardStart: BlockNumber = 1; // somewhere is august 2022
		/// The rate in percent for the network rewards
		pub const NetworkRewardRate: Perquintill = Perquintill::from_percent(50);
	}

    pub const MAX_CANDIDATE_STAKE: Balance = 100 * currency::DOLLARS;

    pub fn dhx_inflation() -> parachain_staking::InflationInfo {
        parachain_staking::InflationInfo::new(
            time::YEAR as u64,
            // max collator staking rate
            Perquintill::from_percent(40),
            // collator reward rate
            Perquintill::from_percent(20),
            // max delegator staking rate
            Perquintill::from_percent(10),
            // delegator reward rate
            Perquintill::from_percent(20),
        )
    }
}

