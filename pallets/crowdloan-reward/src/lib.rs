#![cfg_attr(not(feature = "std"), no_std)]
#![feature(result_flattening)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod functions;
mod types;

#[frame_support::pallet]
pub mod pallet {
    use super::{
        functions,
        types,
    };
    use frame_support::{
        pallet_prelude::{
            DispatchResult,
            *,
        },
        traits::{
            Currency,
            LockableCurrency,
            ReservableCurrency,
        },
    };
    use frame_system::pallet_prelude::*;
    pub use sp_runtime::Percent;
    use sp_runtime::{
        traits::{
            AtLeast32Bit,
            Convert,
            MaybeDisplay,
        },
    };
    use sp_std::fmt::Debug;
    use types::{
        AccountIdOf,
        BalanceOf,
        BlockNumberOf,
        ClaimerStatus,
        CrowdloanIdOf,
        CrowdloanRewardFor,
        CrowdloanRewardParamFor,
        InstantEnsuredResultOf,
        RewardCampaignStatus,
        RewardUnitOf,
        VestedEnsuredResultOf,
        VestingBalanceOf,
    };

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_vesting::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: Currency<AccountIdOf<Self>>
            + ReservableCurrency<AccountIdOf<Self>>
            + LockableCurrency<AccountIdOf<Self>>
            + IsType<VestingBalanceOf<Self>>;

        type CrowdloanId: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + Default
            + MaybeDisplay
            + AtLeast32Bit
            + Copy
            + MaxEncodedLen;

