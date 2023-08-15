use crate::{
    self as pallet_music_styles,
};
use frame_support::{
    construct_runtime,
    traits::{ConstU64},
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
    BuildStorage,
    traits::{ IdentityLookup},
};

pub type AccountId = u64;
pub type Block = frame_system::mocking::MockBlock<Test>;

// Test accounts used
// pub const ALICE: AccountId = 0; // Root
pub const BOB: AccountId = 1; // Regular user

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test
    {
        System: frame_system::{Pallet, Call, Storage, Event<T>},
        MusicStylesPallet: pallet_music_styles::{Pallet, Call, Storage, Event<T>},
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Nonce = u64;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_music_styles::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AdminOrigin = EnsureRoot<AccountId>;
    type Weights = ();
}

// Build genesis storage according to the mock runtime.
pub(crate) fn new_test_ext(include_genesis: bool) -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

    let pallet_config: pallet_music_styles::GenesisConfig<Test> = match include_genesis {
        true => pallet_music_styles::GenesisConfig {
            styles: vec![
                ("Raggae".into(), vec![]),
                (
                    "Rap".into(),
                    vec!["Drill".into(), "Trap".into(), "Hardcore".into()],
                ),
                ("Rock".into(), vec!["Hardcore".into()]),
            ],
            phantom: Default::default(),
        },
        false => pallet_music_styles::GenesisConfig::default(),
    };

    pallet_config.assimilate_storage(&mut storage).unwrap();

    let mut ext: sp_io::TestExternalities = storage.into();

    ext.execute_with(|| System::set_block_number(1));
    ext
}
