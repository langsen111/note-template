use sp_core::{crypto::UncheckedInto, Pair, Public, sr25519};
use node_template_runtime::{
	AccountId, BalancesConfig, GenesisConfig, GrandpaConfig, AuthorityDiscoveryConfig,
	SudoConfig, SystemConfig, WASM_BINARY, Signature, ContractsConfig, ImOnlineConfig,
	CouncilConfig, TechnicalCommitteeConfig, IndicesConfig,
	opaque::SessionKeys, SessionConfig, StakingConfig, StakerStatus, DOLLARS, BDTS,
};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sp_runtime::{Perbill};
use sc_service::ChainType;
// use sp_core::OpaquePeerId; 
use serde_json::map::Map;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use hex_literal::hex;



// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
type AccountPublic = <Signature as Verify>::Signer;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

// 加session_key
fn session_keys(
    babe: BabeId,
    grandpa: GrandpaId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys { babe, grandpa, im_online, authority_discovery, }
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
pub fn authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, BabeId, GrandpaId, ImOnlineId, AuthorityDiscoveryId,) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;
	let mut properties = Map::new();
	properties.insert("tokenSymbol".into(), "test".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());
	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![authority_keys_from_seed("Alice")],
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
		Some(properties),
		// Extensions
		None,
	))
}

