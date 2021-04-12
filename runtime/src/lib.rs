#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_std::prelude::*;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	ApplyExtrinsicResult, generic, create_runtime_str, impl_opaque_keys, MultiSignature, ModuleId,
	curve::PiecewiseLinear,
	transaction_validity::{TransactionValidity, TransactionSource, TransactionPriority},
};
use sp_runtime::traits::{
	AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, Verify, IdentifyAccount, NumberFor,
	Zero, OpaqueKeys,
};
use sp_api::impl_runtime_apis;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_grandpa::fg_primitives;
use sp_version::RuntimeVersion;
#[cfg(feature = "std")]
use sp_version::NativeVersion;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_balances::Call as BalancesCall;
pub use sp_runtime::{Percent, Permill, Perbill};
pub use frame_support::{
	construct_runtime, parameter_types, StorageValue,
	traits::{KeyOwnerProofSystem, Randomness},
	weights::{
		Weight, IdentityFee, DispatchClass,
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
	},
};
use pallet_transaction_payment::CurrencyAdapter;
use pallet_contracts::weights::WeightInfo;
use frame_system::{
	limits::{BlockWeights, BlockLength}, EnsureRoot, EnsureOneOf
};
pub use pallet_staking::StakerStatus;
use pallet_session::historical as session_historical;
use orml_currencies::{BasicCurrencyAdapter, Currency};
use orml_traits::parameter_type_with_key;

/// Import the template pallet.
pub use pallet_template;

/// Import the grandao pallet.
pub use pallet_grandao;

/// Import the gdnft pallet.
pub use pallet_gdnft;
pub mod gdnft;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub babe: Babe,
			pub grandpa: Grandpa,
		}
	}
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("grandao-node"),
	impl_name: create_runtime_str!("grandao-node"),
	authoring_version: 1,
	spec_version: 101,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};


/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
// Kusama
pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = HOURS;

// 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// contracts
pub const BDTS: Balance = 1_000_000_000_000;    // 12bits
pub const DOLLARS: Balance = BDTS / 100;        // 10_000_000_000
pub const CENTS: Balance = DOLLARS / 100;       // 100_000_000
pub const MILLICENTS: Balance = CENTS / 1_000;  // 100_000

// Kusama
pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
}

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 2400;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub RuntimeBlockLength: BlockLength =
	BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_BLOCKS as u64;
    pub const ExpectedBlockTime: u64 = MILLISECS_PER_BLOCK;
    pub const ReportLongevity: u64 = 0; // by Skyh
        // BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get(); // Kusama
}

impl pallet_babe::Config for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type KeyOwnerProofSystem = (); // Historical;
    type KeyOwnerProof =
    <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
    type KeyOwnerIdentification =
    <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::IdentificationTuple;
    type HandleEquivocation = (); //pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, (), ReportLongevity>; // () Offences
    type WeightInfo = ();
}

impl pallet_grandpa::Config for Runtime {
	type Event = Event;
	type Call = Call;

	type KeyOwnerProofSystem = Historical;
	type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
	type KeyOwnerIdentification =
	<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId, )>>::IdentificationTuple;
	type HandleEquivocation = (); //pallet_grandpa::EquivocationHandler<Self::KeyOwnerIdentification, (), ReportLongevity>; // Offences

	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

/// Configure the template pallet in pallets/template.
impl pallet_template::Config for Runtime {
	type Event = Event;
}

/// Configure the grandao pallet in pallets/grandao.
impl pallet_grandao::Config for Runtime {
	type Event = Event;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Zero::zero()
    };
}

parameter_types! {
	pub const BitCountryTreasuryModuleId: ModuleId = ModuleId(*b"gdt/trsy");
    pub TreasuryModuleAccount: AccountId = BitCountryTreasuryModuleId::get().into_account();
}

type Amount = i128;
pub type CurrencyId = u32;

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Runtime, TreasuryModuleAccount>;
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = 0;
}

impl orml_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}

