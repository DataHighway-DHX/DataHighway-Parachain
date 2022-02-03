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
use std::fs::File;
use std::io::Read;
use hex::FromHex;

#[derive(Serialize, Deserialize)]
struct Allocation {
    balances: Vec<(String, String)>
}

// reference/credits: https://github.com/hicommonwealth/edgeware-node/commit/a037f0af1e24d7a9a0a3a7e79662c27fb3ad2f5a
pub fn get_allocation()
	-> Result<Vec<(AccountId, Balance)>> {
    log::info!("loading genesis.json file...");
    let mut file = File::open("node/src/genesis.json").unwrap();
    log::info!("Successfully loaded genesis.json file");
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

	let json: Allocation = serde_json::from_str(&data)?;;
	let balances_json = json.balances;

	let balances: Vec<(AccountId, Balance)> = balances_json.into_iter().map(|e| {
        let a: AccountId = hex!["a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21"].into();
        log::info!("public key from hex: {:#?}", a.clone());
		return (
            // TryFrom::try_from(&[0; 64][..]).unwrap();
            // Balance::try_from(NumberOrHex::deserialize(deserializer)?)
			// .map_err(|_| D::Error::custom("Cannot decode NumberOrHex to Balance"))

            a.clone(),
			// <[u8; 32]>::from_hex("a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21").unwrap().unchecked_into(),
			e.1.to_string().parse::<Balance>().unwrap(),
		);
	}).collect();
	Ok(balances)
}
