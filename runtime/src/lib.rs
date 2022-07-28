#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod xcm_config;

use frame_support::weights::ConstantMultiplier;
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_inherents::{
    CheckInherentsResult,
    InherentData,
};
use sp_runtime::{
    create_runtime_str, curve::PiecewiseLinear, generic, impl_opaque_keys, traits,
    traits::{AccountIdLookup, BlakeTwo256, BlindCheckable, Block as BlockT, Convert, ConvertInto, Checkable, IdentifyAccount,
        IdentityLookup, NumberFor, OpaqueKeys, Saturating, StaticLookup, SaturatedConversion, Verify},
    transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
        ApplyExtrinsicResult, MultiSignature, FixedPointNumber,
};
pub use sp_runtime::{MultiAddress, Perbill, Percent, Permill, Perquintill};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;

use codec::{Decode, Encode, MaxEncodedLen};
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{
        ConstU8, ConstU16, ConstU32, ConstU64, ConstU128, Currency, EnsureOneOf, EqualPrivilegeOnly,
        Everything, Imbalance, InstanceFilter, Contains, ContainsLengthBound, OnUnbalanced, KeyOwnerProofSystem,
        LockIdentifier, Randomness, OnRuntimeUpgrade, StorageInfo, U128CurrencyToVote,
    },
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        DispatchClass, IdentityFee, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
    },
    PalletId,
    RuntimeDebug,
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
use polkadot_runtime_common::{BlockHashCount as BlockHashCountCommon, SlowAdjustingFeeUpdate};

// XCM Imports
use xcm::latest::prelude::BodyId;
use xcm_executor::XcmExecutor;

pub use module_primitives::{
    constants::currency::{
        CENTS,
        deposit,
        DOLLARS,
        EXISTENTIAL_DEPOSIT,
        MICROUNITS,
        MILLIUNITS,
        MILLICENTS,
        UNITS,
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
        AccountId,
        Amount,
        Balance,
        BlockNumber,
        DigestItem,
        Hash,
        Index,
        Moment,
        Signature,
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
    pallet_asset_tx_payment::ChargeAssetTxPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
    frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem>;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        // in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNITS:
        // in our template, we map to 1/10 of that, or 1/10 MILLIUNITS
        let p = MILLIUNITS / 10;
        let q = 100 * Balance::from(ExtrinsicBaseWeight::get());
        smallvec![WeightToFeeCoefficient {
        	degree: 1,
        	negative: false,
        	coeff_frac: Perbill::from_rational(p % q, q),
        	coeff_integer: p / q,
        }]
    }
}

// https://docs.substrate.io/v3/tools/try-runtime/#helper-functions
pub struct EnsureAccountsWontDie;
impl OnRuntimeUpgrade for EnsureAccountsWontDie {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() {
        let account_count = frame_system::Accounts::<Runtime>::iter().count();
        Self::set_temp_storage(account_count, "account_count");
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() {
        // ensure that this migration doesn't kill any account.
        let post_migration = frame_system::Accounts::<Runtime>::iter().count();
        let pre_migration = Self::get_temp_storage::<u32>("account_count");
        ensure!(post_migration == pre_migration, "error ...");
    }
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
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
    }
}

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;
pub use impls::{Author, CreditToBlockAuthor};

use sp_runtime::generic::Era;

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("datahighway-parachain"),
    impl_name: create_runtime_str!("datahighway-parachain"),
    authoring_version: 2,
    spec_version: 4,
    impl_version: 1,
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

pub const BLOCK_HASH_COUNT_AS_CONST: BlockNumber = BlockHashCountCommon::get();
pub const SS58_PREFIX_AS_CONST: u16 = 33;
pub const MAX_CONSUMERS_AS_CONST: u32 = 16;

parameter_types! {
    pub const BlockHashCount: BlockNumber = BLOCK_HASH_COUNT_AS_CONST;
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
    pub const SS58Prefix: u16 = SS58_PREFIX_AS_CONST;
    pub const MaxConsumers: u32 = MAX_CONSUMERS_AS_CONST;
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
    type Lookup = AccountIdLookup<AccountId, AccountIndex>;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    type Event = Event;
    type BlockHashCount = ConstU32<BLOCK_HASH_COUNT_AS_CONST>;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type SS58Prefix = ConstU16<SS58_PREFIX_AS_CONST>;
    type MaxConsumers = ConstU32<MAX_CONSUMERS_AS_CONST>;
}

pub const MAX_AUTHORITIES_AS_CONST: u32 = 100;

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
    type EventHandler = (CollatorSelection,);
}

pub const MAX_LOCKS_AS_CONST: u32 = 50;
pub const EXISTENTIAL_DEPOSIT_AS_CONST: Balance = EXISTENTIAL_DEPOSIT;