parameter_types! {
    pub CreateClassDeposit: Balance = 500 * CENTS;
    pub CreateTokenDeposit: Balance = 100 * DOLLARS;
	pub const NftModuleId: ModuleId = ModuleId(*b"gdt/anft");
}

impl pallet_gdnft::Config for Runtime{
	type Event = Event;
	type CreateClassDeposit = CreateClassDeposit;
    type CreateTokenDeposit = CreateTokenDeposit;
    type Currency = Currency<Runtime, GetNativeCurrencyId>;
    type WeightInfo = gdnft::WeightInfo<Runtime>;
    type ModuleId = NftModuleId;
}

impl orml_nft::Config for Runtime {
    type ClassId = u32;
    type TokenId = u64;
    type ClassData = pallet_gdnft::ClassData;
    type TokenData = pallet_gdnft::TokenData;
}

parameter_types! {
	pub const TombstoneDeposit: Balance = deposit(
		1,
		sp_std::mem::size_of::<pallet_contracts::ContractInfo<Runtime>>() as u32
	);
	pub const DepositPerContract: Balance = TombstoneDeposit::get();
	pub const DepositPerStorageByte: Balance = deposit(0, 1);
	pub const DepositPerStorageItem: Balance = deposit(1, 0);
	pub RentFraction: Perbill = Perbill::from_rational_approximation(1u32, 30 * DAYS);
	pub const SurchargeReward: Balance = 150 * MILLICENTS;
	pub const SignedClaimHandicap: u32 = 2;
	pub const MaxDepth: u32 = 32;
	pub const MaxValueSize: u32 = 16 * 1024;
	// The lazy deletion runs inside on_initialize.
	pub DeletionWeightLimit: Weight = AVERAGE_ON_INITIALIZE_RATIO *
		RuntimeBlockWeights::get().max_block;
	// The weight needed for decoding the queue should be less or equal than a fifth
	// of the overall weight dedicated to the lazy deletion.
	pub DeletionQueueDepth: u32 = ((DeletionWeightLimit::get() / (
			<Runtime as pallet_contracts::Config>::WeightInfo::on_initialize_per_queue_item(1) -
			<Runtime as pallet_contracts::Config>::WeightInfo::on_initialize_per_queue_item(0)
		)) / 5) as u32;
	pub MaxCodeSize: u32 = 128 * 1024;
}

impl pallet_contracts::Config for Runtime {
	type Time = Timestamp;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type Event = Event;
	type RentPayment = ();
	type SignedClaimHandicap = SignedClaimHandicap;
	type TombstoneDeposit = TombstoneDeposit;
	type DepositPerContract = DepositPerContract;
	type DepositPerStorageByte = DepositPerStorageByte;
	type DepositPerStorageItem = DepositPerStorageItem;
	type RentFraction = RentFraction;
	type SurchargeReward = SurchargeReward;
	type MaxDepth = MaxDepth;
	type MaxValueSize = MaxValueSize;
	type WeightPrice = pallet_transaction_payment::Module<Self>;
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	type ChainExtension = ();
	type DeletionQueueDepth = DeletionQueueDepth;
	type DeletionWeightLimit = DeletionWeightLimit;
	type MaxCodeSize = MaxCodeSize;
}

parameter_types! {
	pub const MultisigDepositBase: Balance = 500 * MILLICENTS;
	pub const MultisigDepositFactor: Balance = 100 * MILLICENTS;
	pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type DepositBase = MultisigDepositBase;
	type DepositFactor = MultisigDepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = ();
}

parameter_types! {
	pub const ConfigDepositBase: Balance = 10 * CENTS;
	pub const FriendDepositFactor: Balance = CENTS;
	pub const MaxFriends: u16 = 9;
	pub const RecoveryDeposit: Balance = 10 * CENTS;
}

impl pallet_recovery::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ConfigDepositBase = ConfigDepositBase;
	type FriendDepositFactor = FriendDepositFactor;
	type MaxFriends = MaxFriends;
	type RecoveryDeposit = RecoveryDeposit;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}

