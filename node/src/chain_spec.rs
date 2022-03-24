use crate::fixtures::get_allocation;
use cumulus_primitives_core::ParaId;
use datahighway_parachain_runtime::{
    AuraId,
    AuraConfig,
    BalancesConfig,
    CollatorSelectionConfig,
    CouncilConfig,
    DemocracyConfig,
    ElectionsConfig,
    GenesisConfig,
    IndicesConfig,
    SessionConfig,
    SessionKeys,
    SudoConfig,
    SystemConfig,
    TechnicalCommitteeConfig,
    TechnicalMembershipConfig,
    TransactionPaymentConfig,
    TreasuryConfig,
};
use module_primitives::{
    constants::currency::{
        DOLLARS,
        EXISTENTIAL_DEPOSIT,
    },
    types::{
        AccountId,
        Balance,
        Signature,
    },
};
// required for AccountId::from_str
use std::str::FromStr;
use log::{error, info, debug, trace};
use hex as hex_runtime; // for runtime string parsing use hex_runtime::encode("...");
use hex_literal::{
    hex, // for parsing string literal at compile time use hex!("...");
};
use sc_chain_spec::{
    ChainSpecExtension,
    ChainSpecGroup,
};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::map::Map;
use sp_core::{
    crypto::{
        UncheckedFrom,
        UncheckedInto,
        Wraps,
    },
    sr25519,
    Pair,
    Public,
};
use sp_runtime::{AccountId32};
use sp_runtime::traits::{
    IdentifyAccount,
    Verify,
};
pub use sp_runtime::{
    Perbill,
    Permill,
};

const ROCOCO_DEV_PROTOCOL_ID: &str = "dhx-rococo-dev";
const ROCOCO_LOCAL_PROTOCOL_ID: &str = "dhx-rococo-local";
const ROCOCO_SPREEHAFEN_PROTOCOL_ID: &str = "dhx-rococo-spreehafen";
const CHACHACHA_DEV_PROTOCOL_ID: &str = "dhx-chachacha-dev";
const CHACHACHA_LOCAL_PROTOCOL_ID: &str = "dhx-chachacha-local";
const CHACHACHA_SPREEHAFEN_PROTOCOL_ID: &str = "dhx-chachacha-spreehafen";
const WESTEND_DEV_PROTOCOL_ID: &str = "dhx-westend-dev";
const WESTEND_LOCAL_PROTOCOL_ID: &str = "dhx-westend-local";
const WESTEND_BAIKAL_PROTOCOL_ID: &str = "dhx-westend-baikal";
const KUSAMA_DEV_PROTOCOL_ID: &str = "dhx-kusama-dev";
const KUSAMA_LOCAL_PROTOCOL_ID: &str = "dhx-kusama-local";
const KUSAMA_TANGANIKA_PROTOCOL_ID: &str = "dhx-kusama-tanganika";

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

// Note this is the URL for the telemetry server
const POLKADOT_STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Helper function to generate a crypto pair from seed
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
    get_public_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_public_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn datahighway_session_keys(keys: AuraId) -> SessionKeys {
    SessionKeys { aura: keys }
}

// DHX DAO Unlocked Reserves Balance
// Given a Treasury ModuleId in runtime parameter_types of
// `py/trsry`, we convert that to its associated address
// using Module ID" to Address" at https://www.shawntabrizi.com/substrate-js-utilities/,
// which generates 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z,
// and find its corresponding hex value by pasting the address into
// "AccountId to Hex" at that same link to return
// 6d6f646c70792f74727372790000000000000000000000000000000000000000.
// But since DataHighway is using an SS58 address prefix of 33 instead of
// Substrate's default of 42, the address corresponds to
// 4LTFqiD6H6g8a7ur9WH4RxhWx2givWfK7o5EDed3ai1nYTvk.
// This is treasury's account_id.
//
// In the older version of Substrate 2 it did not have instantiable support for treasury
// but was later supported in Substrate 3 and was fixed here
// https://github.com/paritytech/substrate/pull/7058
//
// Since we are using Substrate 3, we may transfer funds directly to the Treasury,
// which will hold the DHX DAO Unlocked Reserves Balance.
//
// Note: The original DataHighway Testnet Genesis has used:
//   5FmxcuFwGK7kPmQCB3zhk3HtxxJUyb3WjxosF8jvnkrVRLUG
//   4Mh2HyPJohFCzEm22G5VLvu59b1qUwNq3VpghyxDd4W6tJW9
//   hex: a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21
//
// However, the DataHighway Westlake Mainnet and DataHighway Parachain will transfer the funds to:
//   4LTFqiD6H6g8a7ur9WH4RxhWx2givWfK7o5EDed3ai1nYTvk
//   6d6f646c70792f74727372790000000000000000000000000000000000000000
//
// To transfer funds from the Treasury, either the Sudo user needs to
// call the `forceTransfer` extrinsic to transfer funds from the Treasury,
// or a proposal is required.

