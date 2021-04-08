use sp_core::{Pair, Public, sr25519};
use node_template_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig,
	SudoConfig, SystemConfig, WASM_BINARY, Signature, ContractsConfig
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::ChainType;
use sp_core::OpaquePeerId; 
// use node_template_runtime::NodeAuthorizationConfig;
use serde_json::map::Map;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			],
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_mainnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;
	// Token Info
	let mut properties = Map::new();
	properties.insert("tokenSymbol".into(), "GDT".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"GrandaoChain",
		// ID
		"Grandao_Chain",
		ChainType::Local,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
				authority_keys_from_seed("Bob"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
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
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(properties),
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
		}),
		pallet_aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		}),
		pallet_sudo: Some(SudoConfig {
			// Assign network admin rights.
			key: root_key,
		}),
		pallet_contracts: Some(ContractsConfig {
			current_schedule: pallet_contracts::Schedule {
				enable_println, // this should only be enabled on development chains
				..Default::default()
			},
		}),
		// pallet_node_authorization: Some(NodeAuthorizationConfig {
		// 	nodes: vec![
		// 		(
		// 			OpaquePeerId(bs58::decode("12D3KooWLrtav3RP8jBRAH2txWzuzWavnqWzyMvgN3tPpf8o8cu6").into_vec().unwrap()),
		// 			endowed_accounts[0].clone()
		// 		),
		// 		(
		// 			OpaquePeerId(bs58::decode("12D3KooWQgoeQadA3DmC9D9HHYV6aA2Tg4e82xyDTKXvw9HmHESo").into_vec().unwrap()),
		// 			endowed_accounts[1].clone()
		// 		),
		// 		(
		// 			OpaquePeerId(bs58::decode("12D3KooWJFAQhGWAxsf5v5MNzQnZtm1nw6e2cC979gnSgoALzxDC").into_vec().unwrap()),
		// 			endowed_accounts[1].clone()
		// 		),
		// 		(
		// 			OpaquePeerId(bs58::decode("12D3KooWKCBtHRnDuyRFFaDWn1RamiQwNhahhPHoCd68JAMx9fiQ").into_vec().unwrap()),
		// 			endowed_accounts[1].clone()
		// 		),
		// 		(
		// 			OpaquePeerId(bs58::decode("12D3KooWRNCoCC7pMVGPeJT8KThZWiqDmxaWtCbqhx3SLfLoZ8yg").into_vec().unwrap()),
		// 			endowed_accounts[1].clone()
		// 		),
		// 	],
		// }),
		// orml_tokens: Some(TokensConfig {
		// 	endowed_accounts: endowed_accounts
		// 	.iter()
		// 	.flat_map(|x| {
		// 		vec![
		// 			(x.clone(), 1, 10u128.pow(16)),
		// 			(x.clone(), 2, 10u128.pow(16)),
		// 		]
		// 	})
		// 	.collect(),
		// }),

	}
}
