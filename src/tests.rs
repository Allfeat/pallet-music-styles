use super::*;
use crate::{mock::*, Event::*};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use rand::{thread_rng, Rng};
use sp_runtime::traits::{BlakeTwo256, Hash};

// Test hash
// duplicate that hash are different if 2 sub styles use the same name
// Update a name should not updates the id
// Merge `remove` and `remove_sub`

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

/// Quickly create a style without using the pallet
fn new_style<'a>(parent_name: &'a str, sub_names: Vec<&'a str>) -> StyleType<Test> {
    let parent_name: BoundedName<Test> = parent_name.as_bytes().to_vec().try_into().unwrap();
    let parent_id = BlakeTwo256::hash(&parent_name);

    let mut sub_styles: Vec<SubStyleType<Test>> = vec![];

    for sub_name in &sub_names {
        let sub_name = sub_name.as_bytes().to_vec();
        let bounded_sub_name: BoundedName<Test> = sub_name.clone().try_into().unwrap();
        let sub_id = BlakeTwo256::hash(&[parent_id.as_bytes(), &sub_name].concat());

        let sub_style: SubStyleType<Test> = SubStyle {
            id: sub_id,
            name: bounded_sub_name.clone(),
            parent_id,
        };

        sub_styles.push(sub_style);
    }

    let style: StyleType<Test> = Style {
        id: parent_id,
        name: parent_name.clone(),
        sub_styles: BoundedVec::try_from(sub_styles).unwrap(),
    };
    return style;
}

fn rap() -> StyleType<Test> {
    new_style("Rap", vec!["Drill", "Trap", "Hardcore"])
}

#[test]
fn test_genesis() {
    new_test_ext(true).execute_with(|| {
        let styles = MusicStylesPallet::get();

        // Create "Rock" style from scratch to compare to on-chain "Rock" sub fields
        let test_rock_style = new_style("Rock", vec!["Hardcore"]);

        let pallet_rock_style = styles
            .iter()
            .find(|style| &style.name == &test_rock_style.name)
            .unwrap();

        assert_eq!(pallet_rock_style.id, test_rock_style.id);
        assert_eq!(pallet_rock_style.id, test_rock_style.id);
        assert_eq!(
            pallet_rock_style.sub_styles.len(),
            test_rock_style.sub_styles.len()
        );

        // Main styles
        for entry in &["Reggae", "Rap", "Rock"] {
            let name: BoundedName<Test> = entry.as_bytes().to_vec().try_into().unwrap();
            assert!(styles.iter().find(|style| style.name == name).is_some());
        }

        // Sub style in rap
        for entry in &["Drill", "Trap"] {
            let rap_name: BoundedName<Test> = b"Rap".to_vec().try_into().unwrap();
            let rap_style = styles.iter().find(|style| style.name == rap_name).unwrap();
            let name: BoundedName<Test> = entry.as_bytes().to_vec().try_into().unwrap();

            assert!(rap_style
                .sub_styles
                .iter()
                .find(|style| style.name == name)
                .is_some());
        }

        let unexistising: BoundedName<Test> = b"Unexistising".to_vec().try_into().unwrap();
        assert!(styles
            .iter()
            .find(|style| style.name == unexistising)
            .is_none());
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
                Error::<Test>::StylesCapacity
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
                Error::<Test>::StylesCapacity
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
            let styles = MusicStylesPallet::get();
            let added: StyleType<Test> = styles
                .iter()
                .find(|style| style.name == name)
                .unwrap()
                .clone();

            assert!(MusicStylesPallet::contains_name(
                &name.clone().try_into().unwrap()
            ));
            assert!(MusicStylesPallet::contains_name(
                &sub_name.clone().try_into().unwrap()
            ));

            // Check that the event has been called
            assert_last_event(StyleAdded(added));
        });
    }
}

mod add_sub_style {
    use super::*;

    #[test]
    fn non_admin_cannot_add_sub_style() {
        new_test_ext(true).execute_with(|| {
            let rap = rap();
            assert_noop!(
                MusicStylesPallet::add_sub_style(Origin::signed(BOB), rap.id, b"New".to_vec()),
                BadOrigin
            );
        });
    }

    #[test]
    fn cannot_add_existing_sub_style() {
        new_test_ext(true).execute_with(|| {
            assert_noop!(
                MusicStylesPallet::add_sub_style(Origin::root(), rap().id, b"Drill".to_vec()),
                Error::<Test>::NameAlreadyExists
            );
        });
    }

    #[test]
    fn cannot_add_style_to_unexistising_parent_style() {
        new_test_ext(true).execute_with(|| {
            let unexistising_style = new_style("Unexistising", Vec::new());
            assert_noop!(
                MusicStylesPallet::add_sub_style(
                    Origin::root(),
                    unexistising_style.id,
                    b"Electro".to_vec()
                ),
                Error::<Test>::StyleNotFound
            );
        });
    }

