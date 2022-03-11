#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
    WASM_BINARY.expect(
        "Development wasm binary is not available. This means the client is built with `SKIP_WASM_BUILD` flag and it \
         is only usable for production chains. Please rebuild with the flag disabled.",
    )
}

pub mod xcm_config;

use log::{warn, info};
use codec::{
    Decode,
    Encode,
};
use frame_election_provider_support::onchain;
// use pallet_grandpa::{
//     fg_primitives,
//     AuthorityId as GrandpaId,
//     AuthorityList as GrandpaAuthorityList,
// };
// use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
// use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, u32_trait::{_1, _2, _3, _4, _5}, OpaqueMetadata};
use sp_inherents::{
    CheckInherentsResult,
    InherentData,
};
use sp_runtime::{
    create_runtime_str, curve::PiecewiseLinear, generic, impl_opaque_keys, traits,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, Convert, IdentifyAccount,
        IdentityLookup, NumberFor, OpaqueKeys, Saturating, StaticLookup,
        SaturatedConversion, Verify},
	transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature, FixedPointNumber,
};
pub use sp_runtime::{MultiAddress, Perbill, Percent, Permill, Perquintill};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;

pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{
        ConstU8, ConstU16, ConstU32, ConstU64, ConstU128, Currency, EnsureOneOf, EqualPrivilegeOnly,
        Everything, Imbalance, Contains, ContainsLengthBound, OnUnbalanced, KeyOwnerProofSystem,
        LockIdentifier, Randomness, StorageInfo, U128CurrencyToVote,
    },
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, IdentityFee, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	},
	PalletId,
    StorageValue,
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot,
};
use pallet_session::historical as pallet_session_historical;
pub use pallet_transaction_payment::{
    CurrencyAdapter,
    Multiplier,
    TargetedFeeAdjustment,
};
use pallet_transaction_payment::{
    FeeDetails,
    RuntimeDispatchInfo,
};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use xcm_config::{XcmConfig, XcmOriginToTransactDispatchOrigin};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
#[cfg(any(feature = "std", test))]
pub use pallet_balances::Call as BalancesCall;
#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;

// Polkadot Imports
use polkadot_runtime_common::{BlockHashCount as BlockHashCountCommon, RocksDbWeight, SlowAdjustingFeeUpdate};

// XCM Imports
use xcm::latest::prelude::BodyId;
use xcm_executor::XcmExecutor;

pub use module_primitives::{
    constants::currency::{
        CENTS,
        deposit,
        DOLLARS,
        EXISTENTIAL_DEPOSIT,
        MICROUNIT,
        MILLIUNIT,
        MILLICENTS,
        UNIT,
    },
    constants::time::{
        DAYS,
        EPOCH_DURATION_IN_BLOCKS,
        EPOCH_DURATION_IN_SLOTS,
        HOURS,
        MILLISECS_PER_BLOCK,
        MINUTES,
        PRIMARY_PROBABILITY,
        SLOT_DURATION,
    },
    types::{
        AccountIndex,
        Index,
        Moment,
    },
};

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, AccountIndex>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
    frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem, pallet_bags_list::migrations::CheckCounterPrefix<Runtime>>;

// /// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
// /// node's balance type.
// ///
// /// This should typically create a mapping between the following ranges:
// ///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
// ///   - `[Balance::min, Balance::max]`
// ///
// /// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
// ///   - Setting it to `0` will essentially disable the weight fee.
// ///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
// pub struct WeightToFee;
// impl WeightToFeePolynomial for WeightToFee {
//     type Balance = Balance;
//     fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
//         // in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
//         // in our template, we map to 1/10 of that, or 1/10 MILLIUNIT
//         let p = MILLIUNIT / 10;
//         let q = 100 * Balance::from(ExtrinsicBaseWeight::get());
//         smallvec![WeightToFeeCoefficient {
//         	degree: 1,
//         	negative: false,
//         	coeff_frac: Perbill::from_rational(p % q, q),
//         	coeff_integer: p / q,
//         }]
//     }
// }

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
    pub use super::{
        BlockNumber,
        Hash,
    };

	use sp_runtime::{generic, traits::BlakeTwo256};

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
        // pub grandpa: Grandpa,
        // pub im_online: ImOnline,
        // pub authority_discovery: AuthorityDiscovery,
    }
}

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;
pub use impls::Author;

