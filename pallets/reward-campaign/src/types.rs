use codec::{
    Decode,
    Encode,
    MaxEncodedLen,
};
use core::cmp::Ord;
use frame_support::traits::Currency;
use pallet_vesting::VestingInfo;
use scale_info::TypeInfo;
use sp_runtime::traits::{
    CheckedDiv,
    CheckedMul,
    Zero,
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
pub struct CampaignReward<AccountId, BlockNumber> {
    /// Hoster of this crowdload
    /// Note: if this needs to be owned by multiple AccountId,
    /// make this account id a multi-signature
    pub hoster: AccountId,
    /// The source account to give reward from
    pub reward_source: AccountId,
    /// How many of total user reward will be given instantly
    pub instant_percentage: SmallRational,
    /// This crowdload rewards starts from
    pub starts_from: BlockNumber,
    /// Is there any targeted time until when we prefer to finish the reward distribution
    pub end_target: BlockNumber,
}

#[cfg(test)]
impl<Account, Block> Default for CampaignReward<Account, Block>
where
    Account: Default,
    Block: Default,
{
    fn default() -> Self {
        Self {
            hoster: Default::default(),
            reward_source: Default::default(),
            instant_percentage: SmallRational::new(1, 1),
            starts_from: Default::default(),
            end_target: Default::default(),
        }
    }
}

/// Paramater required to start a new reward campaign
#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, MaxEncodedLen, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct CreateCampaignParam<AccountId, BlockNumber>
where
    BlockNumber: Ord,
{
    /// Who owns this campaign and also the funder of whole reward
    /// If not passed, use the origin
    pub hoster: Option<AccountId>,
    /// How much percentage out of 100% to give as instantly without vesting scheule
    pub instant_percentage: SmallRational,
    /// Starting block number for vesting schedule to be applied
    /// If not passed, use the current block number
    pub starts_from: Option<BlockNumber>,
    /// Target block number to prefer to end the vesting scheudle
    pub end_target: BlockNumber,
}

impl<Account, BlockNumber> CampaignReward<Account, BlockNumber>
where
    BlockNumber: Ord,
{
    pub fn validate(&self) -> Option<()> {
        if self.instant_percentage.denomator.is_zero() || !self.end_target.cmp(&self.starts_from).is_gt() {
            None
        } else {
            Some(())
        }
    }
}

/// Parameter to update the already existing reward campaign
/// if any of the field is passed as None,
/// then the old value will be used in-place
#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, MaxEncodedLen, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct UpdateCampaignParam<AccountId, BlockNumber> {
    pub hoster: Option<AccountId>,
    pub instant_percentage: Option<SmallRational>,
    pub starts_from: Option<BlockNumber>,
    pub end_target: Option<BlockNumber>,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, MaxEncodedLen, Debug)]
pub enum RewardCampaignStatus {
    /// A campaign is in progress
    InProgress,
    /// A campaign is in locked state
    Locked,
    /// This campaign existed but have been wiped
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

#[cfg(test)]
impl Default for SmallRational {
    fn default() -> Self {
        SmallRational::new(0, 1)
    }
}

impl SmallRational {
    pub fn new(n: u32, d: u32) -> Self {
        SmallRational {
            numenator: n,
            denomator: d,
        }
    }

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
pub type CampaignRewardFor<T> = CampaignReward<AccountIdOf<T>, BlockNumberOf<T>>;
pub type CreateCampaignParamFor<T> = CreateCampaignParam<AccountIdOf<T>, BlockNumberOf<T>>;
pub type UpdateCampaignParamFor<T> = UpdateCampaignParam<AccountIdOf<T>, BlockNumberOf<T>>;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type BalanceOf<T> = <<T as crate::Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type RewardUnitOf<T> = RewardUnit<BalanceOf<T>, VestingBalanceOf<T>>;
pub type CampaignIdOf<T> = <T as crate::Config>::CampaignId;
pub type VestedEnsuredResultOf<T> = VestedEnsuredResult<VestingBalanceOf<T>>;
pub type InstantEnsuredResultOf<T> = InstantEnsuredResult<BalanceOf<T>>;
pub type VestingInfoOf<T> = VestingInfo<VestingBalanceOf<T>, BlockNumberOf<T>>;
pub type VestingBalanceOf<T> = <<T as pallet_vesting::Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
