use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::AccountId32;
use hex_literal::hex;

#[test]
fn invalid_minter() {
    let alice: AccountId32 = hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"].into();
    println!("Alice: {alice:?}");

    new_test_ext().execute_with(|| {
        assert_ok!(
            DhxRMrkCore::mint_nft(
                Origin::signed(2),
                None,
                1,
                2,
                None,
                None,
                Default::default(),
                false,
                None,
            )
        );
    })
}