// note: we cannot use constants so a constant function has been used instead
// https://datahighway.subscan.io/tools/format_transform
// 6d6f646c70792f74727372790000000000000000000000000000000000000000
pub fn dhx_unlocked_reserves_account() -> AccountId {
    return AccountId32::from_str(&"4LTFqiD6H6g8a7ur9WH4RxhWx2givWfK7o5EDed3ai1nYTvk".to_string()).unwrap();
}
// c8c0ee501c4b115f08f677082b0f2beb59bd18f54f141588792e989bfb54e415
pub fn sudo_account_rococo_and_chachacha() -> AccountId {
    return AccountId32::from_str(&"4NWzRKnSjZcPN1sG1oxRHK1bZkygH5xMLJKrexrgWc9o986s".to_string()).unwrap();
}
// 4842a3314ad10a4e0053b59658f50b3fc5f1b6a9bee98608813a4b399aa3bf38
pub fn sudo_account_westend_baikal() -> AccountId {
    return AccountId32::from_str(&"4KcWmqsDBG1niDXsX31BVs73HMhD8gE63mgJATA98fwRkjG9".to_string()).unwrap();
}
// 2402f0e0ce5856bb7224525aa9ab0408e4b75cf98d45bd0248a49d2bef01ee65
pub fn sudo_account_kusama_tanganika() -> AccountId {
    return AccountId32::from_str(&"4Jnz8PpQoxfTpFdejpJu7VQUMv5zWeHqJpwXm8uuPuWGwSym".to_string()).unwrap();
}

