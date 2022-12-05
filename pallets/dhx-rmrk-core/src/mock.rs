use crate as dhx_rmrk_core;
use frame_support::{parameter_types, traits::{ConstU32, SortedMembers}};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use system::EnsureSignedBy;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Uniques: pallet_uniques::{Pallet, Call, Event<T>},
        RmrkCore: pallet_rmrk_core::{Pallet, Call, Event<T>},
        DhxRmrkCore: dhx_rmrk_core::{Pallet, Call},
        Balances: pallet_balances::{Pallet, Call, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
    pub const ExistentialDeposit: Balance = 1;
}

type AccountId = u64;
type Balance = u64;
impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<2>;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Pallet<Test>;
    type MaxLocks = ();
    type WeightInfo = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

parameter_types! {
	pub const CollectionDeposit: Balance = 10 ;
	pub const ItemDeposit: Balance = 100;
	pub const KeyLimit: u32 = 32;
	pub const ValueLimit: u32 = 256;
	pub const UniquesMetadataDepositBase: Balance = 10;
	pub const AttributeDepositBase: Balance = 10;
	pub const DepositPerByte: Balance = 1;
	pub const UniquesStringLimit: u32 = 128;
	pub const MaxPropertiesPerTheme: u32 = 100;
	pub const MaxCollectionsEquippablePerPart: u32 = 100;
}

impl pallet_uniques::Config for Test {
    type Event = Event;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<system::EnsureSigned<AccountId>>;
	type Locker = pallet_rmrk_core::Pallet<Test>;
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = UniquesMetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = UniquesStringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type WeightInfo = ();

}

impl pallet_rmrk_core::Config for Test {
	type Event = Event;
	type ProtocolOrigin = frame_system::EnsureRoot<AccountId>;
	type ResourceSymbolLimit = ResourceSymbolLimit;
	type PartsLimit = PartsLimit;
	type MaxPriorities = MaxPriorities;
	type CollectionSymbolLimit = CollectionSymbolLimit;
	type MaxResourcesOnMint = MaxResourcesOnMint;
	type NestingBudget = NestingBudget;
	type WeightInfo = pallet_rmrk_core::weights::SubstrateWeight<Test>;
}

pub const ALLOWED_MINTERS: &[AccountId] = &[1, 2, 3];

parameter_types! {
	pub const ResourceSymbolLimit: u32 = 10;
	pub const PartsLimit: u32 = 25;
	pub const MaxPriorities: u32 = 25;
	pub const CollectionSymbolLimit: u32 = 100;
	pub const MaxResourcesOnMint: u32 = 100;
	pub const NestingBudget: u32 = 20;
	pub AllowedMinters: Vec<AccountId> = ALLOWED_MINTERS.to_vec();
}

impl SortedMembers<AccountId> for AllowedMinters {
	fn sorted_members() -> Vec<AccountId> {
		AllowedMinters::get()
	}
}

impl dhx_rmrk_core::Config for Test {
    type ProducerOrigin = EnsureSignedBy<AllowedMinters, AccountId>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