pub fn local_mainnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;
	// Token Info
	let mut properties = Map::new();
	properties.insert("tokenSymbol".into(), "test".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"TestChain",
		// ID
		"Test_Chain",
		ChainType::Local,
		move || mainnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				(
					//     5FEpjDuepNjSTjqeQnQXuFcV54pFeZ6YBdAzCd7pi4kG3xTN
					hex!["8c6c494fe5b23711365cdf544fbc344c3fb89dbaa3ff4b37df4960758bcef137"].into(),
					//     5FEpjDuepNjSTjqeQnQXuFcV54pFeZ6YBdAzCd7pi4kG3xTN
					hex!["8c6c494fe5b23711365cdf544fbc344c3fb89dbaa3ff4b37df4960758bcef137"].into(),
					//     5CrUZ9vuAKmZwWTknVJXaph8xbpWMeufQht9cGWdYsSMkTUa
					hex!["451b3e9b67ffea5e90b61e23396451a336e1449620bba3e13fbb96e187007c1a"].unchecked_into(),
					//5GTyALyDv9EFARPWrotf8yBJ3F3zSyk8NtUqcrtiBDVkbFLb
					hex!["c2af193a251dee1765136b0ae47647c110ac1225b23a157d6ef6629b1c93fe39"].unchecked_into(),
					//5GTyALyDv9EFARPWrotf8yBJ3F3zSyk8NtUqcrtiBDVkbFLb
					hex!["c2af193a251dee1765136b0ae47647c110ac1225b23a157d6ef6629b1c93fe39"].unchecked_into(),
					//5GTyALyDv9EFARPWrotf8yBJ3F3zSyk8NtUqcrtiBDVkbFLb
					hex!["c2af193a251dee1765136b0ae47647c110ac1225b23a157d6ef6629b1c93fe39"].unchecked_into(),
				),
				(
					//     5EyKNvvHTqCiF1mtSj3FSxkt6CRJ85LaE6ckGQ91x6CNfXqc
					hex!["8098ce08491726c0c14879a51fa12f166b55fbc2533e9ecebca31dfcb89df20d"].into(),
					//     5EyKNvvHTqCiF1mtSj3FSxkt6CRJ85LaE6ckGQ91x6CNfXqc
					hex!["8098ce08491726c0c14879a51fa12f166b55fbc2533e9ecebca31dfcb89df20d"].into(),
					//5HWDxcXHPxSowKDXSSKLEkUxXymXw2FA9zKyAwYw7nJ8KpYL
					hex!["f0a3a2eab48b0e51e8d89732d15da0164eb36951c4db3bd33879b0b343619ba7"].unchecked_into(),
					//5Fgn5eu1dhHemGLbHRgFuhdjjTHPuGt6UbLmwd2bi7JonwAG
					hex!["a037c0f83b7ebea2179165f987c6094d5b39e7addc1d2e09edf4a5fa6ebcac32"].unchecked_into(),
					//5Fgn5eu1dhHemGLbHRgFuhdjjTHPuGt6UbLmwd2bi7JonwAG
					hex!["a037c0f83b7ebea2179165f987c6094d5b39e7addc1d2e09edf4a5fa6ebcac32"].unchecked_into(),
					//5Fgn5eu1dhHemGLbHRgFuhdjjTHPuGt6UbLmwd2bi7JonwAG
					hex!["a037c0f83b7ebea2179165f987c6094d5b39e7addc1d2e09edf4a5fa6ebcac32"].unchecked_into(),
				),
				(
					//     5HdJayk1fELmjE5uRMR92haSLdYFvksi13U1EihvPCz2hoce
					hex!["f609ee1a21f29af54c84a8e0567333ec3170a2f987d1b74da3fa9c8afbb52f59"].into(),
					//     5HdJayk1fELmjE5uRMR92haSLdYFvksi13U1EihvPCz2hoce
					hex!["f609ee1a21f29af54c84a8e0567333ec3170a2f987d1b74da3fa9c8afbb52f59"].into(),
					//5H1TccKGpCsVM4STCELgHQAq5cMXXXBRSnJETy7hiZAUGZav
					hex!["dab37ca3624720b03aa2fdf4f2b436041ff151f0e3975f7b9c79e52030ae781e"].unchecked_into(),
					//5HGxatQ8j4HtoDiwUvT8gL3HMrXBwP4dMBQQPaYpvR6W2Ztc
					hex!["7a256c0498e35373006232ae18e18ec44c80c9d73aed563100fc8b7e0cf99001"].unchecked_into(),
					//5HGxatQ8j4HtoDiwUvT8gL3HMrXBwP4dMBQQPaYpvR6W2Ztc
					hex!["7a256c0498e35373006232ae18e18ec44c80c9d73aed563100fc8b7e0cf99001"].unchecked_into(),
					//5HGxatQ8j4HtoDiwUvT8gL3HMrXBwP4dMBQQPaYpvR6W2Ztc
					hex!["7a256c0498e35373006232ae18e18ec44c80c9d73aed563100fc8b7e0cf99001"].unchecked_into(),
				),
			],
			// Sudo account
			hex!["22e8829c160ada39adec02de745c8d43654b4ef89ebb5e8e7868516e5aa85c69"].into(),
			// Pre-funded accounts
			vec![],
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
fn mainnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId, AuthorityDiscoveryId)>,
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
			// Intergalactic initial supply
			balances: vec![
				(
					// Intergalactic BDTS Tokens 15%
					hex!["22e8829c160ada39adec02de745c8d43654b4ef89ebb5e8e7868516e5aa85c69"].into(),
					(1_500_000_000u128 * BDTS) - (3 * INITIAL_STAKING),
				),
				(
					// Treasury for rewards 3%
					hex!["5632419b258f8c13dbb46a49d90a24374388fb4c9a6f678104f6f77087053632"].into(),
					300_000_000 * BDTS,
				),
				(
					// Intergalactic Validator01
					hex!["8c6c494fe5b23711365cdf544fbc344c3fb89dbaa3ff4b37df4960758bcef137"].into(),
					INITIAL_STAKING,
				),
				(
					// Intergalactic Validator02
					hex!["8098ce08491726c0c14879a51fa12f166b55fbc2533e9ecebca31dfcb89df20d"].into(),
					INITIAL_STAKING,
				),
				(
					// Intergalactic Validator03
					hex!["f609ee1a21f29af54c84a8e0567333ec3170a2f987d1b74da3fa9c8afbb52f59"].into(),
					INITIAL_STAKING,
				),
			],
		}),
		pallet_babe: Some(Default::default()),
		pallet_grandpa: Some(Default::default()),
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),
		pallet_im_online: Some(ImOnlineConfig { keys: vec![] }),
		pallet_collective_Instance1: Some(CouncilConfig {
			members: vec![],
			phantom: Default::default(),
		}),
		pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		}),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_elections_phragmen: Some(Default::default()),
		pallet_treasury: Some(Default::default()),
		pallet_indices: Some(IndicesConfig { indices: vec![] }),


		pallet_session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(),
				 x.0.clone(),
				 session_keys(
					 x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()
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
	}
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId, AuthorityDiscoveryId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	enable_println: bool,
) -> GenesisConfig {
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
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),
		pallet_im_online: Some(ImOnlineConfig { keys: vec![] }),
		pallet_collective_Instance1: Some(CouncilConfig {
			members: vec![],
			phantom: Default::default(),
		}),
		pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		}),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_elections_phragmen: Some(Default::default()),
		pallet_treasury: Some(Default::default()),
		pallet_indices: Some(IndicesConfig { indices: vec![] }),


		pallet_session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(),
				 x.0.clone(),
				 session_keys(
					 x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()
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
	}
}