parameter_types! {
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT_AS_CONST;
    pub const MaxLocks: u32 = MAX_LOCKS_AS_CONST;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT_AS_CONST>;
    type MaxLocks = ConstU32<MAX_LOCKS_AS_CONST>;
    type MaxReserves = MaxReserves;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type ReserveIdentifier = [u8; 8];
}

pub const OPERATIONAL_FEE_MULTIPLIER_AS_CONST: u8 = 5;

parameter_types! {
    /// Relay Chain `TransactionByteFee` / 10
    // Note: Kusama relay chain's `TransactionByteFee` is 10 * MILLICENTS,
    // so 1/10 of that is 1 * MILLICENTS
    pub const TransactionByteFee: Balance = 1 * MILLICENTS;
    pub const OperationalFeeMultiplier: u8 = OPERATIONAL_FEE_MULTIPLIER_AS_CONST;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate =
        TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
    type OperationalFeeMultiplier = ConstU8<OPERATIONAL_FEE_MULTIPLIER_AS_CONST>;
}

impl pallet_asset_tx_payment::Config for Runtime {
    type Fungibles = Assets;
    type OnChargeAssetTransaction = pallet_asset_tx_payment::FungiblesAdapter<
        pallet_assets::BalanceToAssetBalance<Balances, Runtime, ConvertInto>,
        CreditToBlockAuthor,
    >;
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
    pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
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
    pub const BountyCuratorDepositMin: Balance = 10 * DOLLARS;
    pub const BountyCuratorDepositMax: Balance = 500 * DOLLARS;
    pub const BountyDepositBase: Balance = 1 * DOLLARS;
    pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
    pub const BountyUpdatePeriod: BlockNumber = 7 * DAYS;
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
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
    >;
    type RejectOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
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

impl pallet_bounties::Config for Runtime {
    type Event = Event;
    type BountyDepositBase = BountyDepositBase;
    type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
    type BountyUpdatePeriod = BountyUpdatePeriod;
    type BountyValueMinimum = BountyValueMinimum;
    type DataDepositPerByte = DataDepositPerByte;
    type CuratorDepositMin = BountyCuratorDepositMin;
    type CuratorDepositMax = BountyCuratorDepositMax;
    type CuratorDepositMultiplier = BountyCuratorDeposit;
    type MaximumReasonLength = MaximumReasonLength;
    type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
    type ChildBountyManager = ChildBounties;
}

impl pallet_child_bounties::Config for Runtime {
    type Event = Event;
    type MaxActiveChildBountyCount = MaxActiveChildBountyCount;
    type ChildBountyValueMinimum = ChildBountyValueMinimum;
    type WeightInfo = pallet_child_bounties::weights::SubstrateWeight<Runtime>;
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
        let period =
            BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
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
            pallet_asset_tx_payment::ChargeAssetTxPayment::<Runtime>::from(tip, None),
        );
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                log::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
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

parameter_types! {
    // matches Kusama
    pub const Period: BlockNumber = 10 * MINUTES;
    pub const Offset: BlockNumber = 0;
    pub const MaxAuthorities: u32 = MAX_AUTHORITIES_AS_CONST;
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
    type MaxAuthorities = ConstU32<MAX_AUTHORITIES_AS_CONST>;
}

parameter_types! {
    pub const BasicDeposit: Balance = 1 * DOLLARS;       // 258 bytes on-chain
    pub const FieldDeposit: Balance = 25 * CENTS;        // 66 bytes on-chain
    pub const SubAccountDeposit: Balance = 25 * CENTS;   // 53 bytes on-chain
    pub const MaxSubAccounts: u32 = 100;
    pub const MaxAdditionalFields: u32 = 100;
    pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type BasicDeposit = BasicDeposit;
    type FieldDeposit = FieldDeposit;
    type SubAccountDeposit = SubAccountDeposit;
    type MaxSubAccounts = MaxSubAccounts;
    type MaxAdditionalFields = MaxAdditionalFields;
    type MaxRegistrars = MaxRegistrars;
    type Slashed = Treasury;
    type ForceOrigin = EnsureRootOrHalfCouncil;
    type RegistrarOrigin = EnsureRootOrHalfCouncil;
    type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ConfigDepositBase: Balance = 1 * DOLLARS;
    pub const FriendDepositFactor: Balance = 50 * CENTS;
    pub const MaxFriends: u16 = 9;
    pub const RecoveryDeposit: Balance = 1 * DOLLARS;
}

impl pallet_recovery::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type ConfigDepositBase = ConfigDepositBase;
    type FriendDepositFactor = FriendDepositFactor;
    type MaxFriends = MaxFriends;
    type RecoveryDeposit = RecoveryDeposit;
    type WeightInfo = pallet_recovery::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const AssetDeposit: Balance = 100 * DOLLARS;
    pub const ApprovalDeposit: Balance = 1 * DOLLARS;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 10 * DOLLARS;
    pub const MetadataDepositPerByte: Balance = 1 * DOLLARS;
}