use sp_runtime::generic::Era;

/// Generated voter bag information.
mod voter_bags;

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("datahighway-parachain"),
    impl_name: create_runtime_str!("datahighway-parachain"),
    authoring_version: 3,
    spec_version: 3,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item=NegativeImbalance>) {
        if let Some(fees) = fees_then_tips.next() {
            // for fees, 80% to treasury, 20% to author
            let mut split = fees.ration(80, 20);
            if let Some(tips) = fees_then_tips.next() {
                // for tips, if any, 80% to treasury, 20% to author (though this can be anything)
                tips.ration_merge_into(80, 20, &mut split);
            }
            Treasury::on_unbalanced(split.0);
            Author::on_unbalanced(split.1);
        }
    }
}

pub const BlockHashCountAsConst: BlockNumber = BlockHashCountCommon;
pub const SS58PrefixAsConst: u16 = 33;
pub const MaxConsumersAsConst: u32 = 16;

parameter_types! {
    pub const BlockHashCount: BlockNumber = BlockHashCountAsConst;
    pub const Version: RuntimeVersion = VERSION;
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
    pub const SS58Prefix: u16 = SS58PrefixAsConst;
    pub const MaxConsumers: u32 = MaxConsumersAsConst;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
    type BaseCallFilter = Everything;
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = RuntimeBlockLength;
    type DbWeight = RocksDbWeight;
    type Origin = Origin;
    type Call = Call;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    type Event = Event;
    type BlockHashCount = ConstU32<BlockHashCountAsConst>;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type SS58Prefix = ConstU16<SS58PrefixAsConst>;
    type MaxConsumers = ConstU32<MaxConsumersAsConst>;
}

pub const MaxAuthoritiesAsConst: u32 = 100;

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    type Moment = Moment;
    type OnTimestampSet = Aura;
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    // TODO - should this list also include Staking and ImOnline?
    type EventHandler = (CollatorSelection,);
}

pub const MaxLocksAsConst: u32 = 50;
pub const ExistentialDepositAsConst: Balance = EXISTENTIAL_DEPOSIT;

parameter_types! {
    pub const ExistentialDeposit: Balance = ExistentialDepositAsConst;
    pub const MaxLocks: u32 = MaxLocksAsConst;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    // TODO - is `System` the same as `frame_system::Pallet<Runtime>`?
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ConstU128<ExistentialDepositAsConst>;
    type MaxLocks = ConstU32<MaxLocksAsConst>;
    type MaxReserves = MaxReserves;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type ReserveIdentifier = [u8; 8];
}

pub const OperationalFeeMultiplierAsConst: u8 = 5;

parameter_types! {
    /// Relay Chain `TransactionByteFee` / 10
    // Note: Kusama relay chain's `TransactionByteFee` is 10 * MILLICENTS,
    // so 1/10 of that is 1 * MILLICENTS
    pub const TransactionByteFee: Balance = 1 * MILLICENTS;
    pub const OperationalFeeMultiplier: u8 = OperationalFeeMultiplierAsConst;
    // TODO - do we need the below in a parachain or only standalone?
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    // TODO - is this change required in parachain codebase or only standalone chain?
    type FeeMultiplierUpdate =
        TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
    type OperationalFeeMultiplier = ConstU8<OperationalFeeMultiplierAsConst>;
}

// parameter_types! {
//     pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
//     pub const CouncilMaxProposals: u32 = 100;
//     pub const CouncilMaxMembers: u32 = 100;
// }

// type GeneralCouncilInstance = pallet_collective::Instance1;
// impl pallet_collective::Config<GeneralCouncilInstance> for Runtime {
//     type DefaultVote = pallet_collective::PrimeDefaultVote;
//     type Event = Event;
//     type MaxMembers = CouncilMaxMembers;
//     type MaxProposals = CouncilMaxProposals;
//     type MotionDuration = CouncilMotionDuration;
//     type Origin = Origin;
//     type Proposal = Call;
//     type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
// }

