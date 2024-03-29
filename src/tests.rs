use super::*;
use crate::{mock::*, Event::*};
use allfeat_support::types::music::style::{MaxParentStyles, MusicSubStyles};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use rand::{thread_rng, Rng};

/// Helper function that generates a random string from a given length
/// Should only be used for testing purpose
fn generate_random_string(length: usize) -> String {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
    let mut result = String::with_capacity(length);
    let mut rng = thread_rng();
    for _ in 0..length {
        let x: usize = rng.gen();
        result.push(chars[x % chars.len()])
    }
    result
}

fn generate_random_name(i: u32) -> Vec<u8> {
    let mut name = generate_random_string(10).as_bytes().to_vec();
    name.push((i % 256) as u8);
    name
}

/// Panic is the given event is different that the last emitted event
fn assert_last_event(event: super::Event<Test>) {
    System::assert_last_event(mock::RuntimeEvent::MusicStylesPallet(event))
}
#[test]
fn test_genesis() {
    new_test_ext(true).execute_with(|| {
        let styles: MusicStyleDB = MusicStylesPallet::get_styles();

        // Create "Rock" style from scratch to compare to on-chain "Rock" sub fields
        let test_rock_style: MusicStyleName = Vec::<u8>::from("Rock").try_into().unwrap();
        let test_rock_substyle: MusicSubStyles =
            vec![Vec::<u8>::from("Hardcore").try_into().unwrap()]
                .try_into()
                .unwrap();

        assert!(styles.contains_key(&test_rock_style));
        assert_eq!(
            styles.get_key_value(&test_rock_style),
            Some((&test_rock_style, &test_rock_substyle))
        );
    });
}

#[test]
fn test_provider_impl() {
    new_test_ext(true).execute_with(|| {
        let parent_style_name: MusicStyleName = b"Rock".to_vec().try_into().unwrap();
        assert_eq!(
            MusicStylesPallet::exist_from(b"Rock".to_vec()).unwrap(),
            Some(parent_style_name)
        );
        assert_eq!(
            MusicStylesPallet::exist_from(b"Unexisting".to_vec()).unwrap(),
            None
        )
    })
}

mod add {
    use super::*;

    #[test]
    fn non_admin_cannot_add_a_style() {
        new_test_ext(false).execute_with(|| {
            assert_noop!(
                MusicStylesPallet::add_style(
                    RuntimeOrigin::signed(BOB),
                    b"Reggae".to_vec().into(),
                    None
                ),
                BadOrigin
            );
        });
    }

    #[test]
    fn too_long_style_name_should_fail() {
        new_test_ext(false).execute_with(|| {
            let long_name =
                generate_random_string((<MaxNameLength as Get<u32>>::get() as usize) + 10)
                    .as_bytes()
                    .to_vec();

            // Too long main style name
            assert_noop!(
                MusicStylesPallet::add_style(RuntimeOrigin::root(), long_name.clone(), None),
                Error::<Test>::NameTooLong
            );

            // Too long name in sub style
            assert_noop!(
                MusicStylesPallet::add_style(
                    RuntimeOrigin::root(),
                    b"test".to_vec(),
                    Some(vec![long_name])
                ),
                Error::<Test>::NameTooLong
            );
        });
    }

    #[test]
    fn should_fail_before_exceeded_main_storage_bound() {
        new_test_ext(false).execute_with(|| {
            // Fill the storage
            for i in 0..<MaxParentStyles as Get<u32>>::get() {
                assert_ok!(MusicStylesPallet::add_style(
                    RuntimeOrigin::root(),
                    generate_random_name(i),
                    None
                ));
            }

            // One more should fail
            assert_noop!(
                MusicStylesPallet::add_style(
                    RuntimeOrigin::root(),
                    generate_random_name(<MaxParentStyles as Get<u32>>::get()),
                    None
                ),
                Error::<Test>::StylesCapacity
            );
        });
    }

    #[test]
    fn should_fail_before_exceeded_sub_storage_bound() {
        new_test_ext(false).execute_with(|| {
            // Create sub style vec with too many items
            let mut sub = vec![];
            for i in 0..<MaxSubStyles as Get<u32>>::get() + 2 {
                sub.push(generate_random_name(i));
            }

            assert_noop!(
                MusicStylesPallet::add_style(RuntimeOrigin::root(), b"Test".to_vec(), Some(sub)),
                Error::<Test>::StylesCapacity
            );
        });
    }