impl pallet_assets::Config for Runtime {
    type Event = Event;
    type Balance = u128;
    type AssetId = u32;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = ConstU128<DOLLARS>;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

pub const MAX_SIGNATORIES_AS_CONST: u16 = 100;

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u16 = MAX_SIGNATORIES_AS_CONST;
}

impl pallet_multisig::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = ConstU16<MAX_SIGNATORIES_AS_CONST>;
    type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    // One storage item; key size 32, value size 8; .
    pub const ProxyDepositBase: Balance = deposit(1, 8);
    // Additional storage item size of 33 bytes.
    pub const ProxyDepositFactor: Balance = deposit(0, 33);
    pub const AnnouncementDepositBase: Balance = deposit(1, 8);
    pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
}

/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode,
    RuntimeDebug, MaxEncodedLen, scale_info::TypeInfo,
)]
pub enum ProxyType {
    Any,
    NonTransfer,
    Governance,
}
impl Default for ProxyType {
    fn default() -> Self {
        Self::Any
    }
}
impl InstanceFilter<Call> for ProxyType {
    fn filter(&self, c: &Call) -> bool {
        match self {
            ProxyType::Any => true,
            ProxyType::NonTransfer => !matches!(
                c,
                Call::Balances(..) |
                Call::Indices(pallet_indices::Call::transfer { .. })
            ),
            ProxyType::Governance => matches!(
                c,
                Call::Democracy(..) |
                Call::Council(..) |
                Call::TechnicalCommittee(..) |
                Call::Elections(..) | Call::Treasury(..)
            ),
        }
    }
    fn is_superset(&self, o: &Self) -> bool {
        match (self, o) {
            (x, y) if x == y => true,
            (ProxyType::Any, _) => true,
            (_, ProxyType::Any) => false,
            (ProxyType::NonTransfer, _) => true,
            _ => false,
        }
    }
}

impl pallet_proxy::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type ProxyType = ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = ConstU32<32>;
    type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
    type MaxPending = ConstU32<32>;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

pub const MAX_SCHEDULED_PER_BLOCK_AS_CONST: u32 = 50;

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
        RuntimeBlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = MAX_SCHEDULED_PER_BLOCK_AS_CONST;
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
    type MaxScheduledPerBlock = ConstU32<MAX_SCHEDULED_PER_BLOCK_AS_CONST>;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type PreimageProvider = Preimage;
    type NoPreimagePostponement = NoPreimagePostponement;
}

parameter_types! {
    pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub const PreimageBaseDeposit: Balance = 1 * DOLLARS;
    // One cent: $10,000 / MB
    pub const PreimageByteDeposit: Balance = 1 * CENTS;
}

impl pallet_preimage::Config for Runtime {
    type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
    type Event = Event;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type MaxSize = PreimageMaxSize;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
}


parameter_types! {
    pub const VoteLockingPeriod: BlockNumber = 7 * DAYS;
}

impl pallet_conviction_voting::Config for Runtime {
    type WeightInfo = pallet_conviction_voting::weights::SubstrateWeight<Self>;
    type Event = Event;
    type Currency = Balances;
    type VoteLockingPeriod = VoteLockingPeriod;
    type MaxVotes = ConstU32<512>;
    type MaxTurnout = frame_support::traits::TotalIssuanceOf<Balances, Self::AccountId>;
    type Polls = Referenda;
}

