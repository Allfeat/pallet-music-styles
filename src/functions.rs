// use frame_support::StorageValue;

use super::*;

pub fn btree_to_vec<T: Clone>(values: BTreeSet<T>) -> Vec<T> {
    values.iter().cloned().collect()
}

impl<T: Config> Pallet<T> {
    /// Search in all styles and sub styles
    /// Used for Pallet::contains
    pub fn contains(t: &BoundedName<T>) -> bool {
        for style in <Styles<T>>::get() {
            if &style.name == t {
                return true;
            }
            if style.sub_styles.iter().find(|&name| name == t).is_some() {
                return true;
            }
        }
        false
    }

    /// Search in the primary style names
    /// Not in the sub styles
    pub fn contains_primary_style(name: &BoundedName<T>) -> bool {
        <Styles<T>>::get()
            .iter()
            .find(|style| &style.name == name)
            .is_some()
    }

    // Search in all sub styles
    // pub fn contains_sub_style(name: &BoundedName<T>) -> bool {
    //     <Styles<T>>::get()
    //         .iter()
    //         .flat_map(|style| style.sub_styles.clone())
    //         .collect::<Vec<BoundedName<T>>>()
    //         .iter()
    //         .find(|style| style == &name)
    //         .is_some()
    // }

    // Search in sub style for a given parent style
    // pub fn contains_sub_style_for(parent: &BoundedName<T>, name: &BoundedName<T>) -> bool {
    //     let styles = <Styles<T>>::get();
    //     match &styles.iter().find(|style| &style.name == parent) {
    //         Some(style) => style
    //             .sub_styles
    //             .iter()
    //             .find(|style| style == &name)
    //             .is_some(),
    //         None => false,
    //     }
    // }

    pub fn unwrap_name(input: &Vec<u8>) -> Result<BoundedName<T>, DispatchError> {
        Ok(input
            .clone()
            .try_into()
            .map_err(|_| Error::<T>::NameTooLong)?)
    }

    fn create_empty_sub_list() -> Result<BoundedNameList<T>, DispatchError> {
        let empty_vec = Vec::from(Vec::new());
        Ok(BoundedVec::try_from(empty_vec).map_err(|_| Error::<T>::StorageFull)?)
    }

    pub fn unwrap_new_sub(
        input: &Option<Vec<Vec<u8>>>,
    ) -> Result<BoundedNameList<T>, DispatchError> {
        let vec = match input {
            Some(vec) => vec,
            None => return Ok(Self::create_empty_sub_list()?),
        };

        let mut btree: BTreeSet<BoundedName<T>> = BTreeSet::new();
        for name_vec in vec.iter() {
            let name = Self::unwrap_name(name_vec)?;

            btree.insert(name);
        }

        if vec.len() != btree.len() {
            return Err(Error::<T>::DuplicatedStyle)?;
        }

        Ok(BoundedVec::try_from(btree_to_vec(btree)).map_err(|_| Error::<T>::StorageFull)?)
    }
}