// type GeneralCouncilMembershipInstance = pallet_membership::Instance1;
// impl pallet_membership::Config<GeneralCouncilMembershipInstance> for Runtime {
//     type AddOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
//     type Event = Event;
//     type MembershipChanged = GeneralCouncil;
//     type MembershipInitialized = GeneralCouncil;
//     type PrimeOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
//     type RemoveOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
//     type ResetOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
//     type SwapOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
//     type WeightInfo = ();
//     type MaxMembers = CouncilMaxMembers;
// }

// pub struct GeneralCouncilProvider;
// impl Contains<AccountId> for GeneralCouncilProvider {
//     fn contains(who: &AccountId) -> bool {
//         GeneralCouncil::is_member(who)
//     }
// }
// impl ContainsLengthBound for GeneralCouncilProvider {
//     fn min_len() -> usize {
//         0
//     }

//     fn max_len() -> usize {
//         100000
//     }
// }

// pallet_staking_reward_curve::build! {
//     const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
//         min_inflation: 0_025_000,
//         max_inflation: 0_100_000,
//         ideal_stake: 0_500_000,
//         falloff: 0_050_000,
//         max_piece_count: 40,
//         test_precision: 0_005_000,
//     );
// }

// parameter_types! {
//     //pub const SessionsPerEra: sp_staking::SessionIndex = 6;
//     //pub const BondingDuration: pallet_staking::EraIndex = 24 * 28;
//     //pub const SlashDeferDuration: pallet_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
//     //pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
//     //pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
//     pub const MaxNominatorRewardedPerValidator: u32 = 64;
//     pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
//     pub const MaxIterations: u32 = 10;
//     pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
//     /// A limit for off-chain phragmen unsigned solution submission.
//     ///
//     /// We want to keep it as high as possible, but can't risk having it reject,
//     /// so we always subtract the base block execution weight.
//     pub OffchainSolutionWeightLimit: Weight = RuntimeBlockWeights::get()
//         .get(DispatchClass::Normal)
//         .max_extrinsic
//         .expect("Normal extrinsics have weight limit configured by default; qed")
//         .saturating_sub(BlockExecutionWeight::get());
// }

// impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
// where
//     Call: From<C>,
// {
//     type Extrinsic = UncheckedExtrinsic;
//     type OverarchingCall = Call;
// }

// TODO - replace pallet_staking with
// https://purestake.github.io/moonbeam/parachain_staking/index.html
// impl pallet_staking::Config for Runtime {
//     type MaxNominations = MaxNominations;
//     type Currency = Balances;
//     type UnixTime = Timestamp;
//     type CurrencyToVote = U128CurrencyToVote;
//     type RewardRemainder = Treasury;
//     type Event = Event;
//     type Slash = Treasury; // send the slashed funds to the treasury.
//     type Reward = (); // rewards are minted from the void
//     type SessionsPerEra = SessionsPerEra;
//     type BondingDuration = BondingDuration;
//     type SlashDeferDuration = SlashDeferDuration;
//     /// A super-majority of the council can cancel the slash.
//     type SlashCancelOrigin = EnsureOneOf<
//         EnsureRoot<AccountId>,
//         pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>,
//     >;
//     type SessionInterface = Self;
//     type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
//     type NextNewSession = Session;
//     type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
//     type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
//     type ElectionProvider = ElectionProviderMultiPhase;
//     type GenesisElectionProvider = onchain::OnChainSequentialPhragmen<Self>;
// 	// Alternatively, use pallet_staking::UseNominatorsMap<Runtime> to just use the nominators map.
// 	// Note that the aforementioned does not scale to a very large number of nominators.
// 	type SortedListProvider = BagsList;
//     type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
//     type BenchmarkingConfig = StakingBenchmarkingConfig;
// }

parameter_types! {
    // matches Kusama
    pub const Period: BlockNumber = 10 * MINUTES;
    pub const Offset: BlockNumber = 0;
    pub const MaxAuthorities: u32 = MaxAuthoritiesAsConst;
}

impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    // we don't have stash and controller, thus we don't need the convert as well.
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = CollatorSelection;
    // Essentially just Aura, but lets be pedantic.
    type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_tips::Config for Runtime {
    type Event = Event;
    type DataDepositPerByte = DataDepositPerByte;
    type MaximumReasonLength = MaximumReasonLength;
    type Tippers = Elections;
    type TipCountdown = TipCountdown;
    type TipFindersFee = TipFindersFee;
    type TipReportDepositBase = TipReportDepositBase;
    type WeightInfo = pallet_tips::weights::SubstrateWeight<Runtime>;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<MaxAuthoritiesAsConst>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

pub const MaxScheduledPerBlockAsConst: u32 = 50;

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
        RuntimeBlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = MaxScheduledPerBlockAsConst;
    // Retry a scheduled item every 10 blocks (1 minute) until the preimage exists.
    pub const NoPreimagePostponement: Option<u32> = Some(10);
}

impl pallet_scheduler::Config for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = ConstU32<MaxScheduledPerBlockAsConst>;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type PreimageProvider = Preimage;
    type NoPreimagePostponement = NoPreimagePostponement;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
    where
        Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as traits::Verify>::Signer,
        account: AccountId,
        nonce: Index,
    ) -> Option<(Call, <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload)> {
        let tip = 0;
        // take the biggest period possible.
        let period = BlockHashCount::get()
            .checked_next_power_of_two()
            .map(|c| c / 2)
            .unwrap_or(2) as u64;
        let current_block = System::block_number()
            .saturated_into::<u64>()
            // The `System::block_number` is initialized with `n+1`,
            // so the actual block number is `n`.
            .saturating_sub(1);
        let era = Era::mortal(period, current_block);
        let extra = (
            frame_system::CheckNonZeroSender::<Runtime>::new(),
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(era),
            frame_system::CheckNonce::<Runtime>::from(nonce),
            frame_system::CheckWeight::<Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
        );
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = raw_payload
            .using_encoded(|payload| {
                C::sign(payload, public)
            })?;
        let address = Indices::unlookup(account);
        let (call, extra, _) = raw_payload.deconstruct();
        Some((call, (address, signature.into(), extra)))
    }
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as traits::Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}

parameter_types! {
    pub const IndexDeposit: Balance = 1 * DOLLARS;
}

impl pallet_indices::Config for Runtime {
    type AccountIndex = AccountIndex;
    type Currency = Balances;
    type Deposit = IndexDeposit;
    type Event = Event;
    type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}


parameter_types! {
    pub const CandidacyBond: Balance = 10 * DOLLARS;
    // 1 storage item created, key size is 32 bytes, value size is 16+16.
    pub const VotingBondBase: Balance = deposit(1, 64);
    // additional data per vote is 32 bytes (account id).
    pub const VotingBondFactor: Balance = deposit(0, 32);
    pub const TermDuration: BlockNumber = 7 * DAYS;
    // Check chain_spec. This value should be greater than or equal to the amount of
    // endowed accounts that are added to election_phragmen
    pub const DesiredMembers: u32 = 62; // validators 1-10 + sudo + treasury
    pub const DesiredRunnersUp: u32 = 7;
    pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
}

// Make sure that there are no more than `MaxMembers` members elected via elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
    type Event = Event;
    type PalletId = ElectionsPhragmenPalletId;
    type Currency = Balances;
    type ChangeMembers = Council;
    // NOTE: this implies that council's genesis members cannot be set directly and must come from
    // this module.
    type InitializeMembers = Council;
    type CurrencyToVote = U128CurrencyToVote;
    type CandidacyBond = CandidacyBond;
    type VotingBondBase = VotingBondBase;
    type VotingBondFactor = VotingBondFactor;
    type LoserCandidate = ();
    type KickedMember = ();
    type DesiredMembers = DesiredMembers;
    type DesiredRunnersUp = DesiredRunnersUp;
    type TermDuration = TermDuration;
    type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TechnicalMotionDuration: BlockNumber = 5 * DAYS;
    pub const TechnicalMaxProposals: u32 = 100;
    pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = TechnicalMotionDuration;
    type MaxProposals = TechnicalMaxProposals;
    type MaxMembers = TechnicalMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

