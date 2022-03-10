use datahighway_parachain_runtime::{
    AccountId,
    Balance,
};
use codec::{
    Decode, Encode,
};
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use serde_json;
use sp_core::{
    crypto::{UncheckedFrom, UncheckedInto, Wraps},
};
use sp_runtime::{AccountId32};
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

// reference/credits: https://github.com/hicommonwealth/edgeware-node/commit/a037f0af1e24d7a9a0a3a7e79662c27fb3ad2f5a
// reference/credits: https://substrate.stackexchange.com/questions/156/how-to-overwrite-endowed-account-balances-using-additional-accounts-and-balances
pub fn get_allocation(endowed_accounts_with_balances: Vec<(AccountId32, Balance)>)
	-> Result<Vec<(AccountId32, Balance)>, String> {

    // only use this instead of change genesis.json to be like the following without the data being stored
    // under a "balances" property https://github.com/ltfschoen/substrate-parachain-template/commit/68e9e8db4fed981c83705ee472c7e43a9ec2810f#diff-03ba6e560e063bae9ad8da38df998bafd6ac9c236bc8fed1ac6d610f01d1778dR1
    let json_data = &include_bytes!("genesis.json")[..];
    let balances_json: Vec<(AccountId32, String)> = serde_json::from_slice(json_data).unwrap();

    let mut combined_balances: Vec<(AccountId32, Balance)> = vec![];

    if endowed_accounts_with_balances.len() != 0 {
        for e in endowed_accounts_with_balances {
            let account_public_key_endowed: String = e.0.to_string();
            let account_balance_endowed: Balance = e.1.to_string().parse::<Balance>().unwrap();
            let account_ss58_address_endowed: AccountId32 = AccountId32::from_str(&account_public_key_endowed).unwrap();
            combined_balances.push((account_ss58_address_endowed.clone(), account_balance_endowed.clone()));
        }
    }

    if balances_json.len() != 0 {
        // overwrite any existing account balances in endowed using the accounts from the json file
        for e in balances_json {
            let account_public_key_json: String = e.0.to_string();
            let account_balance_json: Balance = e.1.to_string().parse::<Balance>().unwrap();
            let account_ss58_address_json: AccountId32 = AccountId32::from_str(&account_public_key_json).unwrap();
            let index_of_match = combined_balances.clone().iter().position(|x| x.0 == account_ss58_address_json.clone());

            if let Some(idx) = index_of_match.clone() {
                // match between endowed account and this json file account so overwriting its balance value
                combined_balances[idx] = (combined_balances[idx].0.clone(), account_balance_json.clone());
            } else {
                // no match between endowed account and this json file account so adding it to the list to be endowed
                combined_balances.push((account_ss58_address_json.clone(), account_balance_json.clone()));
            }
        }
    }

    Ok(combined_balances.clone())
}
