use super::{Event as MusicStyleEvent, *};
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

/// Panic is the given event is different that the last emitted event
fn assert_last_event(event: MusicStyleEvent<Test>) {
    System::assert_last_event(mock::Event::MusicStylesPallet(event))
}

mod add {
    use super::*;

    #[test]
    fn non_admin_cannot_add_a_style() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                MusicStylesPallet::add(Origin::signed(BOB), b"Reggae".to_vec().into()),
                BadOrigin
            );
        });
    }

    #[test]
    fn too_long_style_name_should_fail() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                MusicStylesPallet::add(
                    Origin::root(),
                    generate_random_string(100).as_bytes().to_vec()
                ),
                Error::<Test>::NameTooLong
            );
        });
    }

    #[test]
    fn add_should_mutate_chain_and_emit_event() {
        new_test_ext().execute_with(|| {
            let name = generate_random_string(10).as_bytes().to_vec();

            assert_ok!(MusicStylesPallet::add(Origin::root(), name.clone()));

            let last_count = MusicStylesPallet::count();
            let music_style =
                MusicStylesPallet::get(last_count - 1).expect("Music style not found");
            let music_style_fake = MusicStyle { name };

            // Check that the storage have been updated
            assert_eq!(last_count, 1);
            assert_eq!(music_style.name, music_style_fake.name);

            // Check that the event has been called
            assert_last_event(Added(0));
        });
    }
}

mod remove {
    use super::*;

    #[test]
    fn non_admin_cannot_remove_a_style() {
        new_test_ext().execute_with(|| {
            assert_noop!(MusicStylesPallet::remove(Origin::signed(BOB), 0), BadOrigin);
        });
    }

    #[test]
    fn cannot_remove_an_unexistising_id() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                MusicStylesPallet::remove(Origin::root(), 10),
                Error::<Test>::MusicStyleNotFound
            );
        });
    }

    #[test]
    fn remove_should_mutate_chain_and_emit_event() {
        new_test_ext().execute_with(|| {
            // Add a new style to be able to remove it later
            assert_ok!(MusicStylesPallet::add(
                Origin::root(),
                b"Reggae".to_vec().into()
            ));

            // Remove it
            assert_ok!(MusicStylesPallet::remove(Origin::root(), 0));

            // Querying it should fail since it was removed
            assert_eq!(MusicStylesPallet::get(0), None);

            // Check that the storage has been updated
            assert_eq!(MusicStylesPallet::count(), 0);

            // Check that the event has been called
            assert_last_event(Removed(0));
        });
    }
}
