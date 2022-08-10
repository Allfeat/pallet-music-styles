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
use sp_core::H256;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};
pub use types::*;

impl<T: Config> Contains<H256> for Pallet<T> {
    fn contains(t: &H256) -> bool {
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
        /// A new music style has been added
        StyleAdded(StyleType<T>),
        /// A new sub style has been added to parent (parent, new_sub_style)
        SubStyleAdded(H256, SubStyleType<T>),
        /// A style name has been updated (old, new)
        StyleNameUpdated(H256, Vec<u8>),
        /// A music style has been removed
        StyleRemoved(StyleType<T>),
        /// A sub style has been removed from parent (parent, sub_style)
        SubStyleRemoved(SubStyleType<T>),
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
        /// Something goes wrong when updating on-chain data
        MutationError,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The existing music styles at the genesis
        pub styles: Vec<(Vec<u8>, Option<Vec<Vec<u8>>>)>,
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

            for (input_name, input_sub_styles) in &self.styles {
                let style = Pallet::<T>::try_create_style(input_name, input_sub_styles).unwrap();

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

            let style = Self::try_create_style(&name, &sub)?;

            <Styles<T>>::try_append(style.clone()).map_err(|_| Error::<T>::StylesCapacity)?;

            Self::deposit_event(Event::StyleAdded(style));

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn add_sub_style(
            origin: OriginFor<T>,
            parent_id: H256,
            name: Vec<u8>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let mut added: Option<SubStyleType<T>> = None;

            <Styles<T>>::try_mutate(|styles| -> DispatchResult {
                let parent_style = styles
                    .iter_mut()
                    .find(|style| &style.id == &parent_id)
                    .ok_or_else(|| Error::<T>::StyleNotFound)?;

                let new_sub_style = Self::try_create_sub_style(&name, &parent_id)?;

                added = Some(new_sub_style.clone());

                parent_style
                    .sub_styles
                    .try_push(new_sub_style)
                    .map_err(|_| Error::<T>::StylesCapacity)?;

                Ok(())
            })?;

            let added = added.ok_or_else(|| Error::<T>::MutationError)?;

            Self::deposit_event(Event::SubStyleAdded(parent_id, added));

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn update_style_name(
            origin: OriginFor<T>,
            id: H256,
            new_name: Vec<u8>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let bounded_name = Self::unwrap_name(&new_name)?;
            let mut styles = <Styles<T>>::get();

            let style_kind = Self::get_style(id, &styles);

            match style_kind {
                StyleKind::MainStyle(style) => {
                    // Check if the name is free at main styles level
                    match &styles.iter().find(|s| s.name == bounded_name) {
                        Some(_) => Err(Error::<T>::NameAlreadyExists)?,
                        None => match styles.iter_mut().find(|s| s.id == style.id) {
                            Some(s) => s.name = bounded_name,
                            None => Err(Error::<T>::StyleNotFound)?,
                        },
                    }
                }
                StyleKind::SubStyle(sub_style) => {
                    // Get the parent style to check is the new name is not already used in sub styles.
                    match styles.iter_mut().find(|s| s.id == sub_style.parent_id) {
                        Some(parent_style) => {
                            if let Some(_) = parent_style
                                .sub_styles
                                .iter()
                                .find(|s| s.name == bounded_name)
                            {
                                return Err(Error::<T>::NameAlreadyExists)?;
                            }

                            match parent_style.sub_styles.iter_mut().find(|s| s.id == id) {
                                Some(s) => s.name = bounded_name,
                                None => Err(Error::<T>::StyleNotFound)?,
                            }
                        }
                        None => Err(Error::<T>::StyleNotFound)?,
                    }
                }
                StyleKind::None => Err(Error::<T>::StyleNotFound)?,
            };

            <Styles<T>>::put(styles);

            Self::deposit_event(Event::StyleNameUpdated(id, new_name));

            Ok(())
        }

        /// Remove a sub style or a style (and its own sub styles)
        #[pallet::weight(0)]
        pub fn remove(origin: OriginFor<T>, id: H256) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let mut styles = <Styles<T>>::get();
            let style_kind = Self::get_style(id, &styles);

            let removed: StyleKind<T> = match style_kind {
                StyleKind::MainStyle(style) => {
                    let position = styles
                        .iter()
                        .position(|s| &s.id == &style.id)
                        .ok_or_else(|| Error::<T>::StyleNotFound)?;

                    StyleKind::MainStyle(styles.remove(position))
                }
                StyleKind::SubStyle(sub_style) => {
                    let style = styles
                        .iter_mut()
                        .find(|s| s.id == sub_style.parent_id)
                        .ok_or_else(|| Error::<T>::StyleNotFound)?;

                    // Find the position of the element to delete
                    let remove_position = style
                        .sub_styles
                        .iter()
                        .position(|s| s.id == id)
                        .ok_or_else(|| Error::<T>::StyleNotFound)?;

                    StyleKind::SubStyle(style.sub_styles.remove(remove_position))
                }
                StyleKind::None => Err(Error::<T>::StyleNotFound)?,
            };

            <Styles<T>>::put(styles);

            match removed {
                StyleKind::MainStyle(s) => {
                    Self::deposit_event(Event::StyleRemoved(s));
                }
                StyleKind::SubStyle(s) => {
                    Self::deposit_event(Event::SubStyleRemoved(s));
                }
                StyleKind::None => Err(Error::<T>::StyleNotFound)?,
            }

            Ok(())
        }
    }
}
