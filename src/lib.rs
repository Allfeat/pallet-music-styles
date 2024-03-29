#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod functions;
mod impls;
pub mod weights;

use allfeat_support::prelude::*;
use allfeat_support::types::music::style::{MaxNameLength, MaxSubStyles};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use functions::*;
pub use pallet::*;
use sp_std::prelude::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Who can manage a music style list
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Weight information for extrinsics in this pallet.
        type Weights: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn get_styles)]
    pub(super) type Styles<T: Config> = StorageValue<_, MusicStyleDB, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new music style has been added
        StyleAdded(Vec<u8>),
        /// A new sub style has been added to parent
        SubStyleAdded(Vec<u8>),
        /// A style name has been updated (old, new)
        StyleNameUpdated(Vec<u8>, Vec<u8>),
        /// A sub-style name has been updated (old, new)
        SubStyleNameUpdated(Vec<u8>, Vec<u8>),
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
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// The existing music styles at the genesis
        pub styles: Vec<(Vec<u8>, Vec<Vec<u8>>)>,
        // Note: Use phantom data because we need a Generic in the GenesisConfig
        pub phantom: PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let mut styles = MusicStyleDB::new();

            for (input_name, input_sub_styles) in &self.styles {
                let parent = Pallet::<T>::to_bounded_style(input_name.clone()).unwrap();
                let subs = Pallet::<T>::to_bounded_sub_styles(input_sub_styles.clone()).unwrap();

                styles.try_insert(parent, subs).unwrap();
            }

            <Styles<T>>::put(styles);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add new styles
        /// Supports also adding sub styles into it at the same ime
        #[pallet::call_index(0)]
        #[pallet::weight(T::Weights::add_style(
            <MaxNameLength as Get<u32>>::get(),
            <MaxSubStyles as Get<u32>>::get()
        ))]
        pub fn add_style(
            origin: OriginFor<T>,
            name: Vec<u8>,
            sub: Option<Vec<Vec<u8>>>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let mut styles: MusicStyleDB = Self::get_styles();

            let parent_name = Self::to_bounded_style(name.clone())?;

            if styles.contains_key(&parent_name) {
                return Err(Error::<T>::NameAlreadyExists)?;
            } else {
                styles
                    .try_insert(parent_name.clone(), Default::default())
                    .map_err(|_| Error::<T>::StylesCapacity)?;
            }

            match sub {
                Some(ref subs) => {
                    let bounded_subs = Self::to_bounded_sub_styles(subs.clone())?;

                    Self::checked_add_subs(&mut styles, bounded_subs, parent_name)?;
                }
                // Not adding subs
                None => (),
            }

            <Styles<T>>::put(styles);

            // Emitting events
            Self::deposit_event(Event::StyleAdded(name));
            match sub {
                Some(subs) => {
                    for sub in subs {
                        Self::deposit_event(Event::SubStyleAdded(sub))
                    }
                }
                None => (),
            }

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::Weights::add_sub_style(
            <MaxNameLength as Get<u32>>::get(),
            <MaxSubStyles as Get<u32>>::get()
        ))]
        pub fn add_sub_style(
            origin: OriginFor<T>,
            parent_style: Vec<u8>,
            subs_style: Vec<Vec<u8>>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let mut styles: MusicStyleDB = Self::get_styles();

            let bounded_parent_style = Self::to_bounded_style(parent_style)?;

            if !styles.contains_key(&bounded_parent_style) {
                return Err(Error::<T>::StyleNotFound)?;
            }

            let bounded_subs = Self::to_bounded_sub_styles(subs_style.clone())?;

            Self::checked_add_subs(&mut styles, bounded_subs, bounded_parent_style)?;

            <Styles<T>>::put(styles);

            for sub in subs_style {
                Self::deposit_event(Event::SubStyleAdded(sub))
            }

            Ok(())
        }

        /*#[pallet::weight(0)]
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
        }*/
    }
}