impl pallet_proxy::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ProxyType = ();
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = ();
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
	type Call = Call;
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = ();
}

/// add session
parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;

    type Keys = opaque::SessionKeys;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type WeightInfo = ();
}

impl pallet_session::historical::Config for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

/// add staking
pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_025_000,
        max_inflation: 0_100_000,
        ideal_stake: 0_500_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}

parameter_types! {
    pub const SessionsPerEra: sp_staking::SessionIndex = 3; // 3 hours
    pub const BondingDuration: pallet_staking::EraIndex = 4; // 12 hours
    pub const SlashDeferDuration: pallet_staking::EraIndex = 2; // 6 hours
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;

    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub const MaxIterations: u32 = 5;
    // 0.05%. The higher the value, the more strict solution acceptance becomes.
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
    pub StakingUnsignedPriority: TransactionPriority =
        Perbill::from_percent(90) * TransactionPriority::max_value();
}

parameter_types! {
    pub OffchainSolutionWeightLimit: Weight = RuntimeBlockWeights::get()
        .get(DispatchClass::Normal)
        .max_extrinsic
        .expect("Normal extrinsics have weight limit configured by default; qed")
        .saturating_sub(BlockExecutionWeight::get());
}

pub type CurrencyToVote = frame_support::traits::U128CurrencyToVote;

impl pallet_staking::Config for Runtime {
    type Call = Call;
    type Event = Event;
    type UnixTime = Timestamp;

    type Currency = Balances;
    type CurrencyToVote = CurrencyToVote;
    type RewardRemainder = (); // TODO: Treasury
    type Slash = (); // TODO: Treasury
    type Reward = (); // rewards are minted from the voi

    type SessionInterface = Self;
    type NextNewSession = Session;
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    /// A super-majority of the council can cancel the slash.
    type SlashCancelOrigin = EnsureRoot<Self::AccountId>; // TODO: SlashCancelOrigin
    type RewardCurve = RewardCurve;
    type ElectionLookahead = ElectionLookahead;

    type MinSolutionScoreBump = MinSolutionScoreBump;
    type MaxIterations = MaxIterations;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type UnsignedPriority = StakingUnsignedPriority; // Kusama
    type OffchainSolutionWeightLimit = OffchainSolutionWeightLimit;
    type WeightInfo = ();
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
	where
		Call: From<C>,
{
	type OverarchingCall = Call;
	type Extrinsic = UncheckedExtrinsic;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Module, Storage},
		Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},

		Babe: pallet_babe::{Module, Call, Storage, Config, ValidateUnsigned},
		Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event, ValidateUnsigned},
		Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
		Staking: pallet_staking::{Module, Call, Config<T>, Storage, Event<T>},
		Historical: session_historical::{Module},
		// Include the custom logic from the template pallet in the runtime.

		TemplateModule: pallet_template::{Module, Call, Storage, Event<T>},
		GrandaoModule: pallet_grandao::{Module, Call, Storage, Event<T>},
		Contracts: pallet_contracts::{Module, Call, Config<T>, Storage, Event<T>},
		NftModule: pallet_gdnft::{Module, Call ,Storage, Event<T>},
		OrmlNFT: orml_nft::{Module ,Storage},
		Currencies: orml_currencies::{Module, Storage, Call, Event<T>},
		Tokens: orml_tokens::{Module, Storage, Call, Event<T>},

		Multisig: pallet_multisig::{Module, Call, Storage, Event<T>},
		Recovery: pallet_recovery::{Module, Call, Storage, Event<T>},
		Proxy: pallet_proxy::{Module, Call, Storage, Event<T>},
		Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>},

	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllModules,
>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) ->
			Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            sp_consensus_babe::BabeGenesisConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: PRIMARY_PROBABILITY,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
            ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
                )
        }
    }

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
		for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Module as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
					.to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
					.to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
					.to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
					.to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
					.to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}
