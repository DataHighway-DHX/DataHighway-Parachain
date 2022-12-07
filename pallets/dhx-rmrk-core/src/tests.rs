use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::ensure_signed;
use sp_runtime::DispatchError;

#[test]
fn permissionned_origin() {
    let denied_caller = 6;
    let denied_origin = Origin::signed(denied_caller);
    let allowed_origins = ALLOWED_MINTERS.iter().map(|id| Origin::signed(id.clone())).collect::<Vec<_>>();
    assert!(!ALLOWED_MINTERS.contains(&denied_caller), "this caller is in allowed minter list. Change test value");

    // These calls should fail with insufficient permission
    new_test_ext().execute_with(|| {
        assert_noop!(
            DhxRmrkCore::mint_nft(
                denied_origin.clone(),
                None,
                1,
                2,
                None,
                None,
                Default::default(),
                false,
                None,
            ),
            Error::<Test>::InsufficientPermission,
        );

        assert_noop!(
            DhxRmrkCore::mint_nft_directly_to_nft(
                denied_origin.clone(),
                (1, 2),
                1,
                2,
                None,
                None,
                Default::default(),
                false,
                None,
            ),
            Error::<Test>::InsufficientPermission,
        );

        assert_noop!(
            DhxRmrkCore::create_collection(
                denied_origin.clone(),
                1,
                Default::default(),
                None,
                Default::default(),
            ),
            Error::<Test>::InsufficientPermission,
        );
    });

    // These calls should pass
    new_test_ext().execute_with(|| {
        for (index, origin) in allowed_origins.iter().enumerate() {
            let primary_nft_id = index as u32;
            let derived_nft_id = primary_nft_id + allowed_origins.len() as u32;
            let collection_id = index as u32;
            assert_ok!(
                Balances::set_balance(Origin::root(), ensure_signed(origin.clone()).unwrap(), 10u64.pow(18), 10),
            );

            assert_ok!(
                DhxRmrkCore::create_collection(
                    origin.clone(),
                    collection_id,
                    Default::default(),
                    None,
                    Default::default()
                ),
            );

            assert_ok!(
                DhxRmrkCore::mint_nft(
                    origin.clone(),
                    None,
                    primary_nft_id,
                    collection_id,
                    None,
                    None,
                    Default::default(),
                    true,
                    Default::default()
                ),
            );

            assert_ok!(
                DhxRmrkCore::mint_nft_directly_to_nft(
                    origin.clone(),
                    (collection_id, primary_nft_id),
                    derived_nft_id,
                    collection_id,
                    None,
                    None,
                    Default::default(),
                    true,
                    Default::default()
                ),
            );
        }
    })
}

#[test]
pub fn allowed_producers_genesis() {
    new_test_ext().execute_with(|| {
        let mut expected_producers = ALLOWED_MINTERS.to_vec();
        expected_producers.sort();
        let mut stored_producers = crate::AuthorisedProducers::<Test>::iter_keys().collect::<Vec<_>>();
        stored_producers.sort();

        assert_eq!(expected_producers, stored_producers);
    });
}

#[test]
pub fn allowed_procuer_removal() {
    new_test_ext().execute_with(|| {
        matches!(DhxRmrkCore::get_authorised_producer(&1), Some(_));
        assert_noop!(
            DhxRmrkCore::remove_producer(Origin::signed(10), 1),
            DispatchError::BadOrigin
        );

        assert_ok!(
            DhxRmrkCore::remove_producer(Origin::root(), 1),
        );
        matches!(DhxRmrkCore::get_authorised_producer(&1), None);
    })
}

#[test]
pub fn allowed_producer_insertion() {
    new_test_ext().execute_with(|| {
        matches!(DhxRmrkCore::get_authorised_producer(&10), None);
        assert_noop!(
            DhxRmrkCore::add_producer(Origin::signed(1), 10),
            DispatchError::BadOrigin,
        );

        assert_ok!(
            DhxRmrkCore::add_producer(Origin::root(), 10)
        );
        matches!(DhxRmrkCore::get_authorised_producer(&10), Some(_));
    });
}
