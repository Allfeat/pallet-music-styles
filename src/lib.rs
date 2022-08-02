#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{traits::Contains, BoundedVec};
pub use pallet::*;

type Style<NameMaxLength> = BoundedVec<u8, NameMaxLength>;

impl<T: Config> Contains<Vec<u8>> for Pallet<T> {
    fn contains(name: &Vec<u8>) -> bool {
        let bounded_name: BoundedVec<u8, T::NameMaxLength> = match name.clone().try_into() {
            Ok(x) => x,
            Err(_) => return false,
        };

        Self::get().binary_search(&bounded_name).is_ok()
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;

    /// Structure that holds the music style information that will be stored on-chain
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct MusicStyle<BoundedString> {
        pub name: BoundedString,
    }

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Who can manage an music style list
        type AdminOrigin: EnsureOrigin<Self::Origin>;

        /// The maximum length of a music style name
        #[pallet::constant]
        type MaxStyles: Get<u32>;

        /// The maximum length of a music style name
        #[pallet::constant]
        type NameMaxLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // Note:
    // - The outer BoundedVec could be replaced by a BoundedBTreeMap
    // to quickly check for double and vec length
    // - The inner BoundedVec (in Style) could be replaced by a
    // BoundedString to simplify public API
    #[pallet::storage]
    #[pallet::getter(fn get)]
    pub(super) type Styles<T: Config> =
        StorageValue<_, BoundedVec<Style<T::NameMaxLength>, T::MaxStyles>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new music style have been added
        Added(Vec<u8>),
        /// A music style have been removed
        Removed(Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Music style too long
        NameTooLong,
        /// Music style already exists
        NameAlreadyExists,
        /// Music style not found
        StyleNotFound,
        /// The music styles vec is full
        StorageFull,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The existing music styles at the genesis
        pub styles: Vec<Vec<u8>>,
        // Note: Use phantom data because we need a Generic
        // in the GenesisConfig and BlockNumber impl Default
        pub phantom: T::BlockNumber,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                styles: Default::default(),
                phantom: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            // Note: use BTreeSet to quickly catch duplicates
            use sp_std::collections::btree_set::BTreeSet;

            // Fill <Styles<T>>: StorageValue<Vec<Style>>
            let styles: Vec<BoundedVec<u8, T::NameMaxLength>> = self
                .styles
                .iter()
                .map(|x| x.clone().try_into().expect("Music style name too long"))
                .collect::<BTreeSet<BoundedVec<u8, T::NameMaxLength>>>()
                .iter()
                .cloned()
                .collect();

            let styles: BoundedVec<BoundedVec<u8, T::NameMaxLength>, T::MaxStyles> =
                styles.try_into().expect("Too many music styles");

            assert_eq!(
                styles.len(),
                self.styles.len(),
                "Music styles cannot contain duplicate names."
            );

            <Styles<T>>::put(styles.clone());
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn add(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let name: BoundedVec<u8, T::NameMaxLength> =
                name.try_into().map_err(|_| Error::<T>::NameTooLong)?;

            ensure!(
                !<Styles<T>>::get().contains(&name),
                Error::<T>::NameAlreadyExists
            );

            <Styles<T>>::try_append(&name).map_err(|_| Error::<T>::StorageFull)?;

            Self::deposit_event(Event::Added(name.into()));

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn remove(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let mut styles = <Styles<T>>::get();

            let name: BoundedVec<u8, T::NameMaxLength> =
                name.try_into().map_err(|_| Error::<T>::NameTooLong)?;

            let position = styles
                .binary_search(&name)
                .map_err(|_| Error::<T>::StyleNotFound)?;

            let removed = styles.remove(position);

            <Styles<T>>::put(styles);

            Self::deposit_event(Event::Removed(removed.into()));

            Ok(())
        }
    }
}
