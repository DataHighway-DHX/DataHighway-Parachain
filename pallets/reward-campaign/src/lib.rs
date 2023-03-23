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
pub mod types;
pub mod weights;

pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::{
        functions,
        types,
        weights,
    };
    use frame_support::{
        pallet_prelude::{
            DispatchResult,
            *,
        },
        traits::Currency,
    };
    use frame_system::pallet_prelude::{
        OriginFor,
        *,
    };
    pub use sp_runtime::Percent;
    use sp_runtime::{
        traits::{
            AtLeast32Bit,
            Convert,
            MaybeDisplay,
        },
        DispatchError,
    };
    use sp_std::fmt::Debug;
    use types::{
        AccountIdOf,
        BalanceOf,
        BlockNumberOf,
        CampaignIdOf,
        CampaignRewardFor,
        ClaimerStatus,
        CreateCampaignParamFor,
        InstantEnsuredResultOf,
        RewardCampaignStatus,
        RewardUnitOf,
        UpdateCampaignParamFor,
        VestedEnsuredResultOf,
        VestingBalanceOf,
    };
    use weights::WeightInfo;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration trait for reward campaign pallet
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_vesting::Config {
        /// Overarching event type
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Currency type to mainly represent free balance
        type Currency: Currency<AccountIdOf<Self>>;

        /// Unique Identifier to identify a campaign
        type CampaignId: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + Default
            + MaybeDisplay
            + AtLeast32Bit
            + Copy
            + MaxEncodedLen;

        /// Convert block number type to balance type
        type BlockNumberToBalance: Convert<BlockNumberOf<Self>, BalanceOf<Self>>;

        /// Conversion interface to convert between VestingBalance and Currency
        type CurrencyConvert: Convert<BalanceOf<Self>, VestingBalanceOf<Self>>
            + Convert<VestingBalanceOf<Self>, BalanceOf<Self>>;

        /// Weight information for extrinsic
        type WeightInfo: WeightInfo;
    }

    /// Map campaign_id to it's current status
    #[pallet::storage]
    #[pallet::getter(fn get_campaign_status)]
    pub type CampaignStatus<T: Config> = StorageMap<_, Blake2_128Concat, CampaignIdOf<T>, RewardCampaignStatus>;

    /// Map campaign_id to the details of how reward shall be executed
    #[pallet::storage]
    #[pallet::getter(fn get_reward_info)]
    pub type RewardInfo<T: Config> = StorageMap<_, Blake2_128Concat, CampaignIdOf<T>, CampaignRewardFor<T>>;

    /// Map the pair of campaign_id and contributor accountId to
    /// the details of how much and how this contributor is to be rewarded
    #[pallet::storage]
    #[pallet::getter(fn get_contribution)]
    pub type Contribution<T> =
        StorageDoubleMap<_, Blake2_128Concat, CampaignIdOf<T>, Blake2_128Concat, AccountIdOf<T>, RewardUnitOf<T>>;

    /// Set of event thrown from this pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new reward campaign with `CrowdloanId` started
        CampaignStarted(CampaignIdOf<T>),
        /// This reward campaign's info have been updated
        CampaignUpdated(CampaignIdOf<T>),
        /// Campaign have been locked
        CampaignLocked(CampaignIdOf<T>),
        /// Campaign have been wiped
        CampaignWiped(CampaignIdOf<T>),
        /// Campaign have been discarded
        CampaignDiscarded(CampaignIdOf<T>),
        /// A contributor received instant amount of reward
        InstantRewarded {
            campaign_id: CampaignIdOf<T>,
            contributor: AccountIdOf<T>,
        },
        /// Vesting schdule of vesting amount have been applied to user
        VestingScheduleApplied {
            campaign_id: CampaignIdOf<T>,
            contributor: AccountIdOf<T>,
        },
        /// A contributor have been added as rewardee
        ContributerAdded {
            campaign_id: CampaignIdOf<T>,
            contributor: AccountIdOf<T>,
            amount: BalanceOf<T>,
        },
        /// A contributor have been removed from campaign
        ContributerKicked {
            campaign_id: CampaignIdOf<T>,
            contributor: AccountIdOf<T>,
        },
    }

    /// Error specific to this pallet
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
        /// This contributor already exists
        ContributerExists,
        /// Campaign have been locked
        CampaignLocked,
        /// Campaign is in progressed
        CampaignInProgress,
        /// Campaign wiped
        CampaignWiped,
        /// This crowdloan is in one of read-only state
        ReadOnlyCampaign,
        /// This campaign is not yet in claimable state for contributors
        CampaignNotLocked,
        /// This campaign cannot be locked
        NonLockableCampaign,
        /// This Reward have been taken by this contributor
        RewardTaken,
        CanotSplitAmount,
        /// This campaign is not empty. i.e some contributors exists
        NonEmptyCampaign,
        /// Some contributor have not claimed their reward yet
        UnclaimedContribution,
        /// Campaign is not in claimable state
        NonClaimableCampaign,
        /// Provided input is invalid
        InvalidInput,
        /// Added reward amount is too small
        RewardTooSmall,
    }

    /// Extrinsic calls
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Start a new reward campaign under `campaign_id`
        /// information of the campaign will be given under `info: CreateCampaignParam`
        #[pallet::weight(<T as Config>::WeightInfo::start_new_campaign())]
        pub fn start_new_campaign(
            origin: OriginFor<T>,
            campaign_id: CampaignIdOf<T>,
            info: CreateCampaignParamFor<T>,
        ) -> DispatchResult {
            let hoster = ensure_signed(origin)?;

            ensure!(!<CampaignStatus<T>>::contains_key(&campaign_id), <Error<T>>::RewardCampaignExists);
            ensure!(!<RewardInfo<T>>::contains_key(&campaign_id), <Error<T>>::RewardCampaignExists);
            ensure!(
                <Contribution<T>>::iter_key_prefix(&campaign_id).next().is_none(),
                <Error<T>>::RewardCampaignExists
            );

            let starts_from = info.starts_from.unwrap_or_else(Self::get_current_block_number);
            let hoster = info.hoster.unwrap_or(hoster);
            let reward_source = hoster.clone();

            let campaign_info = CampaignRewardFor::<T> {
                hoster,
                reward_source,
                end_target: info.end_target,
                starts_from,
                instant_percentage: info.instant_percentage,
            };

            ensure!(campaign_info.validate().is_some(), <Error<T>>::InvalidInput);

            <CampaignStatus<T>>::insert(campaign_id, RewardCampaignStatus::InProgress);
            <RewardInfo<T>>::insert(campaign_id, campaign_info);

            Self::deposit_event(Event::<T>::CampaignStarted(campaign_id));
            Ok(())
        }

        /// Update the writeable ( non-locked ) campaign under `campaign_id`
        /// with the new infromation passed in `new_info`
        /// and left out values will be left unchanged
        #[pallet::weight(<T as Config>::WeightInfo::update_campaign())]
        pub fn update_campaign(
            origin: OriginFor<T>,
            campaign_id: CampaignIdOf<T>,
            new_info: UpdateCampaignParamFor<T>,
        ) -> DispatchResult {
            Self::ensure_hoster(origin, campaign_id)?;
            Self::ensure_campaign_writable(&campaign_id)?;

            ensure!(<Contribution<T>>::iter_key_prefix(&campaign_id).next().is_none(), <Error<T>>::NonEmptyCampaign);
            let old_info = Self::get_reward_info(&campaign_id).ok_or(<Error<T>>::NoRewardCampaign)?;

            let end_target = new_info.end_target.unwrap_or(old_info.end_target);
            let instant_percentage = new_info.instant_percentage.unwrap_or(old_info.instant_percentage);
            let starts_from = new_info.starts_from.unwrap_or(old_info.starts_from);
            let hoster = new_info.hoster.unwrap_or(old_info.hoster);
            let reward_source = hoster.clone();

            let campaign_info = CampaignRewardFor::<T> {
                hoster,
                reward_source,
                end_target,
                starts_from,
                instant_percentage,
            };

            ensure!(campaign_info.validate().is_some(), <Error<T>>::InvalidInput);

            <RewardInfo<T>>::insert(campaign_id, campaign_info);

            Self::deposit_event(<Event<T>>::CampaignUpdated(campaign_id));
            Ok(())
        }

        /// Add a user `contributor` as a rewardee of writeable campaign `campaign_id`
        /// with the total reward amount of `amount`
        #[pallet::weight(<T as Config>::WeightInfo::add_contributor())]
        pub fn add_contributor(
            origin: OriginFor<T>,
            campaign_id: CampaignIdOf<T>,
            contributor: AccountIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            Self::ensure_hoster(origin, campaign_id)?;
            Self::ensure_campaign_writable(&campaign_id)?;

            ensure!(!<Contribution<T>>::contains_key(&campaign_id, &contributor), <Error<T>>::ContributerExists);

            let campaign_info = Self::get_reward_info(&campaign_id).ok_or(<Error<T>>::NoRewardCampaign)?;
            let reward_unit = functions::construct_reward_unit::<T>(
                amount,
                campaign_info.instant_percentage,
                campaign_info.starts_from,
                campaign_info.end_target,
            )?;

            <Contribution<T>>::insert(&campaign_id, &contributor, reward_unit);

            Self::deposit_event(Event::<T>::ContributerAdded {
                campaign_id,
                contributor,
                amount,
            });
            Ok(())
        }

        /// remove user `contributor` under unlocked campaign `campaign_id`
        #[pallet::weight(<T as Config>::WeightInfo::remove_contributor())]
        pub fn remove_contributor(
            origin: OriginFor<T>,
            campaign_id: CampaignIdOf<T>,
            contributor: AccountIdOf<T>,
        ) -> DispatchResult {
            Self::ensure_hoster(origin, campaign_id)?;
            Self::ensure_campaign_writable(&campaign_id)?;

            ensure!(<Contribution<T>>::contains_key(&campaign_id, &contributor), <Error<T>>::NoContribution);

            <Contribution<T>>::remove(&campaign_id, &contributor);

            Self::deposit_event(Event::<T>::ContributerKicked {
                campaign_id,
                contributor,
            });
            Ok(())
        }

        /// lock the writable campaign under `campaign_id` and make it read-only
        #[pallet::weight(<T as Config>::WeightInfo::lock_campaign())]
        pub fn lock_campaign(origin: OriginFor<T>, campaign_id: CampaignIdOf<T>) -> DispatchResult {
            Self::ensure_hoster(origin, campaign_id)?;
            Self::ensure_campaign_lockable(&campaign_id)?;

            <CampaignStatus<T>>::insert(campaign_id, RewardCampaignStatus::Locked);

            Self::deposit_event(Event::<T>::CampaignLocked(campaign_id));
            Ok(())
        }

        /// discard the in-progress campaign information of `campaign_id`
        /// and remove all the contributors and reward details from chain
        #[pallet::weight(10_0000)]
        pub fn discard_campaign(origin: OriginFor<T>, campaign_id: CampaignIdOf<T>) -> DispatchResult {
            Self::ensure_hoster(origin, campaign_id)?;
            Self::ensure_campaign_discardable(&campaign_id)?;

            <Contribution<T>>::remove_prefix(&campaign_id, None);
            <RewardInfo<T>>::remove(&campaign_id);
            <CampaignStatus<T>>::remove(&campaign_id);

            Self::deposit_event(Event::<T>::CampaignDiscarded(campaign_id));

            Ok(())
        }

        /// wipe the campaign under `campaign_id` after all the
        /// contributors have claimed their reward
        /// this will still keep the status as `Wiped`
        #[pallet::weight(<T as Config>::WeightInfo::wipe_campaign())]
        pub fn wipe_campaign(origin: OriginFor<T>, campaign_id: CampaignIdOf<T>) -> DispatchResult {
            Self::ensure_hoster(origin, campaign_id.clone())?;
            Self::ensure_campaign_wipable(&campaign_id)?;

            <Contribution<T>>::remove_prefix(&campaign_id, None);
            <RewardInfo<T>>::remove(&campaign_id);

            <CampaignStatus<T>>::insert(&campaign_id, RewardCampaignStatus::Wiped);

            Self::deposit_event(Event::<T>::CampaignWiped(campaign_id));
            Ok(())
        }

        /// Contributer callable to receive the instant reward
        /// they are entitled to receive in campaign `campaign_id`
        #[pallet::weight(<T as Config>::WeightInfo::get_instant_reward())]
        pub fn get_instant_reward(origin: OriginFor<T>, campaign_id: CampaignIdOf<T>) -> DispatchResult {
            let contributor = ensure_signed(origin)?;
            Self::ensure_campaign_claimable(&campaign_id)?;
            let InstantEnsuredResultOf::<T> {
                new_status,
                instant_amount,
            } = Self::ensure_instant_claimable(&campaign_id, &contributor)?;

            let reward_info = Self::get_reward_info(&campaign_id).ok_or(<Error<T>>::NoRewardCampaign)?;

            functions::do_instant_reward::<T>(&reward_info.reward_source, &contributor, instant_amount)?;

            Self::update_contributor_status(&campaign_id, &contributor, new_status);

            Self::deposit_event(Event::<T>::InstantRewarded {
                campaign_id,
                contributor,
            });
            Ok(())
        }

        /// Contributer callable to receive the vesting reward
        /// they are entitled to receive in campaign `campaign_id`
        #[pallet::weight(<T as Config>::WeightInfo::get_vested_reward())]
        pub fn get_vested_reward(origin: OriginFor<T>, campaign_id: CampaignIdOf<T>) -> DispatchResult {
            let contributor = ensure_signed(origin)?;
            Self::ensure_campaign_claimable(&campaign_id)?;
            let VestedEnsuredResultOf::<T> {
                new_status,
                vesting_amount,
                per_block,
            } = Self::ensure_vested_claimable(&campaign_id, &contributor)?;

            let reward_info = Self::get_reward_info(&campaign_id).ok_or(<Error<T>>::NoRewardCampaign)?;

            functions::do_vesting_reward::<T>(
                reward_info.reward_source.clone(),
                reward_info.starts_from,
                contributor.clone(),
                vesting_amount,
                per_block,
            )?;

            Self::update_contributor_status(&campaign_id, &contributor, new_status);

            Self::deposit_event(Event::<T>::VestingScheduleApplied {
                campaign_id,
                contributor,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// ensure that the origin is signed by the hoster of campaign_id
        fn ensure_hoster(origin: OriginFor<T>, campaign_id: CampaignIdOf<T>) -> DispatchResult {
            let claimer = ensure_signed(origin)?;
            Self::verify_crowdloan_hoster(&claimer, &campaign_id)
        }

        /// ensure the account `signer` and the hoster of `campaign_id` is same
        fn verify_crowdloan_hoster(signer: &AccountIdOf<T>, campaign_id: &CampaignIdOf<T>) -> DispatchResult {
            (&Self::get_reward_info(campaign_id).ok_or(<Error<T>>::NoRewardCampaign)?.hoster == signer)
                .then_some(())
                .ok_or(<Error<T>>::PermissionDenied.into())
        }

        /// ensure that `campaign_id` state is writable ( i.e non-locked )
        /// and changed can be made safely
        fn ensure_campaign_writable(campaign_id: &CampaignIdOf<T>) -> DispatchResult {
            ensure!(
                Self::get_campaign_status(campaign_id).ok_or(<Error<T>>::NoRewardCampaign)? ==
                    RewardCampaignStatus::InProgress,
                <Error<T>>::ReadOnlyCampaign
            );
            Ok(())
        }

        /// ensure campaign `campaign_id` is in state
        /// where contributor can claim the reward they are entitled to
        fn ensure_campaign_claimable(campaign_id: &CampaignIdOf<T>) -> DispatchResult {
            ensure!(
                Self::get_campaign_status(campaign_id).ok_or(<Error<T>>::NoRewardCampaign)? ==
                    RewardCampaignStatus::Locked,
                <Error<T>>::NonClaimableCampaign
            );
            Ok(())
        }

        /// ensure the campaign `campaign_id` state is writeable and can be locked
        fn ensure_campaign_lockable(campaign_id: &CampaignIdOf<T>) -> DispatchResult {
            ensure!(
                Self::get_campaign_status(campaign_id).ok_or(<Error<T>>::NoRewardCampaign)? ==
                    RewardCampaignStatus::InProgress,
                <Error<T>>::NonLockableCampaign
            );
            Ok(())
        }

        /// enusre campaign `campaign_id` can be wiped
        fn ensure_campaign_wipable(campaign_id: &CampaignIdOf<T>) -> DispatchResult {
            ensure!(
                Self::get_campaign_status(campaign_id).ok_or(<Error<T>>::NoRewardCampaign)? ==
                    RewardCampaignStatus::Locked,
                <Error<T>>::CampaignNotLocked,
            );
            ensure!(
                <Contribution<T>>::iter_key_prefix(campaign_id)
                    .filter(|acc| {
                        Self::get_contribution(campaign_id, acc).map(|p| p.status) != Some(ClaimerStatus::DoneBoth)
                    })
                    .next()
                    .is_none(),
                <Error<T>>::UnclaimedContribution,
            );

            Ok(())
        }

        /// ensure campaign `campain_id` can be discarded
        fn ensure_campaign_discardable(campaign_id: &CampaignIdOf<T>) -> DispatchResult {
            ensure!(
                Self::get_campaign_status(campaign_id).ok_or(<Error<T>>::NoRewardCampaign)? ==
                    RewardCampaignStatus::InProgress,
                <Error<T>>::CampaignLocked,
            );

            Ok(())
        }

        /// ensure `contributor` can claim instant_reward under `campaign_id`
        fn ensure_instant_claimable(
            campaign_id: &CampaignIdOf<T>,
            contributor: &AccountIdOf<T>,
        ) -> Result<InstantEnsuredResultOf<T>, DispatchError> {
            let info = Self::get_contribution(campaign_id, contributor).ok_or(Error::<T>::NoContribution)?;

            let new_status = match info.status {
                ClaimerStatus::Unprocessed => Ok(ClaimerStatus::DoneInstant),
                ClaimerStatus::DoneVesting => Ok(ClaimerStatus::DoneBoth),
                ClaimerStatus::DoneInstant | ClaimerStatus::DoneBoth => Err(Error::<T>::RewardTaken),
            }?;

            Ok(InstantEnsuredResultOf::<T> {
                new_status,
                instant_amount: info.instant_amount,
            })
        }

        /// ensure `contributor` can claim instant_reward under `campaign_id`
        fn ensure_vested_claimable(
            campaign_id: &CampaignIdOf<T>,
            contributor: &AccountIdOf<T>,
        ) -> Result<VestedEnsuredResultOf<T>, DispatchError> {
            let info = Self::get_contribution(campaign_id, contributor).ok_or(Error::<T>::NoContribution)?;

            let new_status = match info.status {
                ClaimerStatus::Unprocessed => Ok(ClaimerStatus::DoneVesting),
                ClaimerStatus::DoneInstant => Ok(ClaimerStatus::DoneBoth),
                ClaimerStatus::DoneVesting | ClaimerStatus::DoneBoth => Err(Error::<T>::RewardTaken),
            }?;

            Ok(VestedEnsuredResultOf::<T> {
                new_status,
                vesting_amount: info.vesting_amount,
                per_block: info.per_block,
            })
        }

        /// update `contributor` status under `campaign_id` to `new_status`
        fn update_contributor_status(
            campaign_id: &CampaignIdOf<T>,
            contributor: &AccountIdOf<T>,
            new_status: ClaimerStatus,
        ) {
            <Contribution<T>>::mutate(&campaign_id, &contributor, |state| {
                state.as_mut().map(|state| {
                    state.status = new_status;
                });
            });
        }

        /// get current block number from `frame_system`
        fn get_current_block_number() -> BlockNumberOf<T> {
            <frame_system::Pallet<T>>::block_number()
        }
    }
}
