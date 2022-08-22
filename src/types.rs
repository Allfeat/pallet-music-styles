use super::*;

use frame_support::BoundedVec;
use sp_runtime::BoundedBTreeMap;

pub type StylesTree<T> =
    BoundedBTreeMap<BoundedStyle<T>, BoundedSubStyles<T>, <T as Config>::MaxStyleCount>;

pub type BoundedStyle<T> = BoundedVec<u8, <T as Config>::NameMaxLength>;
pub type BoundedSubStyles<T> = BoundedVec<BoundedStyle<T>, <T as Config>::MaxSubStyleCount>;
