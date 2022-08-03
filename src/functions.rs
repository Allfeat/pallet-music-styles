use super::*;

pub fn btree_to_vec<T: Clone>(values: BTreeSet<T>) -> Vec<T> {
    values.iter().cloned().collect()
}

impl<T: Config> Pallet<T> {
    pub fn unwrap_name(input: &Vec<u8>) -> Result<StyleName<T>, DispatchError> {
        Ok(input
            .clone()
            .try_into()
            .map_err(|_| Error::<T>::NameTooLong)?)
    }

    pub fn create_empty_sub_list() -> Result<SubList<T>, DispatchError> {
        let empty_vec = Vec::from(Vec::new());
        Ok(BoundedVec::try_from(empty_vec).map_err(|_| Error::<T>::StorageFull)?)
    }

    pub fn unwrap_new_sub(input: &Option<Vec<Vec<u8>>>) -> Result<SubList<T>, DispatchError> {
        let vec = match input {
            Some(vec) => vec,
            None => return Ok(Self::create_empty_sub_list()?),
        };

        let mut btree: BTreeSet<StyleName<T>> = BTreeSet::new();
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
