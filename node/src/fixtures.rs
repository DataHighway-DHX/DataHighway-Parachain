use module_primitives::{
    // types::*,
    types::{AccountId, Balance},
};
use codec::{
    Decode, Encode,
};
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use serde_json::{Result};
use sp_core::{
    crypto::{UncheckedFrom, UncheckedInto, Wraps},
};
use sp_runtime::{AccountId32};
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use hex::{FromHex, FromHexError};

#[derive(Serialize, Deserialize)]
struct Allocation {
    balances: Vec<(String, String)>
}

// #[derive(Encode, Decode, Debug, Default, Clone, Eq, PartialEq)]
// #[cfg_attr(feature = "std", derive())]
// pub struct AllBalances {
//     pub balances: Vec<(AccountId, Balance)>,
//     pub endowed: Vec<(AccountId, Balance)>,
// }

// reference/credits: https://github.com/hicommonwealth/edgeware-node/commit/a037f0af1e24d7a9a0a3a7e79662c27fb3ad2f5a
pub fn get_allocation(endowed_accounts_with_balances: Vec<(AccountId32, Balance)>)
	-> Result<Vec<(AccountId32, Balance)>> {
    let mut file = File::open("node/src/genesis.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

	let json: Allocation = serde_json::from_str(&data)?;
	let balances_json = json.balances;

    let mut combined_balances: Vec<(AccountId32, Balance)> = vec![];

    if endowed_accounts_with_balances.len() != 0 {
        for e in endowed_accounts_with_balances {
            let account_public_key_endowed: String = e.0.to_string();
            // println!("account_public_key_endowed {:#?}", account_public_key_endowed.clone());

            let account_balance_endowed: Balance = e.1.to_string().parse::<Balance>().unwrap();
            // println!("account_balance_endowed {:#?}", account_balance_endowed.clone());

            let account_ss58_address_endowed: AccountId32 = AccountId32::from_str(&account_public_key_endowed).unwrap();
            // println!("account_ss58_address_endowed {:#?}", account_ss58_address_endowed.clone());

            combined_balances.push((account_ss58_address_endowed.clone(), account_balance_endowed.clone()));
        }
    }

    if balances_json.len() != 0 {
        // overwrite any existing account balances in endowed using the accounts from the json file
        for e in balances_json {
            let account_public_key_json: String = e.0.to_string();
            // println!("account_public_key_json {:#?}", account_public_key_json.clone());

            let account_balance_json: Balance = e.1.to_string().parse::<Balance>().unwrap();
            // println!("account_balance_json {:#?}", account_balance_json.clone());

            let account_ss58_address_json: AccountId32 = AccountId32::from_str(&account_public_key_json).unwrap();
            // println!("account_ss58_address_json {:#?}", account_ss58_address_json.clone());

            let index_of_match = combined_balances.clone().iter().position(|x| x.0 == account_ss58_address_json.clone());

            if let Some(idx) = index_of_match.clone() {
                // println!("match b/w endowed and json for {:#?} so overwriting its balance value", account_ss58_address_json.clone());
                combined_balances[idx] = (combined_balances[idx].0.clone(), account_balance_json.clone());
            } else {
                // println!("no match b/w endowed and json for {:#?} so adding it to the list", e.0.clone());
                combined_balances.push((account_ss58_address_json.clone(), account_balance_json.clone()));
            }
        }
    }

    Ok(combined_balances.clone())

    // let mut all_balances = AllBalances {
    //     balances: vec![],
    //     endowed: vec![]
    // };

    // for e in balances_json {
    //     let accountPublicKey: String = e.0.to_string();
    //     let accountSS58Address: AccountId32 = AccountId32::from_str(&accountPublicKey).unwrap();
    //     all_balances.balances.push(
    //         (
    //             accountSS58Address.clone(),
    //             e.1.to_string().parse::<Balance>().unwrap(),
    //         )
    //     );
    // }

    // Ok(all_balances)

	// let balances: Vec<(AccountId, Balance)> = balances_json.into_iter().map(|e| {
    //     // let accountPublicKey: AccountId32 = e.0.to_string().parse::<AccountId32>().unwrap();

    //     // note: this works, but we want to read from a file instead that contains the Public Key and convert it
    //     // let accountPublicKey = AccountId32::from_str("4MkLjys3KYVtRKBWBeNUSYxymqXK3C8vKzXZuSroWv3cVhqp").unwrap();

    //     // note: this works to where we can provide the Public Key (hex without '0x' prefix)
    //     // let accountSS58Address: AccountId = UncheckedFrom::unchecked_from(hex!("a6b34be9aa95c82927b112dacf99bac1e728acb0fbae849097c0f9150fa49c23").into());

    //     // note: this DOESN'T work as hex_literal::hex!() is only for parsing a string literal at compile time
    //     // but can't be used for parsing a string at runtime, instead for runtime parsing use https://crates.io/crates/hex
    //     // let accountSS58Address: AccountId = UncheckedFrom::unchecked_from(hex!(e.0.to_string()).into());

    //     // let accountPublicKey: AccountId32 = e.0.to_string().parse::<AccountId32>().unwrap();
    //     // let accountSS58AddressString: Vec<u8> = hex::decode(accountPublicKey.clone()).unwrap();
    //     // // convert SS58 Vec<u8> into T::AccountId
    //     // // Reference: https://github.com/DataHighway-DHX/node/pull/238/files#diff-413b87387aba41c752b85ae37f8b0f347e709b83d8ca2869bbf0157c30a0aa64R1354
    //     // let _miner_account_id: AccountId32 = Decode::decode(&mut accountSS58AddressString.as_slice().clone()).unwrap();

    //     let accountPublicKey: String = e.0.to_string();
    //     // let accountSS58AddressString: Vec<u8> = hex::decode(accountPublicKey.clone()).unwrap();
    //     // note: arbitrary bytes generally aren't valid utf8
    //     // let accountSS58AddressStr: &str = std::str::from_utf8(&accountSS58AddressString).unwrap();
    //     let accountSS58Address: AccountId32 = AccountId32::from_str(&accountPublicKey).unwrap();
    //     // let accountSS58Address: AccountId32 = Decode::decode(&mut accountPublicKey.as_slice().clone()).unwrap();
    //     // let accountSS58Address: AccountId32 = Decode::decode(&mut "a6b34be9aa95c82927b112dacf99bac1e728acb0fbae849097c0f9150fa49c23".to_string().as_slice().clone()).unwrap();


    //     // convert Public Key (hex) without '0x' prefix to SS58 Address
    //     // let accountSS58Address: AccountId = hex![`${accountPublicKey.clone().as_str()}`].into();

    //     return (
    //         // accountPublicKey,
    //         accountSS58Address.clone(),
	// 		// <[u8; 32]>::from_hex("a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21").unwrap().unchecked_into(),
	// 		e.1.to_string().parse::<Balance>().unwrap(),
	// 	);
	// }).collect();

	// Ok(all_balances)
}
