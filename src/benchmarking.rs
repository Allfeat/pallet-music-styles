//! Benchmarking setup for pallet-music-styles

use super::*;
use allfeat_support::types::MaxNameLength;

#[allow(unused)]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::traits::UnfilteredDispatchable;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    where_clause { where T: Config }

    add_style {
        let n in 1..<MaxNameLength as Get<u32>>::get();
        let x in 0..<MaxSubStyles as Get<u32>>::get();

        let new_style: Vec<u8> = vec![0x61; n as usize];
        let mut new_sub_styles: Option<Vec<Vec<u8>>> = None;
        for i in 0..x {
            let new_sub = vec![0 + i as u8; n as usize];
            if let Some(ref mut current_subs) = new_sub_styles {
                current_subs.push(new_sub);
            }
            else {
                new_sub_styles = Some(vec![new_sub])
            }
        }

        let origin = T::AdminOrigin::successful_origin();
        let call = Call::<T>::add_style { name: new_style.clone(), sub: new_sub_styles.clone() };
    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        if let Some(sub_styles) = new_sub_styles {
            assert_last_event::<T>(
                Event::<T>::SubStyleAdded(sub_styles.last().unwrap().clone()).into()
            );
        }
        else {
            assert_last_event::<T>(Event::<T>::StyleAdded(new_style).into());
        }
    }

    add_sub_style {
        let n in 1..<MaxNameLength as Get<u32>>::get();
        let x in 1..<MaxSubStyles as Get<u32>>::get();

        let parent_style = vec![0x61, n as u8];
        let mut new_subs_style: Vec<Vec<u8>> = vec![];
        for i in 0..x {
            new_subs_style.push(vec![0 + i as u8; n as usize])
        }

        let origin = T::AdminOrigin::successful_origin();
        Call::<T>::add_style { name: parent_style.clone(), sub: None }.dispatch_bypass_filter(origin.clone())?;
        let call = Call::<T>::add_sub_style { parent_style, subs_style: new_subs_style.clone() };
    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_last_event::<T>(Event::<T>::SubStyleAdded(new_subs_style.last().unwrap().clone()).into());
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(false), crate::mock::Test);
}
