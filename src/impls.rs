use super::*;
use allfeat_support::traits::music::style::MutateMusicStyles;

impl<T: Config> InspectMusicStyles for Pallet<T> {
    type Styles = MusicStyleDB;
    type StyleName = MusicStyleName;

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

#[cfg(feature = "runtime-benchmarks")]
/// This should only be used to create new style in a benchmarking behavior.
impl<T: Config> MutateMusicStyles for Pallet<T> {
    type StyleName = MusicStyleName;

    fn add_parent_style(style_name: Self::StyleName) -> sp_runtime::DispatchResult {
        pallet::Styles::<T>::try_mutate(|db| -> DispatchResult {
            ensure!(!db.contains_key(&style_name), Error::<T>::NameAlreadyExists);
            db.try_insert(style_name, Default::default())
                .map_err(|_| Error::<T>::StylesCapacity)?;
            Ok(())
        })
    }
    fn add_sub_style(
        sub_style_name: Self::StyleName,
        parent_style: Self::StyleName,
    ) -> sp_runtime::DispatchResult {
        pallet::Styles::<T>::try_mutate(|db| -> DispatchResult {
            if let Some(sub_styles) = db.get_mut(&parent_style) {
                ensure!(
                    sub_styles.contains(&sub_style_name),
                    Error::<T>::NameAlreadyExists
                );
                sub_styles
                    .try_push(sub_style_name)
                    .map_err(|_| Error::<T>::StylesCapacity)?;
                Ok(())
            } else {
                Err(Error::<T>::StyleNotFound.into())
            }
        })
    }
}
