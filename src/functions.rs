use super::*;

impl<T: Config> Pallet<T> {
    pub(super) fn to_bounded_style(value: Vec<u8>) -> Result<BoundedStyle, DispatchError> {
        Ok(value.try_into().map_err(|_| Error::<T>::NameTooLong)?)
    }

    pub(super) fn to_bounded_sub_styles(
        value: Vec<Vec<u8>>,
    ) -> Result<BoundedSubStyles, DispatchError> {
        let mut subs: BoundedSubStyles = Default::default();
        for sub in value {
            subs.try_push(BoundedStyle::try_from(sub).map_err(|_| Error::<T>::NameTooLong)?)
                .map_err(|_| Error::<T>::StylesCapacity)?
        }
        Ok(subs)
    }

    pub(super) fn checked_add_subs(
        tree: &mut StylesTree,
        subs: BoundedSubStyles,
        into: BoundedStyle,
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
