use super::*;
use allfeat_support::types::StyleName;
use allfeat_support::MusicStylesProvider;

impl<T: Config> MusicStylesProvider for Pallet<T> {
    type Styles = Styles;
    type StyleName = StyleName;

    fn styles() -> Self::Styles {
        Pallet::<T>::get_styles()
    }

    fn parent_styles() -> Vec<Self::StyleName> {
        Self::styles().keys().cloned().collect()
    }
    fn sub_styles() -> Vec<Self::StyleName> {
        let mut buf: Vec<Self::StyleName> = Vec::new();
        Self::styles()
            .values()
            .for_each(|v| v.iter().cloned().for_each(|s| buf.push(s)));
        buf
    }
    fn is_parent_style(style_name: &Self::StyleName) -> bool {
        Self::parent_styles().contains(style_name)
    }
    fn is_sub_style(style_name: &Self::StyleName) -> bool {
        Self::sub_styles().contains(style_name)
    }
    fn exist(style_name: &Self::StyleName) -> bool {
        let styles = Self::styles();

        if styles.contains_key(style_name) {
            return true;
        }

        styles.values().find(|v| v.contains(style_name)).is_some()
    }
}
