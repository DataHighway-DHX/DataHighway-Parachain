#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;

#[frame_support::pallet]
pub mod pallet {
	use sp_std::fmt::Debug;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, LockableCurrency, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	pub use sp_runtime::Percent;
	use sp_runtime::traits::{AtLeast32Bit, MaybeDisplay};
	use super::types::{
		AccountIdOf, BalanceOf, BlockNumberOf, RewardUnitOf, CrowdloanRewardFor, CrowdloanRewardParamFor, CrowdloanIdOf,
		RewardCampaignStatus,
	};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<AccountIdOf<Self>>
			+ ReservableCurrency<AccountIdOf<Self>>
			+ LockableCurrency<AccountIdOf<Self>>;

		// TODO:
		// do not restrict this to only u32
		type CrowdloanId: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Debug
			+ Default
			+ MaybeDisplay
			+ AtLeast32Bit
			+ Copy
			+ MaxEncodedLen;
	}

	#[pallet::storage]
	pub type Contribution<T> = StorageDoubleMap<_, Blake2_128Concat, CrowdloanIdOf<T>, Blake2_128Concat, AccountIdOf<T>, RewardUnitOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_reward_info)]
	pub type RewardInfo<T: Config> = StorageMap<_, Blake2_128Concat, CrowdloanIdOf<T>, CrowdloanRewardFor<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_campaign_status)]
	pub type CampaignStatus<T: Config> = StorageMap<_, Blake2_128Concat, CrowdloanIdOf<T>, RewardCampaignStatus>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new reward campaign with `CrowdloanId` started
		CampaignStarted(CrowdloanIdOf<T>),
		/// Campaign have been locked
		CampaignLocked(CrowdloanIdOf<T>),
		/// Campaign have been wiped
		CampaignWiped(CrowdloanIdOf<T>),
		/// Campaign details have been updated
		CampaignUpdated {
			crowdloan_id: CrowdloanIdOf<T>,
			previous_state: CrowdloanRewardFor<T>,
		},
		/// A contributer received instant amount of reward
		InstantRewarded {
			crowdloan_id: CrowdloanIdOf<T>,
			contributer: AccountIdOf<T>,
		},
		/// Vesting schdule of vesting amount have been applied to user
		VestingScheduleApplied {
			crowdloan_id: CrowdloanIdOf<T>,
			contributer: AccountIdOf<T>,
		},
		/// A contributer have been added as rewardee
		ContributedAdded {
			crowdloan_id: CrowdloanIdOf<T>,
			contributer: AccountIdOf<T>,
			total_amount: BalanceOf<T>,
		},
		/// A contributer have been removed from campaign
		ContributedKicked {
			crowdloan_id: CrowdloanIdOf<T>,
			contributer: AccountIdOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// This tasj requires more permission from the origin
		PermissionDenied,
		/// No such reward campaign registered
		NoRewardCampaign,
		/// Reward campaign already exists
		RewardCampaignExists,
		/// This user have made no contribution
		NoContribution,
		/// This contributer already exists
		ContributerExists,
		/// Campaign have been locked
		CampaignLocked,
		/// Campaign is in progressed
		CampaignInProgress,
		/// Campaign wiped
		CampaignWiped,
		/// Not all required information was passed
		InsufficientInfo,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
		pub fn start_new_crowdloan(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>, info: CrowdloanRewardParamFor<T>) -> DispatchResult {
			let hoster = ensure_signed(origin)?;

			ensure!(!<CampaignStatus<T>>::contains_key(&crowdloan_id), <Error<T>>::RewardCampaignExists);
			ensure!(!<RewardInfo<T>>::contains_key(&crowdloan_id), <Error<T>>::RewardCampaignExists);
			ensure!(<Contribution<T>>::iter_key_prefix(&crowdloan_id).count() == 0, <Error<T>>::RewardCampaignExists);

			let reward_source = info.reward_source.ok_or(<Error<T>>::InsufficientInfo)?;
			let total_pool = info.total_pool.ok_or(<Error<T>>::InsufficientInfo)?;
			let end_target = info.end_target.ok_or(<Error<T>>::InsufficientInfo)?;
			let starts_from = info.starts_from.ok_or(<Error<T>>::InsufficientInfo)?;
			let instant_percentage = info.instant_percentage.ok_or(<Error<T>>::InsufficientInfo)?;
			let vesting_percentage = info.vesting_percentage.ok_or(<Error<T>>::InsufficientInfo)?;

			let crowdloan_reward_info = CrowdloanRewardFor::<T> {
				hoster,
				reward_source,
				total_pool,
				end_target,
				starts_from,
				instant_percentage,
				vesting_percentage,
			};

			<CampaignStatus<T>>::insert(crowdloan_id, RewardCampaignStatus::InProgress);
			<RewardInfo<T>>::insert(crowdloan_id, crowdloan_reward_info);

			Self::deposit_event(Event::<T>::CampaignStarted(crowdloan_id));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn add_contributer(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>, contributer: AccountIdOf<T>, amount: BalanceOf<T>) -> DispatchResult {
			Self::ensure_hoster(origin, crowdloan_id)?;

			// TODO:
			// - check state is writeable
			// - check the contributer is not already there
			// - calculate required fields from given amount
			// - put vlaue

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn remove_contributer(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>, contributer: AccountIdOf<T>) -> DispatchResult {
			Self::ensure_hoster(origin, crowdloan_id)?;

			// TODO:
			// - check state is writable
			// - remove contributer

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn lock_campaign(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>) -> DispatchResult {
			Self::ensure_hoster(origin, crowdloan_id)?;

			// TODO
			// - check current state is lockable
			// - check the source have enough locked balance ( the locked field ) to reward all contributers
			// - change the status

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn wipe_crowdloan_campaign(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>) -> DispatchResult {
			Self::ensure_hoster(origin, crowdloan_id)?;

			// TODO:
			// - check state is wipeable
			// - check if all receiver have received the reward
			// - kill Contribution storage mapped to this id
			// - kill RewardInfo storage under this id
			// - update the CampaignStatus to wiped

			Ok(())
		}

		#[pallet::weight(10_0000)]
		pub fn get_instant_reward(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>) -> DispatchResult {
			let rewardee = ensure_signed(origin)?;

			// TODO:
			// - check this reward campaign is in locked stage
			// - check this call not have been made already
			// - Reward the instant amount of this rewardee
			// - update the status

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn get_vested_reward(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>) -> DispatchResult {
			let rewardee = ensure_signed(origin)?;

			// TODO:
			// - check this reward campaign is in locked stage
			// - check this call have not been made already
			// - make vesting schedule with locked amont = vesting reward
			// - update status

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn ensure_hoster(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>) -> DispatchResult {
			let claimer = ensure_signed(origin)?;
			Self::verify_crowdloan_hoster(&claimer, &crowdloan_id)
		}

		fn verify_crowdloan_hoster(signer: &AccountIdOf<T>, crowdloan_id: &CrowdloanIdOf<T>) -> DispatchResult {
			(&Self::get_reward_info(crowdloan_id)
				.ok_or(<Error<T>>::NoRewardCampaign)?
				.hoster == signer)
				.then_some(())
				.ok_or(<Error<T>>::PermissionDenied.into())
		}

		fn get_current_block_number() -> BlockNumberOf<T> {
			<frame_system::Pallet<T>>::block_number()
		}
	}
}