type EnsureRootOrHalfCouncil = EnsureOneOf<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;
impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
    type Event = Event;
    type AddOrigin = EnsureRootOrHalfCouncil;
    type RemoveOrigin = EnsureRootOrHalfCouncil;
    type SwapOrigin = EnsureRootOrHalfCouncil;
    type ResetOrigin = EnsureRootOrHalfCouncil;
    type PrimeOrigin = EnsureRootOrHalfCouncil;
    type MembershipInitialized = TechnicalCommittee;
    type MembershipChanged = TechnicalCommittee;
	type MaxMembers = TechnicalMaxMembers;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 1 * DOLLARS;
    pub const SpendPeriod: BlockNumber = 1 * DAYS;
    pub const Burn: Permill = Permill::from_percent(0);
    pub const TipCountdown: BlockNumber = 1 * DAYS;
    pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: Balance = 1 * DOLLARS;
    pub const MaximumReasonLength: u32 = 300;
    pub const BountyValueMinimum: Balance = 5 * DOLLARS;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyDepositBase: Balance = 1 * DOLLARS;
    pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
    pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
    pub const DataDepositPerByte: Balance = 1 * CENTS;
    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
    pub const MaxApprovals: u32 = 100;
    pub const MaxActiveChildBountyCount: u32 = 5;
    pub const ChildBountyValueMinimum: Balance = 1 * DOLLARS;
    pub const ChildBountyCuratorDepositBase: Permill = Permill::from_percent(10);
}

impl pallet_treasury::Config for Runtime {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type ApproveOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
    >;
    type RejectOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
    >;
    type Event = Event;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type ProposalBondMaximum = ();
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = ();
    type SpendFunds = Bounties;
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
    type MaxApprovals = MaxApprovals;
}


pub const MaxVotesAsConst: u32 = 100;

parameter_types! {
    pub const LaunchPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
    pub const VotingPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
    pub const FastTrackVotingPeriod: BlockNumber = 3 * 24 * 60 * MINUTES;
    pub const InstantAllowed: bool = true;
    pub const MinimumDeposit: Balance = 100 * DOLLARS;
    pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * MINUTES;
    pub const CooloffPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
    pub const MaxVotes: u32 = MaxVotesAsConst;
    pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
    type Proposal = Call;
    type Event = Event;
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
    type MinimumDeposit = MinimumDeposit;
    /// A straight majority of the council can decide what their next motion is.
    type ExternalOrigin = pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
    /// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin = pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>;
    /// A unanimous council can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
    /// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
    type InstantOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>;
    type InstantAllowed = frame_support::traits::ConstBool<true>;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    // To cancel a proposal which has been passed, 2/3 of the council must agree to it.
    type CancellationOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
    // To cancel a proposal before it has been passed, the technical committee must be unanimous or
    // Root must agree.
    type CancelProposalOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
    >;
    type BlacklistOrigin = EnsureRoot<AccountId>;
    // Any single technical committee member may veto a coming council proposal, however they can
    // only do it once and it lasts only for the cooloff period.
    type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
    type CooloffPeriod = CooloffPeriod;
    type PreimageByteDeposit = PreimageByteDeposit;
    type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
    type Slash = Treasury;
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MaxVotes = frame_support::traits::ConstU32<MaxVotesAsConst>;
    type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
    type MaxProposals = MaxProposals;
}


impl roaming_operators::Config for Runtime {
    type Currency = Balances;
    type Event = Event;
    type Randomness = RandomnessCollectiveFlip;
    type RoamingOperatorIndex = u64;
}

impl roaming_networks::Config for Runtime {
    type Event = Event;
    type RoamingNetworkIndex = u64;
}

impl roaming_organizations::Config for Runtime {
    type Event = Event;
    type RoamingOrganizationIndex = u64;
}

impl roaming_network_servers::Config for Runtime {
    type Event = Event;
    type RoamingNetworkServerIndex = u64;
}

impl roaming_devices::Config for Runtime {
    type Event = Event;
    type RoamingDeviceIndex = u64;
}

