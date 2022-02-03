use module_primitives::{
    // types::*,
    types::{AccountId, Balance},
};
// use hex_literal::hex;
use serde::{Deserialize, Serialize};
use serde_json::{Result};
use sp_core::{
    crypto::{UncheckedFrom, UncheckedInto, Wraps},
};
use std::fs::File;
use std::io::Read;
use hex::FromHex;

#[derive(Serialize, Deserialize)]
struct Allocation {
    balances: Vec<(String, String)>
}

// reference/credits: https://github.com/hicommonwealth/edgeware-node/commit/a037f0af1e24d7a9a0a3a7e79662c27fb3ad2f5a
pub fn get_allocation()
	-> Result<(
		Vec<(AccountId, Balance)>
	)> {
    let mut file = File::open("node/src/genesis.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

	let json: Allocation = serde_json::from_str(&data)?;;
	let balances_json = json.balances;

	let balances: Vec<(AccountId, Balance)> = balances_json.into_iter().map(|e| {
		return (
			<[u8; 32]>::from_hex(e.0).unwrap().unchecked_into(),
			e.1.to_string().parse::<Balance>().unwrap(),
		);
	}).collect();
	Ok((balances))
}
