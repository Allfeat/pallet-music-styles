use super::*;
use allfeat_support::types::StyleName;
use allfeat_support::MusicStylesProvider;

impl<T: Config> MusicStylesProvider for Pallet<T> {
    type Styles = Styles;
    type StyleName = StyleName;

    fn styles() -> Self::Styles {
        todo!()
    }
    fn parent_styles() -> Vec<Self::StyleName> {
        todo!()
    }
    fn exist(style_name: &Self::StyleName) -> bool {
        todo!()
    }
}
