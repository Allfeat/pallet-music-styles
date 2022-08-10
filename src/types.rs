use super::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

/// Struct stored on-chain that holds a Style
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Style<Hash, BoundedName, BoundedSubStyles> {
    pub id: Hash,
    pub name: BoundedName,
    pub sub_styles: BoundedSubStyles,
}

/// Struct stored on-chain that holds a SubStyle
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct SubStyle<Hash, BoundedName> {
    pub id: Hash,
    pub name: BoundedName,
    pub parent_id: Hash,
}

/// Struct that can hold both Style or SubStyle, useful for polymorphism.
#[derive(Clone, PartialEq, Eq, RuntimeDebug)]
pub enum StyleKind<T: Config> {
    MainStyle(StyleType<T>),
    SubStyle(SubStyleType<T>),
    None,
}

// Helper types used as aliases

// unique pieces
pub type BoundedName<T> = BoundedVec<u8, <T as Config>::NameMaxLength>;
pub type StyleType<T> = Style<H256, BoundedName<T>, BoundedSubStyleList<T>>;
pub type SubStyleType<T> = SubStyle<H256, BoundedName<T>>;

// Bounded vectors
pub type BoundedSubStyleList<T> = BoundedVec<SubStyleType<T>, <T as Config>::MaxSubStyles>;
pub type BoundedStyleList<T> = BoundedVec<StyleType<T>, <T as Config>::MaxStyles>;
