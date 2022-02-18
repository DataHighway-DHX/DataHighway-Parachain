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

// reference/credits: https://github.com/hicommonwealth/edgeware-node/commit/a037f0af1e24d7a9a0a3a7e79662c27fb3ad2f5a
pub fn get_allocation(endowed_accounts_with_balances: Vec<(AccountId, Balance)>)
	-> Result<Vec<(AccountId, Balance)>> {
    let json_data = &include_bytes!("genesis.json")[..];
    let accounts_with_balance = endowed_accounts_with_balances;
    let additional_accounts_with_balance: Vec<(AccountId, Balance)> = serde_json::from_slice(json_data).unwrap();
    let mut accounts = additional_accounts_with_balance.clone();

    accounts_with_balance.iter().for_each(|tup1| {
        for tup2 in additional_accounts_with_balance.iter() {
            if tup1.0 == tup2.0 {
                return;
            }
        }
        accounts.push(tup1.to_owned());
    });

    Ok(accounts)

	// let json: Allocation = serde_json::from_str(&data)?;
	// let balances_json = json.balances;

    // let mut combined_balances: Vec<(AccountId, Balance)> = vec![];

    // if endowed_accounts_with_balances.len() != 0 {
    //     for e in endowed_accounts_with_balances {
    //         let account_public_key_endowed: String = e.0.to_string();
    //         // println!("account_public_key_endowed {:#?}", account_public_key_endowed.clone());

    //         let account_balance_endowed: Balance = e.1.to_string().parse::<Balance>().unwrap();
    //         // println!("account_balance_endowed {:#?}", account_balance_endowed.clone());

    //         let account_ss58_address_endowed: AccountId32 = AccountId32::from_str(&account_public_key_endowed).unwrap();
    //         // println!("account_ss58_address_endowed {:#?}", account_ss58_address_endowed.clone());

    //         combined_balances.push((account_ss58_address_endowed.clone(), account_balance_endowed.clone()));
    //     }
    // }

    // if balances_json.len() != 0 {
    //     // overwrite any existing account balances in endowed using the accounts from the json file
    //     for e in balances_json {
    //         let account_public_key_json: String = e.0.to_string();
    //         // println!("account_public_key_json {:#?}", account_public_key_json.clone());

    //         let account_balance_json: Balance = e.1.to_string().parse::<Balance>().unwrap();
    //         // println!("account_balance_json {:#?}", account_balance_json.clone());

    //         let account_ss58_address_json: AccountId32 = AccountId32::from_str(&account_public_key_json).unwrap();
    //         // println!("account_ss58_address_json {:#?}", account_ss58_address_json.clone());

    //         let index_of_match = combined_balances.clone().iter().position(|x| x.0 == account_ss58_address_json.clone());

    //         if let Some(idx) = index_of_match.clone() {
    //             // println!("match b/w endowed and json for {:#?} so overwriting its balance value", account_ss58_address_json.clone());
    //             combined_balances[idx] = (combined_balances[idx].0.clone(), account_balance_json.clone());
    //         } else {
    //             // println!("no match b/w endowed and json for {:#?} so adding it to the list", e.0.clone());
    //             combined_balances.push((account_ss58_address_json.clone(), account_balance_json.clone()));
    //         }
    //     }
    // }

    // Ok(combined_balances.clone())
}