pub fn datahighway_rococo_development_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        // Name
        "DataHighway Rococo Development Testnet",
        // ID
        "datahighway-rococo-dev",
        ChainType::Development,
        move || {
            dev_genesis(
                // Initial collators
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                // Sudo root key
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Endowed keys
                vec![
                    dhx_unlocked_reserves_account(),
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                // Parachain ID
                2000.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(ROCOCO_DEV_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "rococo-dev".into(),
            para_id: 2000,
        },
    )
}

pub fn datahighway_rococo_local_testnet_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        // Name
        "DataHighway Rococo Local Testnet",
        // ID
        "datahighway-rococo-local",
        ChainType::Local,
        move || {
            testnet_genesis(
                // Initial collators
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                // Sudo root key
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Endowed keys
                vec![
                    dhx_unlocked_reserves_account(),
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                // Parachain ID
                2000.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(ROCOCO_LOCAL_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "rococo-local".into(),
            para_id: 2000,
        },
    )
}

pub fn datahighway_chachacha_development_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        // Name
        "DataHighway ChaChaCha Development Testnet",
        // ID
        "datahighway-chachacha-dev",
        ChainType::Development,
        move || {
            dev_genesis(
                // Initial collators
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                // Sudo root keys
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Endowed keys
                vec![
                    dhx_unlocked_reserves_account(),
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                // Parachain ID
                2000.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(CHACHACHA_DEV_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "chachacha-dev".into(),
            para_id: 2000,
        },
    )
}

pub fn datahighway_chachacha_local_testnet_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        // Name
        "DataHighway ChaChaCha Local Testnet",
        // ID
        "datahighway-chachacha-local",
        ChainType::Local,
        move || {
            testnet_genesis(
                // Initial collators
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                // Sudo root key
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Endowed keys
                vec![
                    dhx_unlocked_reserves_account(),
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                // Parachain ID
                2000.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(CHACHACHA_LOCAL_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "chachacha-local".into(),
            para_id: 2000,
        },
    )
}

pub fn datahighway_rococo_parachain_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        "DataHighway Spreehafen Rococo Parachain Testnet",
        "datahighway-spreehafen-rococo-parachain-testnet",
        ChainType::Live,
        move || {
            spreehafen_testnet_genesis(
                // Initial collators
                vec![
                    // authority #1
                    (
                        // account id
                        hex!["106c208ac262aa3733629ad0860d0dc72d8b9152e1cdcab497949a3f9504517a"].into(),
                        // aura
                        hex!["106c208ac262aa3733629ad0860d0dc72d8b9152e1cdcab497949a3f9504517a"].unchecked_into()
                    ),
                    // authority #2
                    (
                        // account id
                        hex!["0234df0fce3e763e02b6644e589bd256bbd45121bdf6d98dd1cf1072b6228859"].into(),
                        // aura
                        hex!["0234df0fce3e763e02b6644e589bd256bbd45121bdf6d98dd1cf1072b6228859"].unchecked_into()
                    ),
                    // authority #3
                    (
                        // account id
                        hex!["02fe175463b5c7c378416e06780f7c60520d4dbcf759a7634a311e562e13a765"].into(),
                        // aura
                        hex!["02fe175463b5c7c378416e06780f7c60520d4dbcf759a7634a311e562e13a765"].unchecked_into()
                    ),
                    // authority #4
                    (
                        // account id
                        hex!["ea239700d67f53d30e39bee0c056f1165a6fb59ad4d5dd495c06d001af366c02"].into(),
                        // aura
                        hex!["ea239700d67f53d30e39bee0c056f1165a6fb59ad4d5dd495c06d001af366c02"].unchecked_into()
                    )

                ],
                // Sudo root key
                sudo_account_rococo_and_chachacha(),
                // Endowed keys
                vec![
                    // Endow the Sudo account to cover transaction fees
                    sudo_account_rococo_and_chachacha(),
                    // Endow this account with the DHX DAO Unlocked Reserves Balance
                    // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                    dhx_unlocked_reserves_account(),
                    // Endow these accounts with a balance so they may bond as authorities
                    // authority #1 stash
                    hex!["b2f1decb9c6a1e6df2cd7e7b73d6c7eada3683d958b2fed451fb045d2f7cdb55"].into(),
                    // authority #1 controller
                    hex!["467da0333f16ce430bfa18fb8c25cfbbc49f35946370989280aaf3142fff7344"].into(),
                    // authority #1 aura
                    hex!["106c208ac262aa3733629ad0860d0dc72d8b9152e1cdcab497949a3f9504517a"].into(),
                    // authority #2 stash
                    hex!["b2347d115c9300a433a59b0ef321430a6d418d0555a6a41dfebe99fb86765110"].into(),
                    // authority #2 controller
                    hex!["ac691d2b336f8347a22eb3831b381e4adac45ab6f0ad85abc1336633313f173d"].into(),
                    // authority #2 aura
                    hex!["0234df0fce3e763e02b6644e589bd256bbd45121bdf6d98dd1cf1072b6228859"].into(),
                    // authority #3 stash
                    hex!["f4062d6d4ac30ea04659b24994cc0ebf249fed1591e6cf1c25d5f4f78e78bb6b"].into(),
                    // authority #3 controller
                    hex!["4cad3775c026114d4a6e965f72caf11c18eb03ea7a3b4c0516f4cb8856b2575f"].into(),
                    // authority #3 aura
                    hex!["02fe175463b5c7c378416e06780f7c60520d4dbcf759a7634a311e562e13a765"].into(),
                    // authority #4 stash
                    hex!["a0d56496c02c203312ebce4a2804c7e0c31e34f983b9bc037f7c95f34e416613"].into(),
                    // authority #4 controller
                    hex!["6cd4eeb38c45a073d3c8e3ddd24e2502707060f33a1d92e082e32c106512500f"].into(),
                    // authority #4 aura
                    hex!["ea239700d67f53d30e39bee0c056f1165a6fb59ad4d5dd495c06d001af366c02"].into(),
                ],
                // Parachain ID
                2116.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(ROCOCO_SPREEHAFEN_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "rococo".into(),
            para_id: 2116,
        },
    )
}

pub fn datahighway_chachacha_parachain_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        "DataHighway Spreehafen ChaChaCha Parachain Testnet",
        "datahighway-spreehafen-chachacha-parachain-testnet",
        ChainType::Live,
        move || {
            spreehafen_testnet_genesis(
                // Initial collators
                vec![
                    // authority #1
                    (
                        // account id
                        hex!["106c208ac262aa3733629ad0860d0dc72d8b9152e1cdcab497949a3f9504517a"].into(),
                        // aura
                        hex!["106c208ac262aa3733629ad0860d0dc72d8b9152e1cdcab497949a3f9504517a"].unchecked_into()
                    ),
                    // authority #2
                    (
                        // account id
                        hex!["0234df0fce3e763e02b6644e589bd256bbd45121bdf6d98dd1cf1072b6228859"].into(),
                        // aura
                        hex!["0234df0fce3e763e02b6644e589bd256bbd45121bdf6d98dd1cf1072b6228859"].unchecked_into()
                    ),
                    // authority #3
                    (
                        // account id
                        hex!["02fe175463b5c7c378416e06780f7c60520d4dbcf759a7634a311e562e13a765"].into(),
                        // aura
                        hex!["02fe175463b5c7c378416e06780f7c60520d4dbcf759a7634a311e562e13a765"].unchecked_into()
                    ),
                    // authority #4
                    (
                        // account id
                        hex!["ea239700d67f53d30e39bee0c056f1165a6fb59ad4d5dd495c06d001af366c02"].into(),
                        // aura
                        hex!["ea239700d67f53d30e39bee0c056f1165a6fb59ad4d5dd495c06d001af366c02"].unchecked_into()
                    )

                ],
                // Sudo root key
                sudo_account_rococo_and_chachacha(),
                // Endowed keys
                vec![
                    // Endow the Sudo account to cover transaction fees
                    sudo_account_rococo_and_chachacha(),
                    // Endow this account with the DHX DAO Unlocked Reserves Balance
                    // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                    dhx_unlocked_reserves_account(),
                    // Endow these accounts with a balance so they may bond as authorities
                    // authority #1 stash
                    hex!["b2f1decb9c6a1e6df2cd7e7b73d6c7eada3683d958b2fed451fb045d2f7cdb55"].into(),
                    // authority #1 controller
                    hex!["467da0333f16ce430bfa18fb8c25cfbbc49f35946370989280aaf3142fff7344"].into(),
                    // authority #1 aura
                    hex!["106c208ac262aa3733629ad0860d0dc72d8b9152e1cdcab497949a3f9504517a"].into(),
                    // authority #2 stash
                    hex!["b2347d115c9300a433a59b0ef321430a6d418d0555a6a41dfebe99fb86765110"].into(),
                    // authority #2 controller
                    hex!["ac691d2b336f8347a22eb3831b381e4adac45ab6f0ad85abc1336633313f173d"].into(),
                    // authority #2 aura
                    hex!["0234df0fce3e763e02b6644e589bd256bbd45121bdf6d98dd1cf1072b6228859"].into(),
                    // authority #3 stash
                    hex!["f4062d6d4ac30ea04659b24994cc0ebf249fed1591e6cf1c25d5f4f78e78bb6b"].into(),
                    // authority #3 controller
                    hex!["4cad3775c026114d4a6e965f72caf11c18eb03ea7a3b4c0516f4cb8856b2575f"].into(),
                    // authority #3 aura
                    hex!["02fe175463b5c7c378416e06780f7c60520d4dbcf759a7634a311e562e13a765"].into(),
                    // authority #4 stash
                    hex!["a0d56496c02c203312ebce4a2804c7e0c31e34f983b9bc037f7c95f34e416613"].into(),
                    // authority #4 controller
                    hex!["6cd4eeb38c45a073d3c8e3ddd24e2502707060f33a1d92e082e32c106512500f"].into(),
                    // authority #4 aura
                    hex!["ea239700d67f53d30e39bee0c056f1165a6fb59ad4d5dd495c06d001af366c02"].into(),
                ],
                // Parachain ID
                2002.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(CHACHACHA_SPREEHAFEN_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "chachacha".into(),
            para_id: 2002,
        },
    )
}

pub fn datahighway_westend_development_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        // Name
        "DataHighway Westend Development Testnet",
        // ID
        "datahighway-westend-dev",
        ChainType::Development,
        move || {
            dev_genesis(
                // Initial collators
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                // Sudo root key
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Endowed keys
                vec![
                    dhx_unlocked_reserves_account(),
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                // Parachain ID
                2000.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(WESTEND_DEV_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "westend-dev".into(),
            para_id: 2000,
        },
    )
}

pub fn datahighway_westend_local_testnet_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        // Name
        "DataHighway Westend Local Testnet",
        // ID
        "datahighway-westend-local",
        ChainType::Local,
        move || {
            testnet_genesis(
                // Initial collators
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                // Sudo root key
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Endowed keys
                vec![
                    dhx_unlocked_reserves_account(),
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                // Parachain ID
                2000.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(WESTEND_LOCAL_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "westend-local".into(),
            para_id: 2000,
        },
    )
}

pub fn datahighway_kusama_development_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        // Name
        "DataHighway Kusama Development Testnet",
        // ID
        "datahighway-kusama-dev",
        ChainType::Development,
        move || {
            dev_genesis(
                // Initial collators
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                // Sudo root key
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Endowed keys
                vec![
                    dhx_unlocked_reserves_account(),
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                // Parachain ID
                2000.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(KUSAMA_DEV_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "kusama-dev".into(),
            para_id: 2000,
        },
    )
}

pub fn datahighway_kusama_local_testnet_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        // Name
        "DataHighway Kusama Local Testnet",
        // ID
        "datahighway-kusama-local",
        ChainType::Local,
        move || {
            testnet_genesis(
                // Initial collators
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                // Sudo root key
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Endowed keys
                vec![
                    dhx_unlocked_reserves_account(),
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                // Parachain ID
                2000.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(KUSAMA_LOCAL_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "kusama-local".into(),
            para_id: 2000,
        },
    )
}

pub fn datahighway_westend_parachain_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "BKL".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        "DataHighway Baikal Westend Parachain Testnet",
        "datahighway-baikal-westend-parachain-testnet",
        ChainType::Live,
        move || {
            baikal_testnet_genesis(
                // Initial collators
                vec![
                    // authority #1
                    (
                        // account
                        hex!["2628f7a7bb067a23daa14b1aa9f10ff44545d37907f2d5cefee905236944060a"].into(),
                        // aura
                        hex!["2628f7a7bb067a23daa14b1aa9f10ff44545d37907f2d5cefee905236944060a"].unchecked_into()
                    ),
                    // authority #2
                    (
                        // account
                        hex!["709f96ae975cd0cfafd98fb241810a2870d58fcfdbb1ee6892a8740525f4d871"].into(),
                        // aura
                        hex!["709f96ae975cd0cfafd98fb241810a2870d58fcfdbb1ee6892a8740525f4d871"].unchecked_into()
                    ),
                    // authority #3
                    (
                        // account
                        hex!["ce7f04896b8d13da7a4f3f0a49bf6c1d77076043a1184a993ce75d96f6e0ee56"].into(),
                        // aura
                        hex!["ce7f04896b8d13da7a4f3f0a49bf6c1d77076043a1184a993ce75d96f6e0ee56"].unchecked_into()
                    ),
                    // authority #4
                    (
                        // account
                        hex!["c27631914b41a8f58e24277158817d064a4144df430dd2cf7baeaa17414deb3e"].into(),
                        // aura
                        hex!["c27631914b41a8f58e24277158817d064a4144df430dd2cf7baeaa17414deb3e"].unchecked_into()
                    )
                ],
                // Sudo root key
                sudo_account_westend_baikal(),
                // Endowed keys
                vec![
                    // Endow the Sudo account to cover transaction fees
                    sudo_account_westend_baikal(),
                    // Endow this account with the DHX DAO Unlocked Reserves Balance
                    // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                    dhx_unlocked_reserves_account(),
                    // Endow these accounts with a balance so they may bond as authorities
                    // authority #1 stash
                    hex!["b41b286a78df1a87a07db8c8794923d8cc581c4b1a03d90be9ce46a03fbbaa2e"].into(),
                    // authority #1 controller
                    hex!["bece77da74ab38eadde718ca30a0e46a0a3c5827f289c73d331755a7aaf19a11"].into(),
                    // authority #1 aura
                    hex!["2628f7a7bb067a23daa14b1aa9f10ff44545d37907f2d5cefee905236944060a"].into(),
                    // authority #2 stash
                    hex!["8cbd45146df7ce640231639dfd1a78dfd0dfb4d873b13226378c297110d50505"].into(),
                    // authority #2 controller
                    hex!["2001d4a5b0e3c3ab39b88e7f85193a9a8340ca1b5803e9178f52dae126cd595b"].into(),
                    // authority #2 aura
                    hex!["709f96ae975cd0cfafd98fb241810a2870d58fcfdbb1ee6892a8740525f4d871"].into(),
                    // authority #3 stash
                    hex!["b20f2fab27d842763eb355ad978865e34f44da2fbf7a4182ab035d1bad34f021"].into(),
                    // authority #3 controller
                    hex!["1aaaef87d9a3ec62ddcc959730b5d1b89d162fe8e432b0792540069bba518431"].into(),
                    // authority #3 aura
                    hex!["ce7f04896b8d13da7a4f3f0a49bf6c1d77076043a1184a993ce75d96f6e0ee56"].into(),
                    // authority #4 stash
                    hex!["62a173fb0a5bf0651559d560f44afa3de55d60cb0e0a06c9d0e1fef81f41b80a"].into(),
                    // authority #4 controller
                    hex!["82e71bb9a9a8fc2aefbd17a41a4f7686cd95f46f3e3e0522caa6147289581562"].into(),
                    // authority #4 aura
                    hex!["c27631914b41a8f58e24277158817d064a4144df430dd2cf7baeaa17414deb3e"].into(),
                ],
                // Parachain ID
                2000.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(WESTEND_BAIKAL_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "westend".into(),
            para_id: 2000,
        },
    )
}

pub fn datahighway_kusama_parachain_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 33.into());
    ChainSpec::from_genesis(
        "DataHighway Tanganika Kusama Parachain",
        "datahighway-tanganika-kusama-parachain",
        ChainType::Live,
        move || {
            tanganika_testnet_genesis(
                // Initial collators
                vec![
                    // authority #1
                    (
                        // account
                        hex!["a8694c0c9e315e020844944ac76712c84f84a00007016e61c7e2f83fc56c5b3f"].into(),
                        // aura
                        hex!["a8694c0c9e315e020844944ac76712c84f84a00007016e61c7e2f83fc56c5b3f"].unchecked_into()
                    ),
                    // authority #2
                    (
                        // account
                        hex!["a8db9194388b3c038b126a5e2520515be2e989e3f380ce2cb5cf29d5a26c0522"].into(),
                        // aura
                        hex!["a8db9194388b3c038b126a5e2520515be2e989e3f380ce2cb5cf29d5a26c0522"].unchecked_into()
                    ),
                    // authority #3
                    (
                        // account
                        hex!["b8212af17ba93d9175748469afa0a74357712ff4571a36d347df58cf3821cd3d"].into(),
                        // aura
                        hex!["b8212af17ba93d9175748469afa0a74357712ff4571a36d347df58cf3821cd3d"].unchecked_into()
                    ),
                    // authority #4
                    (
                        // account
                        hex!["10a3d6854dc35e4b3fd77af4beda98f79dbe9edf5c29c14c8d57bec4bd733c0f"].into(),
                        // aura
                        hex!["10a3d6854dc35e4b3fd77af4beda98f79dbe9edf5c29c14c8d57bec4bd733c0f"].unchecked_into()
                    )
                ],
                // Sudo root key
                sudo_account_kusama_tanganika(),
                // Endowed keys
                vec![
                    // Endow the Sudo account to cover transaction fees
                    sudo_account_kusama_tanganika(),
                    // Endow this account with the DHX DAO Unlocked Reserves Balance
                    // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                    dhx_unlocked_reserves_account(),
                    // Endow these accounts with a balance so they may bond as authorities
                    // authority #1 stash
                    hex!["f8940eaa011b23f3469805062d1ae33c128caa6b10d71b04609f246cb947f92c"].into(),
                    // authority #1 controller
                    hex!["e409a7faebf39ba76f46bfac84c8001c1243b980f5bac89fdd887eed1401bb35"].into(),
                    // authority #1 aura
                    hex!["a8694c0c9e315e020844944ac76712c84f84a00007016e61c7e2f83fc56c5b3f"].into(),
                    // authority #2 stash
                    hex!["30a9048710bbc3791feb01e2c900f7290c09e124cd774b63950c52b8c6e5d644"].into(),
                    // authority #2 controller
                    hex!["a0b3f77eec476b584fc24631c6a957254bc3e2d9e91c8abb8038e40ba045471f"].into(),
                    // authority #2 aura
                    hex!["a8db9194388b3c038b126a5e2520515be2e989e3f380ce2cb5cf29d5a26c0522"].into(),
                    // authority #3 stash
                    hex!["a2616fd57d21ed85a2deb41bb0628645db5ba24e9dc26c912cfa54608bf21d01"].into(),
                    // authority #3 controller
                    hex!["46cfb03490de202950ea2433f0130730a3f84a4646acb6b10ff6510685457f40"].into(),
                    // authority #3 aura
                    hex!["b8212af17ba93d9175748469afa0a74357712ff4571a36d347df58cf3821cd3d"].into(),
                    // authority #4 stash
                    hex!["fa9089b3bcbad69451a162e1454a9e0aa9efc7bcdf9466f0a4bb762b4ed4755c"].into(),
                    // authority #4 controller
                    hex!["123c907b49233a2ccb6a4d92a1266b3e2feccc10e880e8659368a6338842ba7f"].into(),
                    // authority #4 aura
                    hex!["10a3d6854dc35e4b3fd77af4beda98f79dbe9edf5c29c14c8d57bec4bd733c0f"].into(),
                ],
                // Parachain ID
                2116.into(),
                // Enable println
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some(KUSAMA_TANGANIKA_PROTOCOL_ID),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "kusama".into(),
            para_id: 2116,
        },
    )
}

const INITIAL_ENDOWMENT: u128 = 10_000_000_000_000_000_000_u128; // 10 DHX
const INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE: u128 = 30_000_000_000_000_000_000_000_000_u128; // 30M DHX

fn get_balances(endowed_accounts: Vec<AccountId>) -> Vec<(AccountId32, Balance)> {
    let mut endowed_accounts_with_balances: Vec<(AccountId, Balance)> = vec![];
    for x in endowed_accounts {
        if x == dhx_unlocked_reserves_account() {
            endowed_accounts_with_balances.push((x, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE));
        } else {
            endowed_accounts_with_balances.push((x, INITIAL_ENDOWMENT));
        }
    }
    let allocation = get_allocation(endowed_accounts_with_balances.clone()).unwrap();
    return allocation;
}

fn spreehafen_testnet_genesis(
    invulnerables: Vec<(AccountId, AuraId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
    _enable_println: bool,
) -> GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();
    let hardspoon_balances = get_balances(endowed_accounts.clone());

    GenesisConfig {
        system: SystemConfig {
            code: datahighway_parachain_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
        },
        balances: BalancesConfig {
            balances: hardspoon_balances
                .iter()
                .cloned()
                .map(|x| (x.0.clone(), x.1.clone()))
                .collect(),
        },
        indices: IndicesConfig {
            indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        },
        treasury: TreasuryConfig::default(),
        sudo: SudoConfig {
            key: Some(root_key.clone()),
        },
        parachain_info: datahighway_parachain_runtime::ParachainInfoConfig {
            parachain_id: id,
        },
        collator_selection: CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
            ..Default::default()
        },
        session: SessionConfig {
            keys: invulnerables
                .into_iter()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                    // account id
                        acc,                            // validator id
                        datahighway_session_keys(aura), // session keys
                    )
                })
                .collect(),
        },
        democracy: DemocracyConfig::default(),
        elections: ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, INITIAL_ENDOWMENT))
                .collect(),
        },
        // https://github.com/paritytech/substrate/commit/d6ac9f551b71d9c7b69afcebfc68ace310ef74ee
        // collective_Instance1
        council: CouncilConfig::default(),
        // collective_Instance2
        technical_committee: TechnicalCommitteeConfig::default(),
        // it will panic if we pass anything to Aura. Session will take care of this instead.
        aura: Default::default(),
        // pallet_membership_Instance1
        technical_membership: TechnicalMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        },
        assets: Default::default(),
        transaction_payment: TransactionPaymentConfig::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
		polkadot_xcm: datahighway_parachain_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
    }
}

