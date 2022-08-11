// use frame_support::StorageValue;

use super::*;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};

impl<T: Config> Pallet<T> {
    /// Search in all styles and sub styles
    /// Used for Pallet::contains
    pub fn contains(t: &Vec<u8>) -> bool {
        for style in <Styles<T>>::get() {
            if &style.name.into_inner() == t {
                return true;
            }
            if style
                .sub_styles
                .iter()
                .find(|&sub| &sub.name.clone().into_inner() == t)
                .is_some()
            {
                return true;
            }
        }
        false
    }

    /// Search in all styles and sub styles  like "contains" but by hash
    pub fn contains_hash(t: &H256) -> bool {
        for style in <Styles<T>>::get() {
            if &style.id == t {
                return true;
            }
            if style.sub_styles.iter().find(|&sub| &sub.id == t).is_some() {
                return true;
            }
        }
        false
    }

    /// Find a style without panic
    pub fn get_style(id: H256, styles: &BoundedStyleList<T>) -> StyleKind<T> {
        let mut parent_id: Option<H256> = None;
        let mut is_first_level_style: Option<bool> = None;
        for style in styles {
            if style.id == id {
                is_first_level_style = Some(true)
            }

            for sub_style in &style.sub_styles {
                if sub_style.id == id {
                    parent_id = Some(style.id);
                    is_first_level_style = Some(false)
                }
            }
        }

        // if `is_first_level_style` is still `None`, there is no style found at all
        let is_first_level_style = match is_first_level_style {
            Some(b) => b,
            None => return StyleKind::None,
        };

        if is_first_level_style {
            if let Some(style) = styles.iter().find(|s| s.id == id).cloned() {
                return StyleKind::MainStyle(style);
            }
        } else {
            if let Some(parent_id) = parent_id {
                if let Some(s) = styles.iter().find(|s| s.id == parent_id).cloned() {
                    if let Some(s) = s.sub_styles.iter().find(|sub| sub.id == id).cloned() {
                        return StyleKind::SubStyle(s);
                    }
                }
            }
        }

        StyleKind::None
    }

    /// Returns a storable boundedVec from a given Vec of bytes
    pub fn try_into_bounded_name(input: Vec<u8>) -> Result<BoundedName<T>, DispatchError> {
        Ok(input.try_into().map_err(|_| Error::<T>::NameTooLong)?)
    }

    /// Returns a storable boundedVec of sub styles
    fn try_into_sub_style_list(
        input: Option<Vec<Vec<u8>>>,
        parent_id: H256,
    ) -> Result<BoundedSubStyleList<T>, DispatchError> {
        let default_sub_list = || -> Result<BoundedSubStyleList<T>, DispatchError> {
            let empty_vec = Vec::from(Vec::new());
            let bounded_vec = BoundedVec::try_from(empty_vec);
            Ok(bounded_vec.map_err(|_| Error::<T>::StylesCapacity)?)
        };

        let names_vec = match input {
            Some(vec) => vec,
            None => return default_sub_list(),
        };

        let mut sub_styles: Vec<SubStyleType<T>> = Vec::new();
        for name_vec in names_vec.iter() {
            let sub_style = Self::try_new_sub_style(name_vec.clone(), parent_id)?;

            if Self::contains_hash(&parent_id) {
                return Err(Error::<T>::NameAlreadyExists)?;
            }

            match sub_styles.iter().find(|x| x.name == sub_style.name) {
                Some(_) => Err(Error::<T>::NameAlreadyExists)?,
                None => sub_styles.push(sub_style),
            };
        }

        Ok(BoundedVec::try_from(sub_styles).map_err(|_| Error::<T>::StylesCapacity)?)
    }

    /// Create a new SubStyle struct
    /// The sub style hash is created from the parent hash
    pub fn try_new_sub_style(
        name: Vec<u8>,
        parent_id: H256,
    ) -> Result<SubStyleType<T>, DispatchError> {
        let bounded_name = Self::try_into_bounded_name(name.clone())?;
        let id = BlakeTwo256::hash(&[parent_id.as_bytes(), &name].concat());
        if Self::contains_hash(&id) {
            return Err(Error::<T>::NameAlreadyExists)?;
        }

        let sub_style = SubStyle {
            id,
            name: bounded_name,
            parent_id: parent_id.clone(),
        };

        Ok(sub_style)
    }

    /// Try to create a new style
    pub fn try_new_style(
        name: Vec<u8>,
        sub: Option<Vec<Vec<u8>>>,
    ) -> Result<StyleType<T>, DispatchError> {
        let bounded_name = Self::try_into_bounded_name(name.clone())?;
        let parent_id = BlakeTwo256::hash(&name);
        let bounded_sub = Self::try_into_sub_style_list(sub, parent_id)?;

        if Self::contains_hash(&parent_id) {
            return Err(Error::<T>::NameAlreadyExists)?;
        }

        let style = Style {
            id: parent_id,
            name: bounded_name,
            sub_styles: bounded_sub,
        };

        Ok(style)
    }
}
