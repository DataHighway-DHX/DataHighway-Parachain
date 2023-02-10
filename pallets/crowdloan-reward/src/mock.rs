use crate as crowdloan_reward;
use frame_support::{traits::{ConstU16, ConstU64}, parameter_types};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;
type AccountId = u64;
type BlockNumber = u64;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>},
        Reward: crowdloan_reward::{Pallet, Call, Storage, Event<T> },
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
	pub const VestingMinTransfer: Balance = 1000;
}

impl pallet_balances::Config for Test {
 	type Balance = Balance;
 	type DustRemoval = ();
 	type Event = Event;
 	type ExistentialDeposit = ExistentialDeposit;
 	type AccountStore = System;
 	type WeightInfo = ();
 	type MaxLocks = MaxLocks;
 	type MaxReserves = ();
 	type ReserveIdentifier = [u8; 8];
}

impl pallet_vesting::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
    type MinVestedTransfer = VestingMinTransfer;
    type WeightInfo = ();
    const MAX_VESTING_SCHEDULES:u32 = 20;
}

impl crowdloan_reward::Config for Test {
	type Event = Event;
    type Currency = Balances;
    type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
    type CrowdloanId = u32;
    type CurrencyConvert = sp_runtime::traits::ConvertInto;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}