use crate as dhx_rmrk_core;
use frame_support::{
    parameter_types,
    traits::{
        ConstU32,
        GenesisBuild,
    },
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{
        BlakeTwo256,
        IdentityLookup,
    },
};

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
        DhxRmrkCore: dhx_rmrk_core::{Pallet, Call, Event<T>, Config<T>},
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
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockNumber = u64;
    type BlockWeights = ();
    type Call = Call;
    type DbWeight = ();
    type Event = Event;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = ConstU32<2>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type Origin = Origin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = SS58Prefix;
    type SystemWeightInfo = ();
    type Version = ();
}

impl pallet_balances::Config for Test {
    type AccountStore = frame_system::Pallet<Test>;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
    type WeightInfo = ();
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
    type AttributeDepositBase = AttributeDepositBase;
    type CollectionDeposit = CollectionDeposit;
    type CollectionId = u32;
    type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<system::EnsureSigned<AccountId>>;
    type Currency = Balances;
    type DepositPerByte = DepositPerByte;
    type Event = Event;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type ItemDeposit = ItemDeposit;
    type ItemId = u32;
    type KeyLimit = KeyLimit;
    type Locker = pallet_rmrk_core::Pallet<Test>;
    type MetadataDepositBase = UniquesMetadataDepositBase;
    type StringLimit = UniquesStringLimit;
    type ValueLimit = ValueLimit;
    type WeightInfo = ();
}

impl pallet_rmrk_core::Config for Test {
    type CollectionSymbolLimit = CollectionSymbolLimit;
    type Event = Event;
    type MaxPriorities = MaxPriorities;
    type MaxResourcesOnMint = MaxResourcesOnMint;
    type NestingBudget = NestingBudget;
    type PartsLimit = PartsLimit;
    type ProtocolOrigin = frame_system::EnsureRoot<AccountId>;
    type ResourceSymbolLimit = ResourceSymbolLimit;
    type WeightInfo = pallet_rmrk_core::weights::SubstrateWeight<Test>;
}

parameter_types! {
    pub const ResourceSymbolLimit: u32 = 10;
    pub const PartsLimit: u32 = 25;
    pub const MaxPriorities: u32 = 25;
    pub const CollectionSymbolLimit: u32 = 100;
    pub const MaxResourcesOnMint: u32 = 100;
    pub const NestingBudget: u32 = 20;
}

impl dhx_rmrk_core::Config for Test {
    type Event = Event;
}

pub const ALLOWED_MINTERS: &[AccountId] = &[1, 2, 3];

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    crate::GenesisConfig::<Test> {
        allowed_producers: ALLOWED_MINTERS.to_vec(),
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}
