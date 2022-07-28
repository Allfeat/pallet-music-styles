#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use pallet::*;

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
        type NameMaxLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn get)]
    pub(super) type MusicStyles<T: Config> =
        StorageMap<_, Twox64Concat, u32, MusicStyle<BoundedVec<u8, T::NameMaxLength>>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn count)]
    pub(super) type MusicStyleCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new music style have been added
        Added(u32),
        /// A music style have been removed
        Removed(u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Music style too long
        NameTooLong,
        /// Music style not found
        MusicStyleNotFound,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        /// The existing music styles at the genesis
        pub music_styles: Vec<(u32, Vec<u8>)>,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                music_styles: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            for (index, name) in &self.music_styles {
                let music_style = MusicStyle {
                    name: name.clone().try_into().expect("Music style name too long"),
                };

                <MusicStyles<T>>::insert(index, music_style);
                <MusicStyleCount<T>>::put(index + 1);
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn add(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let index = <MusicStyleCount<T>>::get();
            let music_style = MusicStyle {
                name: name.try_into().map_err(|_| Error::<T>::NameTooLong)?,
            };

            <MusicStyles<T>>::insert(index, music_style);
            <MusicStyleCount<T>>::put(index + 1);

            Self::deposit_event(Event::Added(index));

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn remove(origin: OriginFor<T>, id: u32) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            ensure!(
                <MusicStyles<T>>::contains_key(&id),
                Error::<T>::MusicStyleNotFound
            );

            <MusicStyles<T>>::remove(&id);
            <MusicStyleCount<T>>::put(<MusicStyleCount<T>>::get() - 1);

            Self::deposit_event(Event::Removed(id));

            Ok(())
        }
    }
}