fn testnet_genesis(
    invulnerables: Vec<(AccountId, AuraId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
    _enable_println: bool,
) -> GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();
    let hardspoon_balances = get_balances(endowed_accounts.clone());

    GenesisConfig {
        system: SystemConfig {
            code: datahighway_parachain_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
        },
        balances: BalancesConfig {
            balances: hardspoon_balances
                .iter()
                .cloned()
                .map(|x| (x.0.clone(), x.1.clone()))
                .collect(),
        },
        indices: IndicesConfig {
            indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        },
        treasury: TreasuryConfig::default(),
        sudo: SudoConfig {
            key: Some(root_key.clone()),
        },
        parachain_info: datahighway_parachain_runtime::ParachainInfoConfig {
            parachain_id: id,
        },
        collator_selection: CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
            ..Default::default()
        },
        session: SessionConfig {
            keys: invulnerables
                .into_iter()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                    // account id
                        acc,                            // validator id
                        datahighway_session_keys(aura), // session keys
                    )
                })
                .collect(),
        },
        democracy: DemocracyConfig::default(),
        elections: ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, INITIAL_ENDOWMENT))
                .collect(),
        },
        // https://github.com/paritytech/substrate/commit/d6ac9f551b71d9c7b69afcebfc68ace310ef74ee
        // collective_Instance1
        council: CouncilConfig::default(),
        // collective_Instance2
        technical_committee: TechnicalCommitteeConfig::default(),
        // it will panic if we pass anything to Aura. Session will take care of this instead.
        aura: Default::default(),
        // pallet_membership_Instance1
        technical_membership: TechnicalMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        },
        assets: Default::default(),
        transaction_payment: TransactionPaymentConfig::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
		polkadot_xcm: datahighway_parachain_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
    }
}

