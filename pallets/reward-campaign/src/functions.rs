use crate::Error;
use frame_support::{
    pallet_prelude::DispatchResult,
    traits::{
        Currency,
        ExistenceRequirement,
    },
    *,
};
use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned,
        CheckedSub,
        Convert,
        Get,
        One,
        StaticLookup,
        Zero,
    },
    DispatchError,
};

use crate::types;

/// input details of how a contributor is supposed to receive his reward
/// this will output `SplittedAmount`
#[cfg_attr(test, derive(Clone, Debug))]
pub struct SplitableAmount<BlockNumber, Balance> {
    pub reward_amount: Balance,
    pub vesting_starts: BlockNumber,
    pub vesting_ends: BlockNumber,
    pub instant_percentage: types::SmallRational,
}

/// output of how a contributor is supposed to receive his reward
/// this will be the output generated from `SplittableAmount`
#[cfg_attr(test, derive(Eq, PartialEq, Debug, Clone))]
pub struct SplittedAmount<Balance> {
    pub instant_amount: Balance,
    pub vesting_amount: Balance,
    pub per_block: Balance,
}

impl<BlockNumber, Balance> SplitableAmount<BlockNumber, Balance>
where
    Balance: AtLeast32BitUnsigned + Clone,
    BlockNumber: CheckedSub + Clone,
{
    /// Convert `SplittableAmount` to `SplittedAmount`
    /// - how much ( if any ) to give in instant reward
    /// - if any vesting reward is to be given: how much and under what duration and at what rate
    pub fn split_amount<BlockNumberToBalance>(self) -> Option<SplittedAmount<Balance>>
    where
        BlockNumberToBalance: Convert<BlockNumber, Balance>,
    {
        let instant_amount = types::SmallRational::checked_mul(self.instant_percentage, self.reward_amount.clone())?;
        let vesting_amount = self.reward_amount.checked_sub(&instant_amount)?;
        let mut per_block = 0u32.into();

        if vesting_amount >= One::one() {
            let time_duration = self.vesting_ends.checked_sub(&self.vesting_starts)?;
            per_block = vesting_amount
                .checked_div(&BlockNumberToBalance::convert(time_duration.clone()))
                .unwrap_or_else(One::one);

            per_block = sp_std::cmp::max(One::one(), per_block);
        }

        Some(SplittedAmount {
            instant_amount,
            vesting_amount,
            per_block,
        })
    }
}

/// from the total reward amount of `amount`
/// make vesting reward ( if any ) starting from `starts_from`
/// and an ideal end target of `ends_at` with the instant-vesting
/// ratio of `instant_percentage`
pub fn construct_reward_unit<T: crate::Config>(
    amount: types::BalanceOf<T>,
    instant_percentage: types::SmallRational,
    starts_from: types::BlockNumberOf<T>,
    ends_at: types::BlockNumberOf<T>,
) -> Result<types::RewardUnitOf<T>, DispatchError> {
    let splittable_amount = SplitableAmount::<types::BlockNumberOf<T>, types::BalanceOf<T>> {
        instant_percentage,
        reward_amount: amount,
        vesting_starts: starts_from,
        vesting_ends: ends_at,
    };
    let SplittedAmount::<types::BalanceOf<T>> {
        instant_amount,
        vesting_amount,
        per_block,
    } = splittable_amount
        .split_amount::<<T as crate::Config>::BlockNumberToBalance>()
        .ok_or(Error::<T>::CanotSplitAmount)?;

    let vesting_amount = <T as crate::Config>::CurrencyConvert::convert(vesting_amount);
    let per_block = <T as crate::Config>::CurrencyConvert::convert(per_block);
    let min_vesting_amount = <T as pallet_vesting::Config>::MinVestedTransfer::get();

    // vesting should either be zero or at least minimum vesting_amount
    ensure!(vesting_amount.is_zero() || vesting_amount >= min_vesting_amount, Error::<T>::RewardTooSmall);

    Ok(types::RewardUnitOf::<T> {
        instant_amount,
        vesting_amount,
        per_block,
        status: types::ClaimerStatus::Unprocessed,
    })
}

/// Do instant reward to the `user` from account `reward_source`
/// with the amount `instant_amount`
pub fn do_instant_reward<T: crate::Config>(
    reward_source: &types::AccountIdOf<T>,
    user: &types::AccountIdOf<T>,
    instant_amount: types::BalanceOf<T>,
) -> DispatchResult {
    if instant_amount.is_zero() {
        Ok(())
    } else {
        <T as crate::Config>::Currency::transfer(reward_source, user, instant_amount, ExistenceRequirement::AllowDeath)
    }
}

/// Do vesting reward to the `user` from account `reward_source`
/// Vesting scheudle should start from `starts_from` and
/// `per_block` amount shall be released per block
/// to the total of `vesting_amount`
pub fn do_vesting_reward<T: crate::Config>(
    reward_source: types::AccountIdOf<T>,
    starts_from: types::BlockNumberOf<T>,
    user: types::AccountIdOf<T>,
    vesting_amount: types::VestingBalanceOf<T>,
    per_block: types::VestingBalanceOf<T>,
) -> DispatchResult {
    if vesting_amount.is_zero() {
        Ok(())
    } else {
        let vesting_info = types::VestingInfoOf::<T>::new(vesting_amount, per_block, starts_from);

        let creditor_origin = <T as frame_system::Config>::Origin::from(frame_system::RawOrigin::Signed(reward_source));
        let contributor_lookup = <T::Lookup as StaticLookup>::unlookup(user);

        pallet_vesting::Pallet::<T>::vested_transfer(creditor_origin, contributor_lookup, vesting_info)
    }
}
