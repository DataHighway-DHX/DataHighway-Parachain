use module_primitives::{
    // types::*,
    types::{AccountId, Balance},
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
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
	Unknown,
}

// reference/credits: https://github.com/hicommonwealth/edgeware-node/commit/a037f0af1e24d7a9a0a3a7e79662c27fb3ad2f5a
pub fn get_allocation(endowed_accounts_with_balances: Vec<(AccountId, Balance)>)
	-> Result<Vec<(AccountId, Balance)>, Error> {
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
}