fn dev_genesis(
    invulnerables: Vec<(AccountId, AuraId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
    _enable_println: bool,
) -> GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();
    let hardspoon_balances = get_balances(endowed_accounts.clone());

    GenesisConfig {
        system: SystemConfig {
            code: datahighway_parachain_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
        },
        balances: BalancesConfig {
            balances: hardspoon_balances
                .iter()
                .cloned()
                .map(|x| (x.0.clone(), x.1.clone()))
                .collect(),
        },
        indices: IndicesConfig {
            indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        },
        treasury: TreasuryConfig::default(),
        sudo: SudoConfig {
            key: Some(root_key.clone()),
        },
        parachain_info: datahighway_parachain_runtime::ParachainInfoConfig {
            parachain_id: id,
        },
        collator_selection: CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
            ..Default::default()
        },
        session: SessionConfig {
            keys: invulnerables
                .into_iter()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                    // account id
                        acc,                            // validator id
                        datahighway_session_keys(aura), // session keys
                    )
                })
                .collect(),
        },
        democracy: DemocracyConfig::default(),
        elections: ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, INITIAL_ENDOWMENT))
                .collect(),
        },
        // https://github.com/paritytech/substrate/commit/d6ac9f551b71d9c7b69afcebfc68ace310ef74ee
        // collective_Instance1
        council: CouncilConfig::default(),
        // collective_Instance2
        technical_committee: TechnicalCommitteeConfig::default(),
        // it will panic if we pass anything to Aura. Session will take care of this instead.
        aura: Default::default(),
        // pallet_membership_Instance1
        technical_membership: TechnicalMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        },
        assets: Default::default(),
        transaction_payment: TransactionPaymentConfig::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
		polkadot_xcm: datahighway_parachain_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
    }
}

