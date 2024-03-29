//! Autogenerated weights for pallet_reward_campaign
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-14, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `dhx-temp`, CPU: `Intel(R) Xeon(R) CPU E5-2686 v4 @ 2.30GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// target/release/datahighway-collator
// benchmark
// pallet
// --chain
// dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// pallet_crowdloan_reward
// --steps
// 50
// --repeat
// 20
// --output
// pallets/crowdloan-reward/src/weights.rs
// --template
// .maintain/pallet_weight.hbs
// --extrinsic
// *

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{constants::RocksDbWeight, Weight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for `pallet_reward_campaign`.
pub trait WeightInfo {
	fn start_new_campaign() -> Weight;
	fn update_campaign() -> Weight;
	fn add_contributor() -> Weight;
	fn remove_contributor() -> Weight;
	fn get_instant_reward() -> Weight;
	fn get_vested_reward() -> Weight;
	fn lock_campaign() -> Weight;
	fn wipe_campaign() -> Weight;
	fn discard_campaign() -> Weight;
}

/// Weight functions for `pallet_reward_campaign`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Reward CampaignStatus (r:1 w:1)
	// Storage: Reward RewardInfo (r:1 w:1)
	// Storage: Reward Contribution (r:1 w:0)
	fn start_new_campaign() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(69_841_000_u64)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:1)
	// Storage: Reward CampaignStatus (r:1 w:0)
	// Storage: Reward Contribution (r:1 w:0)
	fn update_campaign() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(74_674_000_u64)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:0)
	// Storage: Reward CampaignStatus (r:1 w:0)
	// Storage: Reward Contribution (r:1 w:1)
	fn add_contributor() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(72_551_000_u64)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:0)
	// Storage: Reward CampaignStatus (r:1 w:0)
	// Storage: Reward Contribution (r:1 w:1)
	fn remove_contributor() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(71_797_000_u64)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: Reward CampaignStatus (r:1 w:0)
	// Storage: Reward Contribution (r:1 w:1)
	// Storage: Reward RewardInfo (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	fn get_instant_reward() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(144_324_000_u64)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	// Storage: Reward CampaignStatus (r:1 w:0)
	// Storage: Reward Contribution (r:1 w:1)
	// Storage: Reward RewardInfo (r:1 w:0)
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Balances Locks (r:1 w:1)
	fn get_vested_reward() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(209_266_000_u64)
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:0)
	// Storage: Reward CampaignStatus (r:1 w:1)
	fn lock_campaign() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(67_080_000_u64)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:1)
	// Storage: Reward CampaignStatus (r:1 w:1)
	// Storage: Reward Contribution (r:11 w:10)
	fn wipe_campaign() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(253_794_000_u64)
			.saturating_add(T::DbWeight::get().reads(13_u64))
			.saturating_add(T::DbWeight::get().writes(12_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:1)
	// Storage: Reward CampaignStatus (r:1 w:1)
	// Storage: Reward Contribution (r:0 w:10)
	fn discard_campaign() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(102_939_000_u64)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(12_u64))
	}
}

impl WeightInfo for () {
	// Storage: Reward CampaignStatus (r:1 w:1)
	// Storage: Reward RewardInfo (r:1 w:1)
	// Storage: Reward Contribution (r:1 w:0)
	fn start_new_campaign() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(69_841_000_u64)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:1)
	// Storage: Reward CampaignStatus (r:1 w:0)
	// Storage: Reward Contribution (r:1 w:0)
	fn update_campaign() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(74_674_000_u64)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:0)
	// Storage: Reward CampaignStatus (r:1 w:0)
	// Storage: Reward Contribution (r:1 w:1)
	fn add_contributor() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(72_551_000_u64)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:0)
	// Storage: Reward CampaignStatus (r:1 w:0)
	// Storage: Reward Contribution (r:1 w:1)
	fn remove_contributor() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(71_797_000_u64)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: Reward CampaignStatus (r:1 w:0)
	// Storage: Reward Contribution (r:1 w:1)
	// Storage: Reward RewardInfo (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	fn get_instant_reward() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(144_324_000_u64)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	// Storage: Reward CampaignStatus (r:1 w:0)
	// Storage: Reward Contribution (r:1 w:1)
	// Storage: Reward RewardInfo (r:1 w:0)
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Balances Locks (r:1 w:1)
	fn get_vested_reward() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(209_266_000_u64)
			.saturating_add(RocksDbWeight::get().reads(7_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:0)
	// Storage: Reward CampaignStatus (r:1 w:1)
	fn lock_campaign() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(67_080_000_u64)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:1)
	// Storage: Reward CampaignStatus (r:1 w:1)
	// Storage: Reward Contribution (r:11 w:10)
	fn wipe_campaign() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(253_794_000_u64)
			.saturating_add(RocksDbWeight::get().reads(13_u64))
			.saturating_add(RocksDbWeight::get().writes(12_u64))
	}
	// Storage: Reward RewardInfo (r:1 w:1)
	// Storage: Reward CampaignStatus (r:1 w:1)
	// Storage: Reward Contribution (r:0 w:10)
	fn discard_campaign() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(102_939_000_u64)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(12_u64))
	}
}