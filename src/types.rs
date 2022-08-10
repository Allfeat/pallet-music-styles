use super::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

// Helper types
// pub type Hash<T> = <T as frame_system::Config>::Hash;

// unique pieces
pub type BoundedName<T> = BoundedVec<u8, <T as Config>::NameMaxLength>;
pub type StyleType<T> = Style<H256, BoundedName<T>, BoundedSubStyleList<T>>;
pub type SubStyleType<T> = SubStyle<H256, BoundedName<T>>;

// Bounded vectors
pub type BoundedSubStyleList<T> = BoundedVec<SubStyleType<T>, <T as Config>::MaxSubStyles>;
pub type BoundedStyleList<T> = BoundedVec<StyleType<T>, <T as Config>::MaxStyles>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Style<Hash, BoundedName, BoundedSubStyles> {
    pub id: Hash,
    pub name: BoundedName,
    pub sub_styles: BoundedSubStyles,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct SubStyle<Hash, BoundedName> {
    pub id: Hash,
    pub name: BoundedName,
    pub parent_id: Hash,
}

#[derive(Clone, PartialEq, Eq, RuntimeDebug)]
pub enum StyleKind<T: Config> {
    MainStyle(StyleType<T>),
    SubStyle(SubStyleType<T>),
    None,
}

// impl<BoundedName, BoundedNameList> Style<BoundedName, BoundedNameList>
// where
//     BoundedName: From<std::vec::Vec<u8>>,
//     BoundedNameList: From<std::vec::Vec<Vec<u8>>>,
// {
//     fn new(name: Vec<u8>, sub_styles: Option<Vec<Vec<u8>>>) -> Self {
//         let name: BoundedName = b"".to_vec().try_into().expect("msg");
//         let sub_styles: BoundedNameList = match sub_styles {
//             Some(names) => {
//                 let name_vec: Vec<BoundedName> = names
//                     .iter()
//                     .map(|&name| name.clone().try_into().expect("msg"))
//                     .collect::<Vec<BoundedName>>();

//                 BoundedVec::try_from(name_vec).expect("msg")
//             }
//             None => BoundedVec::try_from(Vec::new()).expect("msg"),
//         };

//         Style {
//             name,
//             sub_styles: Vec::from(b"".to_vec().try_into::<BoundedName>().expect("msg"))
//                 .try_into()
//                 .expect("msg"),
//         }
//     }

//     fn contains(t: &BoundedName) -> bool {
//         true
//     }
// }

// impl<T: crate::pallet::Config> Contains<BoundedName<T>> for Style<BoundedName<T>, BoundedNameList<T>> {
//     fn contains(t: &BoundedName<T>) -> bool {
//         true
//     }
// }

// Music style name struct
// =======================

// #[derive(Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, Clone)]
// pub struct BoundedName<NameMaxLength>(BoundedVec<u8, NameMaxLength>);

// impl<NameMaxLength: Get<u32>> TryFrom<Vec<u8>> for BoundedName<NameMaxLength> {
//     type Error = ();

//     fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
//         let name: BoundedVec<u8, NameMaxLength> = value.try_into()?;
//         Ok(BoundedName(name))
//     }
// }

// impl<NameMaxLength: Get<u32>> PartialEq for BoundedName<NameMaxLength> {
//     fn eq(&self, other: &Self) -> bool {
//         self.0 == other.0
//     }
// }

// Sub music style list struct
// ==========================
// #[derive(Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, Clone)]
// pub struct SubBoundedStyleList<NameMaxLength, MaxSubStyles>(
//     BoundedVec<BoundedName<NameMaxLength>, MaxSubStyles>,
// );

// Try to create a SubBoundedStyleList from a vec of names (vec<u8>)
// impl<NameMaxLength, MaxSubStyles> TryFrom<Vec<Vec<u8>>>
//     for SubBoundedStyleList<NameMaxLength, MaxSubStyles>
// where
//     NameMaxLength: Get<u32>,
//     MaxSubStyles: Get<u32>,
// {
//     type Error = ();

//     fn try_from(value: Vec<Vec<u8>>) -> Result<Self, Self::Error> {
//         let vec = Vec::new();
//         for name in value {
//             let name = BoundedName::<NameMaxLength>::try_from(name)?;
//             vec.push(name);
//         }

//         let styles = BoundedVec::<BoundedName<NameMaxLength>, MaxSubStyles>::try_from(vec)?;

//         Ok(SubBoundedStyleList(styles))
//     }
// }

// Note: The substrate version of Contains is made for work with on-chain storage
// and don't have `self` in its function signature.
// Here we need it and it's for that we don't impl Contains trait
// impl<NameMaxLength, MaxSubStyles> SubBoundedStyleList<NameMaxLength, MaxSubStyles>
// where
//     NameMaxLength: Get<u32>,
//     MaxSubStyles: Get<u32>,
// {
//     fn contains(&self, t: &BoundedName<NameMaxLength>) -> bool {
//         self.0.iter().find(|&item| item == t).is_some()
//     }
// }

// Main music style struct
// =======================

// TODO: Manually impl Clone, Eq, From, PartialEq for this type
// Structure that holds the music style information that will be stored on-chain
// #[derive(Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
// pub struct Style<NameMaxLength, MaxSubStyles> {
//     pub name: BoundedVec<u8, NameMaxLength>,
//     pub children: BoundedVec<BoundedVec<u8, NameMaxLength>, MaxSubStyles>,
// }

// /// Create a Style from a BoundedName and a SubBoundedStyleList
// impl<NameMaxLength, MaxSubStyles> Style<NameMaxLength, MaxSubStyles>
// where
//     NameMaxLength: Get<u32>,
//     MaxSubStyles: Get<u32>,
// {
//     fn new(
//         name: BoundedName<NameMaxLength>,
//         children: SubBoundedStyleList<NameMaxLength, MaxSubStyles>,
//     ) -> Self {
//         Self { name, children }
//     }

//     // Search in the style name itself as well in the sub styles
//     fn contains(&self, t: &BoundedName<NameMaxLength>) -> bool {
//         if &self.name == t {
//             return true;
//         }
//         if self.children.contains(&t) {
//             return true;
//         }
//         false
//     }
// }

// /// Create a Style from a name with an empty sub style list
// impl<NameMaxLength, MaxSubStyles> TryFrom<BoundedName<NameMaxLength>>
//     for Style<NameMaxLength, MaxSubStyles>
// where
//     NameMaxLength: Get<u32>,
//     MaxSubStyles: Get<u32>,
// {
//     type Error = ();

//     fn try_from(value: BoundedName<NameMaxLength>) -> Result<Self, Self::Error> {
//         let style = Self {
//             name: value,
//             children: SubBoundedStyleList::try_from(Vec::from(Vec::new()))?,
//         };

//         Ok(style)
//     }
// }

// Main music style struct as a list
// =================================
// #[derive(Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
// pub struct BoundedStyleList<NameMaxLength, MaxSubStyles, MaxStyles>(
//     BoundedVec<Style<NameMaxLength, MaxSubStyles>, MaxStyles>,
// );

// impl<NameMaxLength, MaxSubStyles, MaxStyles> BoundedStyleList<NameMaxLength, MaxSubStyles, MaxStyles>
// where
//     NameMaxLength: Get<u32>,
//     MaxSubStyles: Get<u32>,
//     MaxStyles: Get<u32>,
// {
//     fn contains(&self, t: &BoundedName<NameMaxLength>) -> bool {
//         self.0.iter().find(|&item| item.contains(t)).is_some()
//     }
// }