        type BlockNumberToBalance: Convert<BlockNumberOf<Self>, BalanceOf<Self>>;
        type CurrencyConvert: Convert<BalanceOf<Self>, VestingBalanceOf<Self>>
            + Convert<VestingBalanceOf<Self>, BalanceOf<Self>>;
    }

    #[pallet::storage]
    #[pallet::getter(fn get_contribution)]
    pub type Contribution<T> =
        StorageDoubleMap<_, Blake2_128Concat, CrowdloanIdOf<T>, Blake2_128Concat, AccountIdOf<T>, RewardUnitOf<T>>;

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
        ContributerAdded {
            crowdloan_id: CrowdloanIdOf<T>,
            contributer: AccountIdOf<T>,
            amount: BalanceOf<T>,
        },
        /// A contributer have been removed from campaign
        ContributerKicked {
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
        /// This crowdloan is in one of read-only state
        ReadOnlyCampaign,
        /// This campaign is not yet in claimable state for contributers
        CampaignNotLocked,
        /// This campaign cannot be locked
        NonLockableCampaign,
        /// Instant reward have been taken by this contributer
        InstantRewardTaken,
        /// Vesting scheduled have been applied to this contributer's address
        VestingRewardApplied,
        /// Some error occured while splitting total amount
        CanotSplitAmount,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn start_new_crowdloan(
            origin: OriginFor<T>,
            crowdloan_id: CrowdloanIdOf<T>,
            info: CrowdloanRewardParamFor<T>,
        ) -> DispatchResult {
            let hoster = ensure_signed(origin)?;

            ensure!(!<CampaignStatus<T>>::contains_key(&crowdloan_id), <Error<T>>::RewardCampaignExists);
            ensure!(!<RewardInfo<T>>::contains_key(&crowdloan_id), <Error<T>>::RewardCampaignExists);
            ensure!(<Contribution<T>>::iter_key_prefix(&crowdloan_id).count() == 0, <Error<T>>::RewardCampaignExists);

            let reward_source = info.reward_source.ok_or(<Error<T>>::InsufficientInfo)?;
            let total_pool = info.total_pool.ok_or(<Error<T>>::InsufficientInfo)?;
            let end_target = info.end_target.ok_or(<Error<T>>::InsufficientInfo)?;
            let instant_percentage = info.instant_percentage.ok_or(<Error<T>>::InsufficientInfo)?;
            let starts_from = info.starts_from.unwrap_or_else(Self::get_current_block_number);
            let hoster = info.hoster.unwrap_or(hoster);

            let crowdloan_reward_info = CrowdloanRewardFor::<T> {
                hoster,
                reward_source,
                total_pool,
                end_target,
                starts_from,
                instant_percentage,
            };

            <CampaignStatus<T>>::insert(crowdloan_id, RewardCampaignStatus::InProgress);
            <RewardInfo<T>>::insert(crowdloan_id, crowdloan_reward_info);

            Self::deposit_event(Event::<T>::CampaignStarted(crowdloan_id));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn add_contributer(
            origin: OriginFor<T>,
            crowdloan_id: CrowdloanIdOf<T>,
            contributer: AccountIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            Self::ensure_hoster(origin, crowdloan_id)?;
            Self::ensure_campaign_writable(&crowdloan_id)?;

            ensure!(!<Contribution<T>>::contains_key(&crowdloan_id, &contributer), <Error<T>>::ContributerExists);

            let campaign_info = Self::get_reward_info(&crowdloan_id).ok_or(<Error<T>>::NoRewardCampaign)?;
            let reward_unit = functions::construct_reward_unit::<T>(
                amount,
                campaign_info.instant_percentage,
                campaign_info.starts_from,
                campaign_info.end_target,
            )?;

            <Contribution<T>>::insert(&crowdloan_id, &contributer, reward_unit);

            Self::deposit_event(Event::<T>::ContributerAdded {
                crowdloan_id,
                contributer,
                amount,
            });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn remove_contributer(
            origin: OriginFor<T>,
            crowdloan_id: CrowdloanIdOf<T>,
            contributer: AccountIdOf<T>,
        ) -> DispatchResult {
            Self::ensure_hoster(origin, crowdloan_id)?;
            Self::ensure_campaign_writable(&crowdloan_id)?;

            ensure!(<Contribution<T>>::contains_key(&crowdloan_id, &contributer), <Error<T>>::NoContribution);

            <Contribution<T>>::remove(&crowdloan_id, &contributer);

            Self::deposit_event(Event::<T>::ContributerKicked {
                crowdloan_id,
                contributer,
            });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn lock_campaign(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>) -> DispatchResult {
            Self::ensure_hoster(origin, crowdloan_id)?;
            Self::ensure_campaign_lockable(&crowdloan_id)?;

            // TODO
            // - check the source have enough locked balance ( the locked field ) to reward all contributers

            <CampaignStatus<T>>::insert(crowdloan_id, RewardCampaignStatus::Locked);

            Self::deposit_event(Event::<T>::CampaignLocked(crowdloan_id));
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

            <CampaignStatus<T>>::insert(&crowdloan_id, RewardCampaignStatus::Wiped);

            Self::deposit_event(Event::<T>::CampaignWiped(crowdloan_id));
            Ok(())
        }

        #[pallet::weight(10_0000)]
        pub fn get_instant_reward(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>) -> DispatchResult {
            let contributer = ensure_signed(origin)?;
            Self::ensure_campaign_claimable(&crowdloan_id)?;
            let InstantEnsuredResultOf::<T> {
                new_status,
                instant_amount,
            } = Self::ensure_instant_claimable(&crowdloan_id, &contributer)?;

            let reward_info = Self::get_reward_info(&crowdloan_id).ok_or(<Error<T>>::NoRewardCampaign)?;

            functions::do_instant_reward::<T>(&reward_info.reward_source, &contributer, instant_amount)?;

            Self::update_contributer_status(&crowdloan_id, &contributer, new_status);

            Self::deposit_event(Event::<T>::InstantRewarded {
                crowdloan_id,
                contributer,
            });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn get_vested_reward(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>) -> DispatchResult {
            let contributer = ensure_signed(origin)?;
            Self::ensure_campaign_claimable(&crowdloan_id)?;
            let VestedEnsuredResultOf::<T> {
                new_status,
                vesting_amount,
                per_block,
            } = Self::ensure_vested_claimable(&crowdloan_id, &contributer)?;

            let reward_info = Self::get_reward_info(&crowdloan_id).ok_or(<Error<T>>::NoRewardCampaign)?;

            functions::do_vesting_reward::<T>(
                reward_info.reward_source.clone(),
                reward_info.starts_from,
                contributer.clone(),
                vesting_amount,
                per_block,
            )?;

            Self::update_contributer_status(&crowdloan_id, &contributer, new_status);

            Self::deposit_event(Event::<T>::VestingScheduleApplied {
                crowdloan_id,
                contributer,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn ensure_hoster(origin: OriginFor<T>, crowdloan_id: CrowdloanIdOf<T>) -> DispatchResult {
            let claimer = ensure_signed(origin)?;
            Self::verify_crowdloan_hoster(&claimer, &crowdloan_id)
        }

        fn verify_crowdloan_hoster(signer: &AccountIdOf<T>, crowdloan_id: &CrowdloanIdOf<T>) -> DispatchResult {
            (&Self::get_reward_info(crowdloan_id).ok_or(<Error<T>>::NoRewardCampaign)?.hoster == signer)
                .then_some(())
                .ok_or(<Error<T>>::PermissionDenied.into())
        }

        fn ensure_campaign_writable(crowdloan_id: &CrowdloanIdOf<T>) -> DispatchResult {
            ensure!(
                Self::get_campaign_status(crowdloan_id).ok_or(<Error<T>>::NoRewardCampaign)? ==
                    RewardCampaignStatus::InProgress,
                <Error<T>>::ReadOnlyCampaign
            );
            Ok(())
        }

        fn ensure_campaign_claimable(crowdloan_id: &CrowdloanIdOf<T>) -> DispatchResult {
            ensure!(
                Self::get_campaign_status(crowdloan_id).ok_or(<Error<T>>::NoRewardCampaign)? ==
                    RewardCampaignStatus::Locked,
                <Error<T>>::CampaignNotLocked
            );
            Ok(())
        }

        fn ensure_campaign_lockable(crowdloan_id: &CrowdloanIdOf<T>) -> DispatchResult {
            ensure!(
                Self::get_campaign_status(crowdloan_id).ok_or(<Error<T>>::NoRewardCampaign)? ==
                    RewardCampaignStatus::InProgress,
                <Error<T>>::NonLockableCampaign
            );
            Ok(())
        }

        fn ensure_instant_claimable(
            crowdloan_id: &CrowdloanIdOf<T>,
            contributer: &AccountIdOf<T>,
        ) -> Result<InstantEnsuredResultOf<T>, DispatchError> {
            let info = Self::get_contribution(crowdloan_id, contributer).ok_or(Error::<T>::NoContribution)?;

            let new_status = match info.status {
                ClaimerStatus::Unprocessed => Ok(ClaimerStatus::DoneInstant),
                ClaimerStatus::DoneVesting => Ok(ClaimerStatus::DoneBoth),
                ClaimerStatus::DoneInstant | ClaimerStatus::DoneBoth => Err(Error::<T>::InstantRewardTaken),
            }?;

            Ok(InstantEnsuredResultOf::<T> {
                new_status,
                instant_amount: info.instant_amount,
            })
        }

        fn ensure_vested_claimable(
            crowdloan_id: &CrowdloanIdOf<T>,
            contributer: &AccountIdOf<T>,
        ) -> Result<VestedEnsuredResultOf<T>, DispatchError> {
            let info = Self::get_contribution(crowdloan_id, contributer).ok_or(Error::<T>::NoContribution)?;

            let new_status = match info.status {
                ClaimerStatus::Unprocessed => Ok(ClaimerStatus::DoneVesting),
                ClaimerStatus::DoneInstant => Ok(ClaimerStatus::DoneBoth),
                ClaimerStatus::DoneVesting | ClaimerStatus::DoneBoth => Err(Error::<T>::VestingRewardApplied),
            }?;

            Ok(VestedEnsuredResultOf::<T> {
                new_status,
                vesting_amount: info.vesting_amount,
                per_block: info.per_block,
            })
        }

        fn update_contributer_status(
            crowdloan_id: &CrowdloanIdOf<T>,
            contributer: &AccountIdOf<T>,
            new_status: ClaimerStatus,
        ) {
            <Contribution<T>>::mutate(&crowdloan_id, &contributer, |state| {
                state.as_mut().map(|state| {
                    state.status = new_status;
                });
            });
        }

        fn get_current_block_number() -> BlockNumberOf<T> {
            <frame_system::Pallet<T>>::block_number()
        }
    }
}
