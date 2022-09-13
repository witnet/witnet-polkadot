use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64, GenesisBuild},
};
use sp_runtime::testing::Header;
use sp_runtime::traits::IdentityLookup;

use sp_core::H256;

use crate as pallet_witnet_oracle;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        Witnet: pallet_witnet_oracle,
    }
);

parameter_types! {
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type Balance = u64;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ();
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

pub const MAX_WITNET_BYTE_SIZE: u16 = 2048;

parameter_types! {
    pub const MaxWitnetByteSize: u16 = MAX_WITNET_BYTE_SIZE;
}

impl pallet_witnet_oracle::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type MaxByteSize = MaxWitnetByteSize;
    type TimeProvider = pallet_timestamp::Pallet<Test>;
}

#[derive(Default)]
pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        // Account #5 will be pre-approved as an operator
        let operators = vec![5];
        // Fund all accounts in [0, 10) with a balance of 1_000
        let balances = (0..10).map(|i| (i, 1_000)).collect::<Vec<_>>();

        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        pallet_balances::GenesisConfig::<Test> { balances }
            .assimilate_storage(&mut t)
            .unwrap();
        pallet_witnet_oracle::GenesisConfig::<Test>::from_operators(operators)
            .assimilate_storage(&mut t)
            .unwrap();
        t.into()
    }

    pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
        self.build().execute_with(|| {
            System::set_block_number(1);
            test()
        })
    }
}