parameter_types! {
    pub const AlarmInterval: BlockNumber = 1;
    pub const SubmissionDeposit: Balance = 1 * DOLLARS;
    pub const UndecidingTimeout: BlockNumber = 7 * DAYS;
}

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
    type Id = u8;
    type Origin = <Origin as frame_support::traits::OriginTrait>::PalletsOrigin;
    fn tracks() -> &'static [(Self::Id, pallet_referenda::TrackInfo<Balance, BlockNumber>)] {
        static DATA: [(u8, pallet_referenda::TrackInfo<Balance, BlockNumber>); 1] = [(
            0u8,
            pallet_referenda::TrackInfo {
                name: "root",
                max_deciding: 1,
                decision_deposit: 10,
                prepare_period: 4,
                decision_period: 4,
                confirm_period: 2,
                min_enactment_period: 4,
                min_approval: pallet_referenda::Curve::LinearDecreasing {
                    begin: Perbill::from_percent(100),
                    delta: Perbill::from_percent(50),
                },
                min_turnout: pallet_referenda::Curve::LinearDecreasing {
                    begin: Perbill::from_percent(100),
                    delta: Perbill::from_percent(100),
                },
            },
        )];
        &DATA[..]
    }
    fn track_for(id: &Self::Origin) -> Result<Self::Id, ()> {
        if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
            match system_origin {
                frame_system::RawOrigin::Root => Ok(0),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}

impl pallet_referenda::Config for Runtime {
    type WeightInfo = pallet_referenda::weights::SubstrateWeight<Self>;
    type Call = Call;
    type Event = Event;
    type Scheduler = Scheduler;
    type Currency = pallet_balances::Pallet<Self>;
    type CancelOrigin = EnsureRoot<AccountId>;
    type KillOrigin = EnsureRoot<AccountId>;
    type Slash = ();
    type Votes = pallet_conviction_voting::VotesOf<Runtime>;
    type Tally = pallet_conviction_voting::TallyOf<Runtime>;
    type SubmissionDeposit = SubmissionDeposit;
    type MaxQueued = ConstU32<100>;
    type UndecidingTimeout = UndecidingTimeout;
    type AlarmInterval = AlarmInterval;
    type Tracks = TracksInfo;
}

pub const MAX_VOTES_AS_CONST: u32 = 100;

parameter_types! {
    pub const LaunchPeriod: BlockNumber = 7 * DAYS;
    pub const VotingPeriod: BlockNumber = 7 * DAYS;
    pub const FastTrackVotingPeriod: BlockNumber = 3 * DAYS;
    pub const InstantAllowed: bool = true;
    pub const MinimumDeposit: Balance = 1 * DOLLARS;
    pub const EnactmentPeriod: BlockNumber = 1 * DAYS;
    pub const CooloffPeriod: BlockNumber = 7 * DAYS;
    pub const MaxVotes: u32 = MAX_VOTES_AS_CONST;
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
    type ExternalOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
    /// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;
    /// A unanimous council can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
    /// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>;
    type InstantOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>;
    type InstantAllowed = frame_support::traits::ConstBool<true>;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    // To cancel a proposal which has been passed, 2/3 of the council must agree to it.
    type CancellationOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
    // To cancel a proposal before it has been passed, the technical committee must be unanimous or
    // Root must agree.
    type CancelProposalOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>,
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
    type MaxVotes = frame_support::traits::ConstU32<MAX_VOTES_AS_CONST>;
    type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
    type MaxProposals = MaxProposals;
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
    type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Runtime>;
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
        Identity: pallet_identity = 5,
        Recovery: pallet_recovery = 6,
        Scheduler: pallet_scheduler = 7,
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 8,
        ParachainInfo: parachain_info::{Pallet, Storage, Config} = 9,
        Indices: pallet_indices = 10,

        // Monetary stuff.
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 15,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Config} = 16,
        AssetTxPayment: pallet_asset_tx_payment = 17,

        // Collator support. The order of these 4 are important and shall not change.
        // Authorship must be before session in order to note author in the correct session and era
        // for im-online and staking.
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
        Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>},
        Bounties: pallet_bounties,
        ChildBounties: pallet_child_bounties,
        Tips: pallet_tips,
        Assets: pallet_assets,
        Preimage: pallet_preimage,
        Proxy: pallet_proxy,
        Multisig: pallet_multisig,
        Referenda: pallet_referenda,
        ConvictionVoting: pallet_conviction_voting,
    }
);

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_system, SystemBench::<Runtime>]
        [pallet_utility, Utility]
        [pallet_timestamp, Timestamp]
        [pallet_identity, Identity]
        [pallet_scheduler, Scheduler]
        [pallet_indices, Indices]
        [pallet_balances, Balances]
        [pallet_collator_selection, CollatorSelection]
        [pallet_democracy, Democracy]
        [pallet_aura, Aura]
        [pallet_xcm, PolkadotXcm]
        [pallet_collective::<Instance1>, Council]
        [pallet_collective::<Instance2>, TechnicalCommittee]
        [pallet_elections_phragmen, Elections]
        [pallet_membership::<Instance1>, TechnicalMembership]
        [pallet_treasury, Treasury]
        [pallet_bounties, Bounties]
        [pallet_child_bounties, ChildBounties]
        [pallet_tips, Tips]
        [pallet_preimage, Preimage]
        [pallet_proxy, Proxy]
        [pallet_multisig, Multisig]
        [pallet_referenda, Referenda]
        [pallet_conviction_voting, ConvictionVoting]
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
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade().unwrap();
            (weight, BlockWeights::get().max_block)
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