impl roaming_routing_profiles::Config for Runtime {
    type Event = Event;
    // https://polkadot.js.org/api/types/#primitive-types
    type RoamingRoutingProfileAppServer = Vec<u8>;
    type RoamingRoutingProfileIndex = u64;
}

impl roaming_service_profiles::Config for Runtime {
    type Event = Event;
    type RoamingServiceProfileDownlinkRate = u32;
    type RoamingServiceProfileIndex = u64;
    type RoamingServiceProfileUplinkRate = u32;
}

impl roaming_accounting_policies::Config for Runtime {
    type Event = Event;
    type RoamingAccountingPolicyDownlinkFeeFactor = u32;
    type RoamingAccountingPolicyIndex = u64;
    type RoamingAccountingPolicyType = Vec<u8>;
    type RoamingAccountingPolicyUplinkFeeFactor = u32;
}

impl roaming_agreement_policies::Config for Runtime {
    type Event = Event;
    type RoamingAgreementPolicyActivationType = Vec<u8>;
    type RoamingAgreementPolicyIndex = u64; // <pallet_timestamp::Pallet<Runtime> as Config>::Moment` timestamp::Pallet<Runtime>::Moment;
}

impl roaming_network_profiles::Config for Runtime {
    type Event = Event;
    type RoamingNetworkProfileIndex = u64;
}

impl roaming_device_profiles::Config for Runtime {
    type Event = Event;
    type RoamingDeviceProfileDevAddr = Vec<u8>;
    type RoamingDeviceProfileDevEUI = Vec<u8>;
    type RoamingDeviceProfileIndex = u64;
    type RoamingDeviceProfileJoinEUI = Vec<u8>;
    type RoamingDeviceProfileVendorID = Vec<u8>;
}

impl roaming_sessions::Config for Runtime {
    type Event = Event;
    type RoamingSessionIndex = u64;
}

impl roaming_billing_policies::Config for Runtime {
    type Event = Event;
    type RoamingBillingPolicyIndex = u64;
}

impl roaming_charging_policies::Config for Runtime {
    type Event = Event;
    type RoamingChargingPolicyIndex = u64;
}

impl roaming_packet_bundles::Config for Runtime {
    type Event = Event;
    type RoamingPacketBundleExternalDataStorageHash = Hash;
    type RoamingPacketBundleIndex = u64;
    type RoamingPacketBundleReceivedAtHome = bool;
    type RoamingPacketBundleReceivedPacketsCount = u64;
    type RoamingPacketBundleReceivedPacketsOkCount = u64;
}

impl mining_setting_token::Config for Runtime {
    type Event = Event;
    // FIXME - restore when stop temporarily using roaming-operators
    // type Currency = Balances;
    // type Randomness = RandomnessCollectiveFlip;
    type MiningSettingTokenIndex = u64;
    type MiningSettingTokenLockAmount = u64;
    // Mining Speed Boost Token Mining Config
    // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
    type MiningSettingTokenType = Vec<u8>;
}

impl mining_setting_hardware::Config for Runtime {
    type Event = Event;
    type MiningSettingHardwareDevEUI = u64;
    // type MiningSettingHardwareType =
    // MiningSettingHardwareTypes;
    type MiningSettingHardwareID = u64;
    // FIXME - restore when stop temporarily using roaming-operators
    // type Currency = Balances;
    // type Randomness = RandomnessCollectiveFlip;
    type MiningSettingHardwareIndex = u64;
    // Mining Speed Boost Hardware Mining Config
    type MiningSettingHardwareSecure = bool;
    // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
    type MiningSettingHardwareType = Vec<u8>;
}

impl mining_rates_token::Config for Runtime {
    type Event = Event;
    type MiningRatesTokenIndex = u64;
    type MiningRatesTokenMaxLoyalty = u32;
    // Mining Speed Boost Max Rates
    type MiningRatesTokenMaxToken = u32;
    type MiningRatesTokenTokenDOT = u32;
    type MiningRatesTokenTokenIOTA = u32;
    // Mining Speed Boost Rate
    type MiningRatesTokenTokenMXC = u32;
}

