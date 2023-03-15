use crate as crowdloan_reward;
use crate::types;
use frame_support::{
    parameter_types,
    traits::{
        ConstU16,
        ConstU64,
        Currency,
        Imbalance,
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
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockHashCount = ConstU64<250>;
    type BlockLength = ();
    type BlockNumber = BlockNumber;
    type BlockWeights = ();
    type Call = Call;
    type DbWeight = ();
    type Event = Event;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type Origin = Origin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ConstU16<42>;
    type SystemWeightInfo = ();
    type Version = ();
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
    pub const VestingMinTransfer: Balance = 1000;
}

impl pallet_balances::Config for Test {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_vesting::Config for Test {
    type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
    type Currency = Balances;
    type Event = Event;
    type MinVestedTransfer = VestingMinTransfer;
    type WeightInfo = ();

    const MAX_VESTING_SCHEDULES: u32 = 20;
}

impl crowdloan_reward::Config for Test {
    type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
    type CampaignId = u32;
    type Currency = Balances;
    type CurrencyConvert = sp_runtime::traits::ConvertInto;
    type Event = Event;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn run_to_block(n: types::BlockNumberOf<Test>) {
    use frame_support::traits::Hooks;

    while System::block_number() < n {
        if System::block_number() > 1 {
            Reward::on_finalize(System::block_number());
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);

        System::on_initialize(System::block_number());
        Reward::on_initialize(System::block_number());
    }
}

pub fn credit_account<T: crate::Config>(account: &AccountId, amount: Balance) {
    assert_eq!(<Test as crate::Config>::Currency::deposit_creating(account, amount).peek(), amount)
}

pub fn reward_events() -> Vec<crate::Event<Test>> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::Reward(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}