fn baikal_testnet_genesis(
    invulnerables: Vec<(AccountId, AuraId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
    _enable_println: bool,
) -> GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();

    GenesisConfig {
        system: SystemConfig {
            code: datahighway_parachain_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
        },
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| {
                    // Insert Public key (hex) of the account without the 0x prefix below
                    if x == dhx_unlocked_reserves_account() {
                        // If we use println, then the top of the chain specification file that gets
                        // generated contains the println, and then we have to remove the println from
                        // the top of that file to generate the "raw" chain definition
                        // println!("endowed_account treasury {:?}", x.clone());
                        return (x, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);
                    } else {
                        // println!("endowed_account {:?}", x.clone());
                        return (x, INITIAL_ENDOWMENT);
                    }
                })
                .collect(),
        },
        indices: IndicesConfig {
            indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        },
        treasury: TreasuryConfig::default(),
        sudo: SudoConfig {
            key: Some(root_key.clone()),
        },
        parachain_info: datahighway_parachain_runtime::ParachainInfoConfig {
            parachain_id: id,
        },
        collator_selection: CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
            ..Default::default()
        },
        session: SessionConfig {
            keys: invulnerables
                .into_iter()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                    // account id
                        acc,                            // validator id
                        datahighway_session_keys(aura), // session keys
                    )
                })
                .collect(),
        },
        democracy: DemocracyConfig::default(),
        elections: ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, INITIAL_ENDOWMENT))
                .collect(),
        },
        // https://github.com/paritytech/substrate/commit/d6ac9f551b71d9c7b69afcebfc68ace310ef74ee
        // collective_Instance1
        council: CouncilConfig::default(),
        // collective_Instance2
        technical_committee: TechnicalCommitteeConfig::default(),
        // it will panic if we pass anything to Aura. Session will take care of this instead.
        aura: Default::default(),
        // pallet_membership_Instance1
        technical_membership: TechnicalMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        },
        assets: Default::default(),
        transaction_payment: TransactionPaymentConfig::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
		polkadot_xcm: datahighway_parachain_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
    }
}

