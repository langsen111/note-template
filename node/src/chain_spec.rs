use sp_core::{Pair, Public, sr25519};
use node_template_runtime::{
	AccountId, BalancesConfig, GenesisConfig, GrandpaConfig,
	SudoConfig, SystemConfig, WASM_BINARY, Signature, ContractsConfig,
	opaque::SessionKeys, SessionConfig, StakingConfig, StakerStatus,DOLLARS,
};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sp_runtime::{Perbill};
use sc_service::ChainType;
use sp_core::OpaquePeerId; 
use serde_json::map::Map;


// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
type AccountPublic = <Signature as Verify>::Signer;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

// 加session_key
fn session_keys(
    babe: BabeId,
    grandpa: GrandpaId,
) -> SessionKeys {
    SessionKeys { babe, grandpa }
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Babe authority key.
pub fn authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, BabeId, GrandpaId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
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
	initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	enable_println: bool,
) -> GenesisConfig {
	// const INITIAL_BALANCE: u128 = 1_000_000 * DOLLARS;
	const INITIAL_STAKING: u128 = 100_000 * DOLLARS;

	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_sudo: Some(SudoConfig { key: root_key.clone() }),
		pallet_balances: Some(BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
		}),
		pallet_babe: Some(Default::default()),
		pallet_grandpa: Some(Default::default()),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(),
				 x.0.clone(),
				 session_keys(
					 x.2.clone(), x.3.clone()
				 ))
			}).collect::<Vec<_>>(),
		}),
		pallet_contracts: Some(ContractsConfig {
			current_schedule: pallet_contracts::Schedule {
				enable_println, // this should only be enabled on development chains
				..Default::default()
			},
		}),
		pallet_staking: Some(StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), INITIAL_STAKING, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
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
