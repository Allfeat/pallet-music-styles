#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod functions;
mod types;
use frame_support::{pallet_prelude::*, traits::Contains, BoundedVec};
use frame_system::pallet_prelude::*;
pub use functions::*;
pub use pallet::*;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};
pub use types::*;

impl<T: Config> Contains<BoundedName<T>> for Pallet<T> {
    fn contains(t: &BoundedName<T>) -> bool {
        Self::contains(t)
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
    pub(super) type Styles<T: Config> = StorageValue<_, BoundedStyleList<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new music style have been added
        StyleAdded(Vec<u8>, Option<Vec<Vec<u8>>>),
        /// A new sub style have been added to parent (parent, new_sub_style)
        SubStyleAdded(Vec<u8>, Vec<u8>),
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
        StylesCapacity,
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
            let mut styles: BoundedStyleList<T> = BoundedVec::try_from(Vec::new()).unwrap();
            // Keys are used to check duplicates quickly
            let mut keys: Vec<BoundedName<T>> = Vec::new();

            for (input_name, input_sub_styles) in &self.styles {
                let name: BoundedName<T> =
                    input_name.clone().try_into().expect("Style name too long");

                if keys.contains(&name) {
                    panic!("Duplicate style name");
                } else {
                    keys.push(name.clone());
                }

                let mut sub_styles: BoundedNameList<T> = BoundedVec::try_from(Vec::new()).unwrap();

                for sub_name in input_sub_styles {
                    let name: BoundedName<T> =
                        sub_name.clone().try_into().expect("Style name too long");

                    if sub_styles.contains(&name) {
                        panic!("Duplicate sub style name");
                    }

                    sub_styles.try_push(name).expect("Sub style max reached");
                }

                let style = Style { name, sub_styles };

                styles.try_push(style).expect("Style max reached");
            }

            <Styles<T>>::put(styles);
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

            if Self::contains_primary_style(&bounded_name) {
                return Err(Error::<T>::NameAlreadyExists)?;
            }

            let style = Style {
                name: bounded_name,
                sub_styles: bounded_sub,
            };

            <Styles<T>>::try_append(style).map_err(|_| Error::<T>::StylesCapacity)?;

            Self::deposit_event(Event::StyleAdded(name, sub));

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn add_sub_style(
            origin: OriginFor<T>,
            parent: Vec<u8>,
            name: Vec<u8>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let bounded_name = Self::unwrap_name(&name)?;
            let bounded_parent = Self::unwrap_name(&parent)?;

            <Styles<T>>::try_mutate(|styles| -> DispatchResult {
                if let Some(index) = styles.iter().position(|style| style.name == bounded_parent) {
                    let style = styles
                        .get_mut(index)
                        .ok_or_else(|| Error::<T>::StyleNotFound)?;

                    if style.sub_styles.contains(&bounded_name) {
                        return Err(Error::<T>::NameAlreadyExists)?;
                    }

                    style
                        .sub_styles
                        .try_push(bounded_name)
                        .map_err(|_| Error::<T>::StylesCapacity)?;

                    Ok(())
                } else {
                    Err(Error::<T>::StyleNotFound.into())
                }
            })?;

            Self::deposit_event(Event::SubStyleAdded(parent, name));

            Ok(())
        }

        /// Remove a style and its own sub styles
        #[pallet::weight(0)]
        pub fn remove(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let bounded_name = Self::unwrap_name(&name)?;

            <Styles<T>>::try_mutate(|styles| -> DispatchResult {
                if let Some(index) = styles.iter().position(|style| &style.name == &bounded_name) {
                    styles.remove(index);
                    Ok(())
                } else {
                    Err(Error::<T>::StyleNotFound.into())
                }
            })?;

            Self::deposit_event(Event::Removed(name));

            Ok(())
        }
    }
}
