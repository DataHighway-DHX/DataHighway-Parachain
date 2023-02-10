use codec::{
    Decode,
    Encode,
    MaxEncodedLen,
};
use frame_support::traits::Currency;
use pallet_vesting::VestingInfo;
use scale_info::TypeInfo;
// use sp_runtime::Percent;
use sp_runtime::{
    traits::{
        CheckedDiv,
        CheckedMul,
    },
    ArithmeticError,
};
use sp_std::fmt::Debug;

/// Represent the status of claimer
#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, MaxEncodedLen, Debug)]
pub enum ClaimerStatus {
    /// This is a fresh entry and nothing is being processed
    Unprocessed,
    /// Instant transfer have been done
    /// but vesting schedule have not been applied
    DoneInstant,
    /// Vesting scheduled have been applied
    /// but instant transfer is not done
    DoneVesting,
    /// Both instant transfer and vesting schedule
    /// is applied
    DoneBoth,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, MaxEncodedLen, Debug)]
pub struct RewardUnit<InstantBalance, VestingBalance> {
    pub instant_amount: InstantBalance,
    pub vesting_amount: VestingBalance,
    pub per_block: VestingBalance,
    pub status: ClaimerStatus,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, MaxEncodedLen, Debug)]
pub struct CrowdloanReward<AccountId, BlockNumber, Balance> {
    /// Hoster of this crowdload
    /// Note: if this needs to be owned by multiple AccountId,
    /// make this account id a multi-signature
    pub hoster: AccountId,
    /// The source account to give reward from
    pub reward_source: AccountId,
    /// Total pool limit if exists
    pub total_pool: Option<Balance>,
    /// How many of total user reward will be given instantly
    pub instant_percentage: SmallRational,
    /// How many of total user reward will be given in vested manner
    pub vesting_percentage: SmallRational,
    /// This crowdload rewards starts from
    pub starts_from: BlockNumber,
    /// Is there any targeted time until when we prefer to finish the reward distribution
    pub end_target: BlockNumber,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, MaxEncodedLen, Debug)]
pub struct CrowdloanRewardParam<AccountId, BlockNumber, Balance> {
    // If present change the hoster
    // else:
    // 		updating: use previous one
    // 		creating: use the origin
    pub hoster: Option<AccountId>,
    // If present change the reward source
    // else: 0 (while creating) or previous (while updating)
    pub reward_source: Option<AccountId>,
    // if preset change the total pool
    // else: throw error ( while creating) or previous (while updating)
    pub total_pool: Option<Option<Balance>>,
    // if present change the instant percentage
    // else: throw error (while creating) or unchanged ( while updating )
    pub instant_percentage: Option<SmallRational>,
    // same asd instant_percentage
    pub vesting_percentage: Option<SmallRational>,
    // if None set to previous one ( whole updating ) None (while creating)
    // if Some(Some(bl)) change to Some(bl)
    // if Some(None) set to None
    pub starts_from: Option<BlockNumber>,
    // same as starts_form
    pub end_target: Option<BlockNumber>,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, MaxEncodedLen, Debug)]
pub enum RewardCampaignStatus {
    /// A campaign is in progress from this blocknumber
    InProgress,
    /// A campaign is in locked state from this blocknumber
    Locked,
    /// A previous campaign ended in this blocknumber
    Ended,
    /// This crowdloan existed but have been wiped
    Wiped,
}

pub struct InstantEnsuredResult<Balance> {
    pub new_status: ClaimerStatus,
    pub instant_amount: Balance,
}

pub struct VestedEnsuredResult<Balance> {
    pub new_status: ClaimerStatus,
    pub vesting_amount: Balance,
    pub per_block: Balance,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, MaxEncodedLen, Debug)]
pub struct SmallRational {
    pub numenator: u32,
    pub denomator: u32,
}

impl SmallRational {
    pub fn checked_mul<Number>(self, number: Number) -> Option<Number>
    where
        Number: From<u32> + CheckedMul + CheckedDiv,
    {
        number.checked_mul(&self.numenator.into()).map(|u| u.checked_div(&self.denomator.into())).flatten().or_else(
            || number.checked_div(&self.denomator.into()).map(|d| d.checked_mul(&self.numenator.into())).flatten(),
        )
    }
}

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type CrowdloanRewardFor<T> = CrowdloanReward<AccountIdOf<T>, BlockNumberOf<T>, BalanceOf<T>>;
pub type CrowdloanRewardParamFor<T> = CrowdloanRewardParam<AccountIdOf<T>, BlockNumberOf<T>, BalanceOf<T>>;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type BalanceOf<T> = <<T as crate::Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type RewardUnitOf<T> = RewardUnit<BalanceOf<T>, VestingBalanceOf<T>>;
pub type CrowdloanIdOf<T> = <T as crate::Config>::CrowdloanId;
pub type VestedEnsuredResultOf<T> = VestedEnsuredResult<VestingBalanceOf<T>>;
pub type InstantEnsuredResultOf<T> = InstantEnsuredResult<BalanceOf<T>>;
pub type VestingInfoOf<T> = VestingInfo<VestingBalanceOf<T>, BlockNumberOf<T>>;
pub type VestingBalanceOf<T> = <<T as pallet_vesting::Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
