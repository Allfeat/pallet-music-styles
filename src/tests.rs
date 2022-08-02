use super::{Event as MusicStyleEvent, *};
use crate::{mock::*, Event::*};
use frame_support::{assert_noop, assert_ok, error::BadOrigin, BoundedVec};
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

/// Panic is the given event is different that the last emitted event
fn assert_last_event(event: MusicStyleEvent<Test>) {
    System::assert_last_event(mock::Event::MusicStylesPallet(event))
}

#[test]
fn test_genesis() {
    new_test_ext(true).execute_with(|| {
        type Style = BoundedVec<u8, <Test as Config>::NameMaxLength>;
        let name: Style = b"Reggae"
            .to_vec()
            .try_into()
            .expect("Music style name too long");

        let styles_vec = MusicStylesPallet::get();

        assert_eq!(styles_vec.len(), 4);
        assert!(styles_vec.contains(&name));
    });
}

mod add {
    use super::*;

    #[test]
    fn non_admin_cannot_add_a_style() {
        new_test_ext(false).execute_with(|| {
            assert_noop!(
                MusicStylesPallet::add(Origin::signed(BOB), b"Reggae".to_vec().into()),
                BadOrigin
            );
        });
    }

    #[test]
    fn too_long_style_name_should_fail() {
        new_test_ext(false).execute_with(|| {
            assert_noop!(
                MusicStylesPallet::add(
                    Origin::root(),
                    generate_random_string((NameMaxLength::get() as usize) + 10)
                        .as_bytes()
                        .to_vec()
                ),
                Error::<Test>::NameTooLong
            );
        });
    }

    #[test]
    fn should_fail_before_exceeded_storage_bound() {
        new_test_ext(false).execute_with(|| {
            let get_name = |i: u32| {
                let mut name = generate_random_string(10).as_bytes().to_vec();
                name.push((i % 256) as u8);
                name
            };

            // Fill the storage
            for i in 0..MaxStyles::get() {
                assert_ok!(MusicStylesPallet::add(Origin::root(), get_name(i)));
            }

            // One more should fail
            assert_noop!(
                MusicStylesPallet::add(Origin::root(), get_name(MaxStyles::get())),
                Error::<Test>::StorageFull
            );
        });
    }

    #[test]
    fn add_should_mutate_chain_and_emit_event() {
        new_test_ext(false).execute_with(|| {
            let name = generate_random_string(10).as_bytes().to_vec();

            assert_ok!(MusicStylesPallet::add(Origin::root(), name.clone()));

            let styles = MusicStylesPallet::get();

            // Check that the storage have been updated
            assert_eq!(styles.len(), 1);
            assert!(styles.contains(&name.clone().try_into().expect("Music style name too long")));

            // Check that the event has been called
            assert_last_event(Added(name));
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
        new_test_ext(false).execute_with(|| {
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
            let name = b"Reggae".to_vec();

            // Remove it
            assert_ok!(MusicStylesPallet::remove(Origin::root(), name.clone()));

            // Querying it should fail since it was removed
            assert!(!MusicStylesPallet::contains(&name));

            // Check that the storage has been updated
            assert_eq!(MusicStylesPallet::get().len(), initial_count - 1);

            // Check that the event has been called
            assert_last_event(Removed(name));
        });
    }
}