    #[test]
    fn too_long_style_name_should_fail() {
        new_test_ext(true).execute_with(|| {
            let rap = rap();
            let long_name = generate_random_string((NameMaxLength::get() as usize) + 10)
                .as_bytes()
                .to_vec();

            // Too long main style name
            assert_noop!(
                MusicStylesPallet::add_sub_style(Origin::root(), rap.id, long_name.clone(),),
                Error::<Test>::NameTooLong
            );
        });
    }

    #[test]
    fn add_style_should_not_exceeds_capacity() {
        // The "Reggae" parent style is empty
        new_test_ext(true).execute_with(|| {
            let reggae = new_style("Reggae", vec![]);

            // Fill the storage
            for i in 0..MaxSubStyles::get() {
                assert_ok!(MusicStylesPallet::add_sub_style(
                    Origin::root(),
                    reggae.id.clone(),
                    generate_random_name(i)
                ));
            }

            // One more should fail
            assert_noop!(
                MusicStylesPallet::add_sub_style(
                    Origin::root(),
                    reggae.id.clone(),
                    b"Too much".to_vec()
                ),
                Error::<Test>::StylesCapacity
            );
        });
    }

    #[test]
    fn add_sub_style_should_mutate_chain_and_emit_event() {
        // The "Reggae" parent style is empty
        new_test_ext(true).execute_with(|| {
            let rap = rap();
            let new_name = b"Victory".to_vec();
            assert_ok!(MusicStylesPallet::add_sub_style(
                Origin::root(),
                rap.id.clone(),
                new_name.clone()
            ));

            // Check that the storage have been updated
            let styles = MusicStylesPallet::get();
            let parent_style = styles.iter().find(|s| s.id == rap.id.clone()).unwrap();
            let bounded_new_name: BoundedName<Test> = new_name.try_into().unwrap();
            let created_sub_style = parent_style
                .sub_styles
                .iter()
                .find(|s| s.name == bounded_new_name)
                .unwrap();

            assert!(MusicStylesPallet::contains(&created_sub_style.id));

            // Check that the event has been called
            assert_last_event(SubStyleAdded(rap.id, created_sub_style.clone()));
        });
    }
}

mod update_style_name {
    use super::*;

    #[test]
    fn non_admin_cannot_update_style() {
        let rap = rap();
        let new_name = b"New".to_vec();

        new_test_ext(true).execute_with(|| {
            assert_noop!(
                MusicStylesPallet::update_style_name(Origin::signed(BOB), rap.id, new_name),
                BadOrigin
            );
        });
    }

    #[test]
    fn cannot_update_unexistising_style() {
        new_test_ext(true).execute_with(|| {
            let unexistising_parent = new_style("unexistising_parent", vec!["unexistising_child"]);
            assert_noop!(
                MusicStylesPallet::update_style_name(
                    Origin::root(),
                    unexistising_parent.id,
                    b"Pop".to_vec()
                ),
                Error::<Test>::StyleNotFound
            );
        });
    }

    #[test]
    fn cannot_update_style_with_existing_style_name() {
        new_test_ext(true).execute_with(|| {
            let rap = rap();
            let drill_name: BoundedName<Test> = b"Drill".to_vec().try_into().unwrap();
            let drill = rap
                .sub_styles
                .iter()
                .find(|s| s.name == drill_name)
                .unwrap();

            // 1. 2 first level styles shouldn't have same name
            assert_noop!(
                MusicStylesPallet::update_style_name(Origin::root(), rap.id, b"Reggae".to_vec()),
                Error::<Test>::NameAlreadyExists
            );

            // 2. 2 sub styles shouldn't have same name
            assert_noop!(
                MusicStylesPallet::update_style_name(
                    Origin::root(),
                    drill.id,
                    b"Hardcore".to_vec()
                ),
                Error::<Test>::NameAlreadyExists
            );
        });
    }

    #[test]
    fn cannot_update_with_too_long_style_name() {
        new_test_ext(true).execute_with(|| {
            let rap = rap();
            let long_name = generate_random_string((NameMaxLength::get() as usize) + 10)
                .as_bytes()
                .to_vec();

            // Too long main style name
            assert_noop!(
                MusicStylesPallet::update_style_name(Origin::root(), rap.id, long_name.clone(),),
                Error::<Test>::NameTooLong
            );
        });
    }

    // TODO: Same for sub style
    #[test]
    fn update_style_should_mutate_chain_and_emit_event() {
        new_test_ext(true).execute_with(|| {
            let rap = rap();
            let new_name_vec = b"Relax".to_vec();
            let bounded_new_name: BoundedName<Test> = new_name_vec.clone().try_into().unwrap();

            assert_ok!(MusicStylesPallet::update_style_name(
                Origin::root(),
                rap.id,
                new_name_vec.clone(),
            ));

            let updated_styles = MusicStylesPallet::get();
            let updated_style = updated_styles.iter().find(|s| s.id == rap.id).unwrap();

            assert_eq!(updated_style.name, bounded_new_name);
            assert_ne!(updated_style.name, rap.name);

            assert_last_event(StyleNameUpdated(rap.id, new_name_vec));
        });
    }

