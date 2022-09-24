use super::*;
use crate::{BoundedStyle, Pallet, Styles};
use frame_support::traits::Contains;

impl<T: Config> Contains<BoundedStyle<T>> for Pallet<T> {
    fn contains(t: &BoundedStyle<T>) -> bool {
        let styles = <Styles<T>>::get();

        // checking in parent styles
        if styles.contains_key(t) {
            return true;
        }
        // checking in sub styles
        for style in styles.values() {
            if style.contains(t) {
                return true;
            }
        }

        false
    }
}