impl mining_rates_hardware::Config for Runtime {
    type Event = Event;
    type MiningRatesHardwareCategory1MaxTokenBonusPerGateway = u32;
    type MiningRatesHardwareCategory2MaxTokenBonusPerGateway = u32;
    type MiningRatesHardwareCategory3MaxTokenBonusPerGateway = u32;
    type MiningRatesHardwareIndex = u64;
    type MiningRatesHardwareInsecure = u32;
    // Mining Speed Boost Max Rates
    type MiningRatesHardwareMaxHardware = u32;
    // Mining Speed Boost Rate
    type MiningRatesHardwareSecure = u32;
}

impl mining_sampling_token::Config for Runtime {
    type Event = Event;
    type MiningSamplingTokenIndex = u64;
    type MiningSamplingTokenSampleLockedAmount = u64;
}

impl mining_sampling_hardware::Config for Runtime {
    type Event = Event;
    type MiningSamplingHardwareIndex = u64;
    type MiningSamplingHardwareSampleHardwareOnline = u64;
}

impl mining_eligibility_token::Config for Runtime {
    type Event = Event;
    type MiningEligibilityTokenCalculatedEligibility = u64;
    type MiningEligibilityTokenIndex = u64;
    type MiningEligibilityTokenLockedPercentage = u32;
    // type MiningEligibilityTokenAuditorAccountID = u64;
}

impl mining_eligibility_hardware::Config for Runtime {
    type Event = Event;
    type MiningEligibilityHardwareCalculatedEligibility = u64;
    type MiningEligibilityHardwareIndex = u64;
    type MiningEligibilityHardwareUptimePercentage = u32;
    // type MiningEligibilityHardwareAuditorAccountID = u64;
}

impl mining_eligibility_proxy::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    // Check membership
    type MembershipSource = MembershipSupernodes;
    type MiningEligibilityProxyIndex = u64;
    type RewardsOfDay = u64;
}

impl mining_claims_token::Config for Runtime {
    type Event = Event;
    type MiningClaimsTokenClaimAmount = u64;
    type MiningClaimsTokenIndex = u64;
}

impl mining_claims_hardware::Config for Runtime {
    type Event = Event;
    type MiningClaimsHardwareClaimAmount = u64;
    type MiningClaimsHardwareIndex = u64;
}

impl mining_execution_token::Config for Runtime {
    type Event = Event;
    type MiningExecutionTokenIndex = u64;
}

impl exchange_rate::Config for Runtime {
    type DOTRate = u64;
    type DecimalsAfterPoint = u32;
    type Event = Event;
    type ExchangeRateIndex = u64;
    type FILRate = u64;
    type HBTCRate = u64;
    type IOTARate = u64;
}

impl membership_supernodes::Config for Runtime {
    type Event = Event;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 1000;
	pub const MinCandidates: u32 = 5;
	pub const SessionLength: BlockNumber = 6 * HOURS;
	pub const MaxInvulnerables: u32 = 100;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

// We allow root only to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;

impl pallet_collator_selection::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type UpdateOrigin = CollatorSelectionUpdateOrigin;
    type PotId = PotId;
    type MaxCandidates = MaxCandidates;
    type MinCandidates = MinCandidates;
    type MaxInvulnerables = MaxInvulnerables;
    // should be a multiple of session or things will get inconsistent
    type KickThreshold = Period;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ValidatorRegistration = Session;
    type WeightInfo = ();
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        ParachainSystem: cumulus_pallet_parachain_system::{
            Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
        } = 1,
        Utility: pallet_utility = 2,
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 3,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 4,
        Scheduler: pallet_scheduler,
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 5,
        ParachainInfo: parachain_info::{Pallet, Storage, Config} = 6,
        Indices: pallet_indices = 7,

        // Monetary stuff.
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 11,

        // Collator support. The order of these 4 are important and shall not change.
        Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
        CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
        Democracy: pallet_democracy = 23,
        Aura: pallet_aura::{Pallet, Storage, Config<T>} = 24,
        AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 25,