fn tanganika_testnet_genesis(
    invulnerables: Vec<(AccountId, AuraId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
    _enable_println: bool,
) -> GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();
    let hardspoon_balances = get_balances(endowed_accounts.clone());

    GenesisConfig {
        system: SystemConfig {
            code: datahighway_parachain_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
        },
        balances: BalancesConfig {
            balances: hardspoon_balances
                .iter()
                .cloned()
                .map(|x| (x.0.clone(), x.1.clone()))
                .collect(),
        },
        indices: IndicesConfig {
            indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        },
        treasury: TreasuryConfig::default(),
        sudo: SudoConfig {
            key: Some(root_key.clone()),
        },
        parachain_info: datahighway_parachain_runtime::ParachainInfoConfig {
            parachain_id: id,
        },
        collator_selection: CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
            ..Default::default()
        },
        session: SessionConfig {
            keys: invulnerables
                .into_iter()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                    // account id
                        acc,                            // validator id
                        datahighway_session_keys(aura), // session keys
                    )
                })
                .collect(),
        },
        democracy: DemocracyConfig::default(),
        elections: ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, INITIAL_ENDOWMENT))
                .collect(),
        },
        // https://github.com/paritytech/substrate/commit/d6ac9f551b71d9c7b69afcebfc68ace310ef74ee
        // collective_Instance1
        council: CouncilConfig::default(),
        // collective_Instance2
        technical_committee: TechnicalCommitteeConfig::default(),
        // it will panic if we pass anything to Aura. Session will take care of this instead.
        aura: Default::default(),
        // pallet_membership_Instance1
        technical_membership: TechnicalMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        },
        assets: Default::default(),
        transaction_payment: TransactionPaymentConfig::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
		polkadot_xcm: datahighway_parachain_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
    }
}
