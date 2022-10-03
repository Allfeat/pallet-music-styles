use super::*;
use allfeat_support::types::music::style::MusicSubStyles;

impl<T: Config> Pallet<T> {
    pub(super) fn to_bounded_style(value: Vec<u8>) -> Result<MusicStyleName, DispatchError> {
        Ok(value.try_into().map_err(|_| Error::<T>::NameTooLong)?)
    }

    pub(super) fn to_bounded_sub_styles(
        value: Vec<Vec<u8>>,
    ) -> Result<MusicSubStyles, DispatchError> {
        let mut subs: MusicSubStyles = Default::default();
        for sub in value {
            subs.try_push(MusicStyleName::try_from(sub).map_err(|_| Error::<T>::NameTooLong)?)
                .map_err(|_| Error::<T>::StylesCapacity)?
        }
        Ok(subs)
    }

    pub(super) fn checked_add_subs(
        tree: &mut MusicStyleDB,
        subs: MusicSubStyles,
        into: MusicStyleName,
    ) -> DispatchResult {
        match tree.get_mut(&into) {
            Some(s) => {
                for sub in subs.iter() {
                    if s.contains(sub) {
                        return Err(Error::<T>::NameAlreadyExists)?;
                    } else {
                        s.try_push(sub.clone())
                            .map_err(|_| Error::<T>::StylesCapacity)?;
                    }
                }

                Ok(())
            }
            // No existing sub style, we can insert without any check
            None => {
                tree.try_insert(into, subs.clone())
                    .map_err(|_| Error::<T>::StylesCapacity)?;

                Ok(())
            }
        }
    }
}
