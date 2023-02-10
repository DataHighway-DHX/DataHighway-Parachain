use crate::Error;
use frame_support::{
    pallet_prelude::DispatchResult,
    traits::{
        Currency,
        ExistenceRequirement,
    },
};
use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned,
        CheckedAdd,
        CheckedSub,
        Convert,
        One,
        StaticLookup,
    },
    ArithmeticError,
    DispatchError,
};

use crate::types;

pub struct SplitableAmount<BlockNumber, Balance> {
    pub reward_amount: Balance,
    pub vesting_starts: BlockNumber,
    pub vesting_ends: BlockNumber,
    pub instant_percentage: types::SmallRational,
}

pub struct SplittedAmount<Balance> {
    pub instant_amount: Balance,
    pub vesting_amount: Balance,
    pub per_block: Balance,
    pub reminder_amount: Balance,
}

impl<BlockNumber, Balance> SplitableAmount<BlockNumber, Balance>
where
    Balance: AtLeast32BitUnsigned + Clone,
    BlockNumber: CheckedSub + Clone,
{
    pub fn split_amount<BlockNumberToBalance>(self) -> Option<SplittedAmount<Balance>>
    where
        BlockNumberToBalance: Convert<BlockNumber, Balance>,
    {
        let instant_amount = types::SmallRational::checked_mul(self.instant_percentage, self.reward_amount.clone())?;
        let vesting_amount = self.reward_amount.checked_sub(&instant_amount)?;

        let time_duration = self.vesting_ends.checked_sub(&self.vesting_starts)?;
        let per_block =
            vesting_amount.checked_div(&BlockNumberToBalance::convert(time_duration.clone())).unwrap_or_else(One::one);
        let covered = per_block.checked_mul(&BlockNumberToBalance::convert(time_duration.clone()))?;
        let reminder_amount = vesting_amount.checked_sub(&covered)?;

        Some(SplittedAmount {
            instant_amount,
            vesting_amount,
            per_block,
            reminder_amount,
        })
    }
}

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
        mut instant_amount,
        vesting_amount,
        per_block,
        reminder_amount,
    } = splittable_amount
        .split_amount::<<T as crate::Config>::BlockNumberToBalance>()
        .ok_or(Error::<T>::CanotSplitAmount)?;

    instant_amount = instant_amount.checked_add(&reminder_amount).ok_or(ArithmeticError::Overflow)?;
    let vesting_amount = <T as crate::Config>::CurrencyConvert::convert(vesting_amount);
    let per_block = <T as crate::Config>::CurrencyConvert::convert(per_block);

    Ok(types::RewardUnitOf::<T> {
        instant_amount,
        vesting_amount,
        per_block,
        status: types::ClaimerStatus::Unprocessed,
    })
}

pub fn do_instant_reward<T: crate::Config>(
    reward_source: &types::AccountIdOf<T>,
    user: &types::AccountIdOf<T>,
    instant_amount: types::BalanceOf<T>,
) -> DispatchResult {
    <T as crate::Config>::Currency::transfer(reward_source, user, instant_amount, ExistenceRequirement::KeepAlive)
}

pub fn do_vesting_reward<T: crate::Config>(
    reward_source: types::AccountIdOf<T>,
    starts_from: types::BlockNumberOf<T>,
    user: types::AccountIdOf<T>,
    vesting_amount: types::VestingBalanceOf<T>,
    per_block: types::VestingBalanceOf<T>,
) -> DispatchResult {
    let vesting_info = types::VestingInfoOf::<T>::new(vesting_amount, per_block, starts_from);

    let creditor_origin = <T as frame_system::Config>::Origin::from(frame_system::RawOrigin::Signed(reward_source));
    let contributer_lookup = <T::Lookup as StaticLookup>::unlookup(user);

    pallet_vesting::Pallet::<T>::vested_transfer(creditor_origin, contributer_lookup, vesting_info)
}
