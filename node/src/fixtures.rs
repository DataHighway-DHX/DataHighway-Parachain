use module_primitives::{
    // types::*,
    types::{AccountId, Balance},
};
use hex_literal::hex;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{Result};
use sp_core::{
    crypto::{UncheckedFrom, UncheckedInto, Wraps},
};
use sp_runtime::{AccountId32};
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use hex::FromHex;

#[derive(Serialize, Deserialize)]
struct Allocation {
    balances: Vec<(String, String)>
}

// reference/credits: https://github.com/hicommonwealth/edgeware-node/commit/a037f0af1e24d7a9a0a3a7e79662c27fb3ad2f5a
pub fn get_allocation()
	-> Result<Vec<(AccountId, Balance)>> {
    let mut file = File::open("node/src/genesis.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

	let json: Allocation = serde_json::from_str(&data)?;;
	let balances_json = json.balances;

	let balances: Vec<(AccountId, Balance)> = balances_json.into_iter().map(|e| {
        // let accountPublicKey: AccountId = e.0.to_string().parse::<AccountId>().unwrap();
        // convert Public Key (hex) without '0x' prefix to SS58 Address
        // let accountSS58Address: AccountId = hex![`${accountPublicKey.clone().as_str()}`].into();
        let a = AccountId32::from_str("4MkLjys3KYVtRKBWBeNUSYxymqXK3C8vKzXZuSroWv3cVhqp");
        return (
            a.unwrap(),
            // accountSS58Address.clone(),
			// <[u8; 32]>::from_hex("a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21").unwrap().unchecked_into(),
			e.1.to_string().parse::<Balance>().unwrap(),
		);
	}).collect();
	Ok(balances)
}
