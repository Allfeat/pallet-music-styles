use super::*;
use crate::{mock::*, Event::*};
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
    System::assert_last_event(mock::Event::MusicStylesPallet(event))
}

#[test]
fn test_genesis() {
    new_test_ext(true).execute_with(|| {
        for entry in &["Reggae", "Rap", "Drill", "Trap", "Rock"] {
            let name: BoundedName<Test> = entry.as_bytes().to_vec().try_into().unwrap();
            assert!(MusicStylesPallet::contains(&name));
        }

        let unexistising = b"Unexistising".to_vec().try_into().unwrap();
        assert_eq!(MusicStylesPallet::contains(&unexistising), false);
    });
}

mod add {
    use super::*;

    #[test]
    fn non_admin_cannot_add_a_style() {
        new_test_ext(false).execute_with(|| {
            assert_noop!(
                MusicStylesPallet::add(Origin::signed(BOB), b"Reggae".to_vec().into(), None),
                BadOrigin
            );
        });
    }

    #[test]
    fn too_long_style_name_should_fail() {
        new_test_ext(false).execute_with(|| {
            let long_name = generate_random_string((NameMaxLength::get() as usize) + 10)
                .as_bytes()
                .to_vec();

            // Too long main style name
            assert_noop!(
                MusicStylesPallet::add(Origin::root(), long_name.clone(), None),
                Error::<Test>::NameTooLong
            );

            // Too long name in sub style
            assert_noop!(
                MusicStylesPallet::add(Origin::root(), b"test".to_vec(), Some(vec![long_name])),
                Error::<Test>::NameTooLong
            );
        });
    }

    #[test]
    fn should_fail_before_exceeded_main_storage_bound() {
        new_test_ext(false).execute_with(|| {
            // Fill the storage
            for i in 0..MaxStyles::get() {
                assert_ok!(MusicStylesPallet::add(
                    Origin::root(),
                    generate_random_name(i),
                    None
                ));
            }

            // One more should fail
            assert_noop!(
                MusicStylesPallet::add(
                    Origin::root(),
                    generate_random_name(MaxStyles::get()),
                    None
                ),
                Error::<Test>::StorageFull
            );
        });
    }

    #[test]
    fn should_fail_before_exceeded_sub_storage_bound() {
        new_test_ext(false).execute_with(|| {
            // Create sub style vec with too many items
            let mut sub = vec![];
            for i in 0..MaxSubStyles::get() + 2 {
                sub.push(generate_random_name(i));
            }

            assert_noop!(
                MusicStylesPallet::add(Origin::root(), b"Test".to_vec(), Some(sub)),
                Error::<Test>::StorageFull
            );
        });
    }

    #[test]
    fn add_should_mutate_chain_and_emit_event() {
        new_test_ext(false).execute_with(|| {
            let name = generate_random_name(1);
            let sub_name = generate_random_name(2);
            let sub = Some(Vec::from([sub_name.clone()]));

            assert_ok!(MusicStylesPallet::add(
                Origin::root(),
                name.clone(),
                sub.clone()
            ));

            // Check that the storage have been updated
            assert!(MusicStylesPallet::contains(
                &name.clone().try_into().unwrap()
            ));
            assert!(MusicStylesPallet::contains(
                &sub_name.clone().try_into().unwrap()
            ));

            // Check that the event has been called
            assert_last_event(Added(name, sub));
        });
    }
}

mod remove {
    use super::*;

    #[test]
    fn non_admin_cannot_remove_a_style() {
        new_test_ext(true).execute_with(|| {
            assert_noop!(
                MusicStylesPallet::remove(Origin::signed(BOB), b"Rap".to_vec()),
                BadOrigin
            );
        });
    }

    #[test]
    fn cannot_remove_an_unexistising_id() {
        new_test_ext(true).execute_with(|| {
            assert_noop!(
                MusicStylesPallet::remove(Origin::root(), b"unexistising".to_vec()),
                Error::<Test>::StyleNotFound
            );
        });
    }

    #[test]
    fn remove_should_mutate_chain_and_emit_event() {
        new_test_ext(true).execute_with(|| {
            let initial_count = MusicStylesPallet::get().len();
            let name = b"Rap".to_vec();

            // Remove it
            assert_ok!(MusicStylesPallet::remove(Origin::root(), name.clone()));

            // Querying it should fail since it was removed
            assert!(!MusicStylesPallet::contains(
                &name.clone().try_into().unwrap()
            ));
            // Same for sub styles
            assert!(!MusicStylesPallet::contains(
                &b"Drill".to_vec().try_into().unwrap()
            ));

            // Check that the storage has been updated
            assert_eq!(MusicStylesPallet::get().len(), initial_count - 1);

            // Check that the event has been called
            assert_last_event(Removed(name));
        });
    }
}
