use crate::{
    self as pallet_music_styles,
    mock::sp_api_hidden_includes_construct_runtime::hidden_include::traits::GenesisBuild,
};
use frame_support::{
    traits::{ConstU16, ConstU64},
    {construct_runtime, parameter_types},
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

pub type AccountId = u64;
pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

// Test accounts used
// pub const ALICE: AccountId = 0; // Root
pub const BOB: AccountId = 1; // Regular user

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
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
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MaxStyles: u32 = 5;
    pub const NameMaxLength: u32 = 20;
}

impl pallet_music_styles::Config for Test {
    type Event = Event;
    type AdminOrigin = EnsureRoot<AccountId>;
    type MaxStyles = MaxStyles;
    type NameMaxLength = NameMaxLength;
}

// Build genesis storage according to the mock runtime.
pub(crate) fn new_test_ext(include_genesis: bool) -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let pallet_config: pallet_music_styles::GenesisConfig<Test> = match include_genesis {
        true => pallet_music_styles::GenesisConfig {
            styles: ["Reggae", "Rock", "Rap", "Pop"]
                .iter()
                .map(|style| style.as_bytes().to_vec())
                .collect(),
            phantom: Default::default(),
        },
        false => pallet_music_styles::GenesisConfig::default(),
    };

    pallet_config.assimilate_storage(&mut storage).unwrap();

    let mut ext: sp_io::TestExternalities = storage.into();

    ext.execute_with(|| System::set_block_number(1));
    ext
}