    #[test]
    fn update_sub_style_should_mutate_chain_and_emit_event() {
        new_test_ext(true).execute_with(|| {
            let rap = rap();
            let drill_name: BoundedName<Test> = b"Drill".to_vec().try_into().unwrap();
            let drill = rap
                .sub_styles
                .iter()
                .find(|s| s.name == drill_name)
                .unwrap();
            let new_name_vec = b"Relax".to_vec();
            let bounded_new_name: BoundedName<Test> = new_name_vec.clone().try_into().unwrap();

            assert_ok!(MusicStylesPallet::update_style_name(
                Origin::root(),
                drill.id,
                new_name_vec.clone(),
            ));

            let updated_styles = MusicStylesPallet::get();
            let updated_style = updated_styles.iter().find(|s| s.id == rap.id).unwrap();
            let updated_sub_style = updated_style
                .sub_styles
                .iter()
                .find(|s| s.id == drill.id)
                .unwrap();

            assert_eq!(updated_sub_style.name, bounded_new_name);
            assert_ne!(updated_sub_style.name, rap.name);

            assert_last_event(StyleNameUpdated(drill.id, new_name_vec));
        });
    }
}

mod remove {
    use super::*;

    #[test]
    fn non_admin_cannot_remove_a_style() {
        new_test_ext(true).execute_with(|| {
            let rap = rap();
            assert_noop!(
                MusicStylesPallet::remove(Origin::signed(BOB), rap.id),
                BadOrigin
            );
        });
    }

    #[test]
    fn cannot_remove_an_unexistising_id() {
        new_test_ext(true).execute_with(|| {
            let hash = BlakeTwo256::hash(&b"unexistising".to_vec());
            assert_noop!(
                MusicStylesPallet::remove(Origin::root(), hash),
                Error::<Test>::StyleNotFound
            );
        });
    }

    #[test]
    fn remove_should_mutate_chain_and_emit_event() {
        new_test_ext(true).execute_with(|| {
            let initial_count = MusicStylesPallet::get().len();
            let rap = rap();
            let drill = rap
                .sub_styles
                .iter()
                .find(|&s| s.name.clone().into_inner() == b"Drill".to_vec())
                .unwrap();

            // Remove it
            assert_ok!(MusicStylesPallet::remove(Origin::root(), rap.id));

            // Querying it should fail since it was removed
            assert!(!MusicStylesPallet::contains(&rap.id));
            assert!(!MusicStylesPallet::contains_name(&rap.name));
            // Same for sub styles
            assert!(!MusicStylesPallet::contains(&drill.id));

            // Check that the storage has been updated
            assert_eq!(MusicStylesPallet::get().len(), initial_count - 1);

            // Check that the event has been called
            assert_last_event(StyleRemoved(rap));
        });
    }
}

mod remove_sub_style {
    use super::*;

    #[test]
    fn non_admin_cannot_remove_sub_style() {
        new_test_ext(true).execute_with(|| {
            let rap = rap();
            let bounded_name: BoundedName<Test> = b"Drill".to_vec().try_into().unwrap();
            let sub_rap_item = rap
                .sub_styles
                .iter()
                .find(|s| s.name == bounded_name)
                .unwrap();

            assert_noop!(
                MusicStylesPallet::remove_sub_style(Origin::signed(BOB), sub_rap_item.id),
                BadOrigin
            );
        });
    }

    #[test]
    fn cannot_remove_an_unexistising_sub_id() {
        new_test_ext(true).execute_with(|| {
            let unexistising_parent = new_style("unexistising_parent", vec!["unexistising_child"]);
            let unexistising_child = unexistising_parent.sub_styles.get(0).unwrap();
            assert_noop!(
                MusicStylesPallet::remove_sub_style(Origin::root(), unexistising_child.id),
                Error::<Test>::StyleNotFound
            );
        });
    }

    #[test]
    fn remove_should_mutate_chain_and_emit_event() {
        new_test_ext(true).execute_with(|| {
            let rap = rap();
            let bounded_name: BoundedName<Test> = b"Drill".to_vec().try_into().unwrap();
            let styles = MusicStylesPallet::get();

            let style = styles.iter().find(|&s| s.id == rap.id).unwrap();
            let initial_sub_count = style.sub_styles.len();
            let sub_style = style
                .sub_styles
                .iter()
                .find(|s| s.name == bounded_name)
                .unwrap();

            assert_ok!(MusicStylesPallet::remove_sub_style(
                Origin::root(),
                sub_style.id
            ));

            // Parent is unchanged
            assert!(MusicStylesPallet::contains(&rap.id));

            // sub style is removed
            assert!(!MusicStylesPallet::contains(&sub_style.id));
            let updated_styles = MusicStylesPallet::get();
            let updated_style = updated_styles.iter().find(|&s| s.id == rap.id).unwrap();
            assert_eq!(updated_style.sub_styles.len(), initial_sub_count - 1);

            // Check that the event has been called
            assert_last_event(SubStyleRemoved(rap.id, sub_style.clone()));
        });
    }
}
