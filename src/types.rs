use super::*;
use frame_support::{dispatch::PartialEq, pallet_prelude::*};
use scale_info::TypeInfo;
use sp_runtime::traits::Get;
use sp_std::convert::TryFrom;

// Music style name struct
// =======================

// #[derive(Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, Clone)]
// pub struct StyleName<NameMaxLength>(BoundedVec<u8, NameMaxLength>);

// impl<NameMaxLength: Get<u32>> TryFrom<Vec<u8>> for StyleName<NameMaxLength> {
//     type Error = ();

//     fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
//         let name: BoundedVec<u8, NameMaxLength> = value.try_into()?;
//         Ok(StyleName(name))
//     }
// }

// impl<NameMaxLength: Get<u32>> PartialEq for StyleName<NameMaxLength> {
//     fn eq(&self, other: &Self) -> bool {
//         self.0 == other.0
//     }
// }

// Sub music style list struct
// ==========================
// #[derive(Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, Clone)]
// pub struct SubStyleList<NameMaxLength, MaxSubStyles>(
//     BoundedVec<StyleName<NameMaxLength>, MaxSubStyles>,
// );

// Try to create a SubStyleList from a vec of names (vec<u8>)
// impl<NameMaxLength, MaxSubStyles> TryFrom<Vec<Vec<u8>>>
//     for SubStyleList<NameMaxLength, MaxSubStyles>
// where
//     NameMaxLength: Get<u32>,
//     MaxSubStyles: Get<u32>,
// {
//     type Error = ();

//     fn try_from(value: Vec<Vec<u8>>) -> Result<Self, Self::Error> {
//         let vec = Vec::new();
//         for name in value {
//             let name = StyleName::<NameMaxLength>::try_from(name)?;
//             vec.push(name);
//         }

//         let styles = BoundedVec::<StyleName<NameMaxLength>, MaxSubStyles>::try_from(vec)?;

//         Ok(SubStyleList(styles))
//     }
// }

// Note: The substrate version of Contains is made for work with on-chain storage
// and don't have `self` in its function signature.
// Here we need it and it's for that we don't impl Contains trait
// impl<NameMaxLength, MaxSubStyles> SubStyleList<NameMaxLength, MaxSubStyles>
// where
//     NameMaxLength: Get<u32>,
//     MaxSubStyles: Get<u32>,
// {
//     fn contains(&self, t: &StyleName<NameMaxLength>) -> bool {
//         self.0.iter().find(|&item| item == t).is_some()
//     }
// }

// Main music style struct
// =======================

// TODO: Manually impl Clone, Eq, From, PartialEq for this type
/// Structure that holds the music style information that will be stored on-chain
// #[derive(Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
// pub struct Style<NameMaxLength, MaxSubStyles> {
//     pub name: BoundedVec<u8, NameMaxLength>,
//     pub children: BoundedVec<BoundedVec<u8, NameMaxLength>, MaxSubStyles>,
// }

// /// Create a Style from a StyleName and a SubStyleList
// impl<NameMaxLength, MaxSubStyles> Style<NameMaxLength, MaxSubStyles>
// where
//     NameMaxLength: Get<u32>,
//     MaxSubStyles: Get<u32>,
// {
//     fn new(
//         name: StyleName<NameMaxLength>,
//         children: SubStyleList<NameMaxLength, MaxSubStyles>,
//     ) -> Self {
//         Self { name, children }
//     }

//     // Search in the style name itself as well in the sub styles
//     fn contains(&self, t: &StyleName<NameMaxLength>) -> bool {
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
// impl<NameMaxLength, MaxSubStyles> TryFrom<StyleName<NameMaxLength>>
//     for Style<NameMaxLength, MaxSubStyles>
// where
//     NameMaxLength: Get<u32>,
//     MaxSubStyles: Get<u32>,
// {
//     type Error = ();

//     fn try_from(value: StyleName<NameMaxLength>) -> Result<Self, Self::Error> {
//         let style = Self {
//             name: value,
//             children: SubStyleList::try_from(Vec::from(Vec::new()))?,
//         };

//         Ok(style)
//     }
// }

// Main music style struct as a list
// =================================
// #[derive(Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
// pub struct StyleList<NameMaxLength, MaxSubStyles, MaxStyles>(
//     BoundedVec<Style<NameMaxLength, MaxSubStyles>, MaxStyles>,
// );

// impl<NameMaxLength, MaxSubStyles, MaxStyles> StyleList<NameMaxLength, MaxSubStyles, MaxStyles>
// where
//     NameMaxLength: Get<u32>,
//     MaxSubStyles: Get<u32>,
//     MaxStyles: Get<u32>,
// {
//     fn contains(&self, t: &StyleName<NameMaxLength>) -> bool {
//         self.0.iter().find(|&item| item.contains(t)).is_some()
//     }
// }
