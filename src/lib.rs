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
use sp_std::prelude::*;
pub use types::*;

impl<T: Config> Contains<Vec<u8>> for Pallet<T> {
    fn contains(t: &Vec<u8>) -> bool {
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

        /// The maximum storable music style count
        #[pallet::constant]
        type MaxStyleCount: Get<u32>;

        /// The maximum storable music sub style count
        #[pallet::constant]
        type MaxSubStyleCount: Get<u32>;

        /// The maximum length of a music style name
        #[pallet::constant]
        type NameMaxLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn get)]
    pub(super) type Styles<T: Config> = StorageValue<_, BoundedStyleList<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new music style has been added
        StyleAdded(StyleType<T>),
        /// A new sub style has been added to parent
        SubStyleAdded(SubStyleType<T>),
        /// A style name has been updated (old, new)
        StyleNameUpdated(StyleType<T>, StyleType<T>),
        /// A sub-style name has been updated (old, new)
        SubStyleNameUpdated(SubStyleType<T>, SubStyleType<T>),
        /// A music style has been removed
        StyleRemoved(StyleType<T>),
        /// A sub style has been removed from parent
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
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The existing music styles at the genesis
        pub styles: Vec<(Vec<u8>, Option<Vec<Vec<u8>>>)>,
        // Note: Use phantom data because we need a Generic in the GenesisConfig
        pub phantom: PhantomData<T>,
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
                let style =
                    Pallet::<T>::try_new_style(input_name.clone(), input_sub_styles.clone())
                        .unwrap();

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

            let style = Self::try_new_style(name, sub)?;

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

            let mut styles = <Styles<T>>::get();
            let new_sub_style = Self::try_new_sub_style(name, parent_id)?;

            styles
                .iter_mut()
                .find(|style| &style.id == &parent_id)
                .ok_or_else(|| Error::<T>::StyleNotFound)?
                .sub_styles
                .try_push(new_sub_style.clone())
                .map_err(|_| Error::<T>::StylesCapacity)?;

            <Styles<T>>::put(styles);

            Self::deposit_event(Event::SubStyleAdded(new_sub_style));

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn update_style_name(
            origin: OriginFor<T>,
            id: H256,
            new_name: Vec<u8>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let bounded_name = Self::try_into_bounded_name(new_name.clone())?;
            let mut styles = <Styles<T>>::get();
            let style_kind = Self::get_style(id, &styles);

            match style_kind {
                StyleKind::MainStyle(style) => {
                    // Check if the name is free at main styles level
                    if styles.iter().find(|s| s.name == bounded_name).is_some() {
                        return Err(Error::<T>::NameAlreadyExists)?;
                    }

                    // Get and mutate style
                    let mut style = styles
                        .iter_mut()
                        .find(|s| s.id == style.id)
                        .ok_or_else(|| Error::<T>::StyleNotFound)?;

                    let old_style = style.clone();
                    style.name = bounded_name;
                    let new_style = style.clone();

                    <Styles<T>>::put(styles);
                    Self::deposit_event(Event::StyleNameUpdated(old_style, new_style));
                }
                StyleKind::SubStyle(sub_style) => {
                    // Get the parent style
                    let parent_style = styles
                        .iter_mut()
                        .find(|s| s.id == sub_style.parent_id)
                        .ok_or_else(|| Error::<T>::StyleNotFound)?;

                    // check is the new name is not already used in sub styles.
                    if let Some(_) = parent_style
                        .sub_styles
                        .iter()
                        .find(|s| s.name == bounded_name)
                    {
                        return Err(Error::<T>::NameAlreadyExists)?;
                    }

                    // Get and mutate style
                    let sub_style = parent_style
                        .sub_styles
                        .iter_mut()
                        .find(|s| s.id == sub_style.id)
                        .ok_or_else(|| Error::<T>::StyleNotFound)?;

                    let old_sub_style = sub_style.clone();
                    sub_style.name = bounded_name;
                    let new_sub_style = sub_style.clone();

                    <Styles<T>>::put(styles);
                    Self::deposit_event(Event::SubStyleNameUpdated(old_sub_style, new_sub_style));
                }
                StyleKind::None => Err(Error::<T>::StyleNotFound)?,
            };

            Ok(())
        }

        /// Remove a sub style or a style (and its own sub styles)
        #[pallet::weight(0)]
        pub fn remove(origin: OriginFor<T>, id: H256) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let mut styles = <Styles<T>>::get();
            let style_kind = Self::get_style(id, &styles);

            match style_kind {
                StyleKind::MainStyle(style) => {
                    let position = styles
                        .iter()
                        .position(|s| &s.id == &style.id)
                        .ok_or_else(|| Error::<T>::StyleNotFound)?;

                    let removed = styles.remove(position);
                    <Styles<T>>::put(styles);
                    Self::deposit_event(Event::StyleRemoved(removed));
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

                    let removed = style.sub_styles.remove(remove_position);
                    <Styles<T>>::put(styles);
                    Self::deposit_event(Event::SubStyleRemoved(removed));
                }
                StyleKind::None => Err(Error::<T>::StyleNotFound)?,
            };

            Ok(())
        }
    }
}