    #[test]
    fn add_should_mutate_chain_and_emit_event() {
        new_test_ext(false).execute_with(|| {
            let before_styles: MusicStyleDB = MusicStylesPallet::get_styles();

            let name = generate_random_name(1);
            let sub_name = generate_random_name(2);
            let subs = Some(Vec::from([sub_name.clone()]));

            assert_ok!(MusicStylesPallet::add_style(
                RuntimeOrigin::root(),
                name.clone(),
                subs.clone()
            ));

            // Check that the storage have been updated
            let after_styles: MusicStyleDB = MusicStylesPallet::get_styles();
            assert!(after_styles != before_styles);
            assert!(after_styles.contains_key(&name.clone().try_into().unwrap()));
            let after_subs = after_styles.get(&name.clone().try_into().unwrap()).unwrap();
            for sub in subs.clone().unwrap().iter() {
                let bounded_sub: MusicStyleName = sub.clone().try_into().unwrap();
                after_subs.iter().find(|sub| *sub == &bounded_sub).unwrap();
            }

            // Check that the events has been called
            assert_eq!(
                System::events()[0].event,
                mock::RuntimeEvent::MusicStylesPallet(StyleAdded(name.clone()))
            );
            for (i, sub) in subs.unwrap().iter().enumerate() {
                assert_eq!(
                    System::events()[i + 1].event,
                    mock::RuntimeEvent::MusicStylesPallet(SubStyleAdded(sub.clone()))
                );
            }
        });
    }
}

mod add_sub_style {
    use super::*;

    #[test]
    fn non_admin_cannot_add_sub_style() {
        new_test_ext(true).execute_with(|| {
            assert_noop!(
                MusicStylesPallet::add_sub_style(
                    RuntimeOrigin::signed(BOB),
                    b"Rap".to_vec(),
                    vec![b"New".to_vec()]
                ),
                BadOrigin
            );
        });
    }

    #[test]
    fn cannot_add_existing_sub_style() {
        new_test_ext(true).execute_with(|| {
            assert_noop!(
                MusicStylesPallet::add_sub_style(
                    RuntimeOrigin::root(),
                    b"Rap".to_vec(),
                    vec![b"Drill".to_vec()]
                ),
                Error::<Test>::NameAlreadyExists
            );
        });
    }

    #[test]
    fn cannot_add_style_to_unexistising_parent_style() {
        new_test_ext(true).execute_with(|| {
            assert_noop!(
                MusicStylesPallet::add_sub_style(
                    RuntimeOrigin::root(),
                    b"Inexisting Style".to_vec(),
                    vec![b"test sub style".to_vec()],
                ),
                Error::<Test>::StyleNotFound
            );
        });
    }

    #[test]
    fn too_long_style_name_should_fail() {
        new_test_ext(true).execute_with(|| {
            let long_name =
                generate_random_string((<MaxNameLength as Get<u32>>::get() as usize) + 10)
                    .as_bytes()
                    .to_vec();

            // Too long main style name
            assert_noop!(
                MusicStylesPallet::add_sub_style(
                    RuntimeOrigin::root(),
                    b"Rap".to_vec(),
                    vec![long_name]
                ),
                Error::<Test>::NameTooLong
            );
        });
    }

    #[test]
    fn add_sub_style_should_not_exceeds_capacity() {
        // The "Reggae" parent style is empty
        new_test_ext(true).execute_with(|| {
            // Fill the storage
            for i in 0..<MaxSubStyles as Get<u32>>::get() {
                assert_ok!(MusicStylesPallet::add_sub_style(
                    RuntimeOrigin::root(),
                    b"Raggae".to_vec(),
                    vec![generate_random_name(i)]
                ));
            }

            // One more should fail
            assert_noop!(
                MusicStylesPallet::add_sub_style(
                    RuntimeOrigin::root(),
                    b"Raggae".to_vec(),
                    vec![b"Too much".to_vec()]
                ),
                Error::<Test>::StylesCapacity
            );
        });
    }

    #[test]
    fn add_sub_style_should_mutate_chain_and_emit_event() {
        // The "Reggae" parent style is empty
        new_test_ext(true).execute_with(|| {
            let before_styles: MusicStyleDB = MusicStylesPallet::get_styles();

            let new_name = b"Victory".to_vec();
            assert_ok!(MusicStylesPallet::add_sub_style(
                RuntimeOrigin::root(),
                b"Rap".to_vec(),
                vec![new_name.clone()]
            ));

            let after_styles: MusicStyleDB = MusicStylesPallet::get_styles();

            // Check that the storage have been updated
            assert!(before_styles != after_styles);

            after_styles
                .get(&b"Rap".to_vec().try_into().unwrap())
                .unwrap()
                .iter()
                .find(|s| *s == &MusicStyleName::from(new_name.clone().try_into().unwrap()))
                .unwrap();

            // Check that the event has been called
            assert_last_event(SubStyleAdded(new_name));
        });
    }
}
