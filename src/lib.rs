#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod functions;
use frame_support::{pallet_prelude::*, traits::Contains, BoundedVec};
use frame_system::pallet_prelude::*;
pub use functions::*;
pub use pallet::*;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

// Helper types
pub type StyleName<T> = BoundedVec<u8, <T as Config>::NameMaxLength>;
pub type SubList<T> = BoundedVec<StyleName<T>, <T as Config>::MaxSubStyles>;

impl<T: Config> Contains<StyleName<T>> for Pallet<T> {
    fn contains(t: &StyleName<T>) -> bool {
        let styles = <Styles<T>>::get();

        // Search in main styles
        if styles.iter().find(|&s| s == t).is_some() {
            return true;
        }

        // Search in sub-styles
        for style in styles.iter() {
            let sub = <SubStyles<T>>::get(style);
            if sub.iter().find(|&s| s == t).is_some() {
                return true;
            }
        }

        return false;
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

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

        /// The maximum length of a sub music style per style
        #[pallet::constant]
        type MaxSubStyles: Get<u32>;

        /// The maximum length of a music style name
        #[pallet::constant]
        type NameMaxLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn get)]
    pub(super) type Styles<T: Config> =
        StorageValue<_, BoundedVec<StyleName<T>, T::MaxStyles>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_sub_style)]
    pub(super) type SubStyles<T: Config> = StorageMap<
        _,
        Twox64Concat,
        StyleName<T>,
        BoundedVec<StyleName<T>, T::MaxSubStyles>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new music style have been added
        Added(Vec<u8>, Option<Vec<Vec<u8>>>),
        /// A music style have been removed
        /// The second parameter is the removed sub style count
        Removed(Vec<u8>, u32),
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
        /// There is a duplicate style name in the given list
        DuplicatedStyle,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The existing music styles at the genesis
        pub styles: Vec<(Vec<u8>, Vec<Vec<u8>>)>,
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
            let mut main_styles: BTreeSet<StyleName<T>> = BTreeSet::new();

            for (input_name, input_sub_styles) in &self.styles {
                let name: StyleName<T> =
                    input_name.clone().try_into().expect("Style name too long");

                main_styles.insert(name.clone());

                let sub_styles = input_sub_styles
                    .iter()
                    .map(|n| n.clone().try_into().expect("Sub style name too long"))
                    .collect::<BTreeSet<StyleName<T>>>();

                assert_eq!(
                    input_sub_styles.len(),
                    sub_styles.len(),
                    "Music sub styles cannot contain duplicate names."
                );

                <SubStyles<T>>::insert(
                    name,
                    BoundedVec::try_from(btree_to_vec(sub_styles)).expect("Sub style max reached"),
                );
            }

            assert_eq!(
                self.styles.len(),
                main_styles.len(),
                "Music styles cannot contain duplicate names."
            );

            <Styles<T>>::put(
                BoundedVec::try_from(btree_to_vec(main_styles)).expect("Style max reached"),
            );
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add a new style
        /// Supports also sub styles
        #[pallet::weight(0)]
        pub fn add(
            origin: OriginFor<T>,
            name: Vec<u8>,
            sub: Option<Vec<Vec<u8>>>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let bounded_name = Self::unwrap_name(&name)?;
            let bounded_sub = Self::unwrap_new_sub(&sub)?;

            ensure!(
                !<Styles<T>>::get().contains(&bounded_name),
                Error::<T>::NameAlreadyExists
            );

            <Styles<T>>::try_append(&bounded_name).map_err(|_| Error::<T>::StorageFull)?;
            <SubStyles<T>>::insert(bounded_name, &bounded_sub);

            Self::deposit_event(Event::Added(name, sub));

            Ok(())
        }

        /// Remove a style and its own sub styles
        #[pallet::weight(0)]
        pub fn remove(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            // Find all related style and sub-styles
            let bounded_name = Self::unwrap_name(&name)?;
            let mut styles = <Styles<T>>::get();
            let sub_styles =
                <SubStyles<T>>::try_get(&bounded_name).map_err(|_| Error::<T>::StyleNotFound)?;

            // Search into <Style<T>> instead of Pallet::contains to
            // search into the first level styles only
            let position = styles
                .binary_search(&bounded_name)
                .map_err(|_| Error::<T>::StyleNotFound)?;

            <SubStyles<T>>::remove(bounded_name);

            let removed = styles.remove(position);

            <Styles<T>>::put(styles);

            Self::deposit_event(Event::Removed(removed.into(), sub_styles.len() as u32));

            Ok(())
        }
    }
}