        // XCM helpers.
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 30,
        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin, Config} = 31,
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 32,
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 33,

        Council: pallet_collective::<Instance1>,
        TechnicalCommittee: pallet_collective::<Instance2>,
        Elections: pallet_elections_phragmen,
        TechnicalMembership: pallet_membership::<Instance1>,
        // GeneralCouncil: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        // GeneralCouncilMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>},
        PalletTreasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>},
        Tips: pallet_tips,
        // Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>},
        MembershipSupernodes: membership_supernodes::{Pallet, Call, Storage, Event<T>},
        RoamingOperators: roaming_operators::{Pallet, Call, Storage, Event<T>},
        RoamingNetworks: roaming_networks::{Pallet, Call, Storage, Event<T>},
        RoamingOrganizations: roaming_organizations::{Pallet, Call, Storage, Event<T>},
        RoamingNetworkServers: roaming_network_servers::{Pallet, Call, Storage, Event<T>},
        RoamingDevices: roaming_devices::{Pallet, Call, Storage, Event<T>},
        RoamingRoutingProfiles: roaming_routing_profiles::{Pallet, Call, Storage, Event<T>},
        RoamingServiceProfiles: roaming_service_profiles::{Pallet, Call, Storage, Event<T>},
        RoamingAccountingPolicies: roaming_accounting_policies::{Pallet, Call, Storage, Event<T>},
        RoamingAgreementPolicies: roaming_agreement_policies::{Pallet, Call, Storage, Event<T>},
        RoamingNetworkProfiles: roaming_network_profiles::{Pallet, Call, Storage, Event<T>},
        RoamingDeviceProfiles: roaming_device_profiles::{Pallet, Call, Storage, Event<T>},
        RoamingSessions: roaming_sessions::{Pallet, Call, Storage, Event<T>},
        RoamingBillingPolicies: roaming_billing_policies::{Pallet, Call, Storage, Event<T>},
        RoamingChargingPolicies: roaming_charging_policies::{Pallet, Call, Storage, Event<T>},
        RoamingPacketBundles: roaming_packet_bundles::{Pallet, Call, Storage, Event<T>},
        MiningSettingToken: mining_setting_token::{Pallet, Call, Storage, Event<T>},
        MiningSettingHardware: mining_setting_hardware::{Pallet, Call, Storage, Event<T>},
        MiningRatesToken: mining_rates_token::{Pallet, Call, Storage, Event<T>},
        MiningRatesHardware: mining_rates_hardware::{Pallet, Call, Storage, Event<T>},
        MiningSamplingToken: mining_sampling_token::{Pallet, Call, Storage, Event<T>},
        MiningSamplingHardware: mining_sampling_hardware::{Pallet, Call, Storage, Event<T>},
        MiningEligibilityToken: mining_eligibility_token::{Pallet, Call, Storage, Event<T>},
        MiningEligibilityHardware: mining_eligibility_hardware::{Pallet, Call, Storage, Event<T>},
        MiningEligibilityProxy: mining_eligibility_proxy::{Pallet, Call, Storage, Event<T>},
        MiningClaimsToken: mining_claims_token::{Pallet, Call, Storage, Event<T>},
        MiningClaimsHardware: mining_claims_hardware::{Pallet, Call, Storage, Event<T>},
        MiningExecutionToken: mining_execution_token::{Pallet, Call, Storage, Event<T>},
        ExchangeRate: exchange_rate::{Pallet, Call, Storage, Event<T>},
    }
);

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_collator_selection, CollatorSelection]
	);
}

impl_runtime_apis! {
    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities().into_inner()
        }
    }

    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }

    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
        	SessionKeys::generate(seed)
        }

        fn decode_session_keys(
        	encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
        	SessionKeys::decode_into_raw_public_keys(&encoded)
        }
	}

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    // TODO - do we need this in parachain or only standalone chain?
    // impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
    //     fn authorities() -> Vec<AuthorityDiscoveryId> {
    //         AuthorityDiscovery::authorities()
    //     }
    // }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
    }


	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade parachain-template.");
			let weight = Executive::try_runtime_upgrade().unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block_no_check(block: Block) -> Weight {
			Executive::execute_block_no_check(block)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
