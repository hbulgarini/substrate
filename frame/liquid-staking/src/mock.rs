use crate as pallet_liquid_staking;
use frame_support::{
	ord_parameter_types,
	pallet_prelude::{DispatchError, DispatchResult},
	parameter_types,
	traits::{ConstU32, ConstU64, EqualPrivilegeOnly, SortedMembers},
	weights::Weight,
	PalletId,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use sp_keystore::{testing::KeyStore, KeystoreExt};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use std::sync::Arc;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type AccountId = u128;
pub type Balance = u64;
pub type EraIndex = u32;

parameter_types! {
	pub static CurrentEra: EraIndex = 0;
	pub static BondingDuration: EraIndex = 3;
	pub storage BondedBalanceMap: BTreeMap<AccountId, Balance> = Default::default();
	pub storage UnbondingBalanceMap: BTreeMap<AccountId, Balance> = Default::default();
	#[derive(Clone, PartialEq)]
	pub static MaxUnbonding: u32 = 8;
	pub storage Nominations: Option<Vec<AccountId>> = None;
}

pub struct StakingMock;
impl StakingMock {
	pub(crate) fn set_bonded_balance(who: AccountId, bonded: Balance) {
		let mut x = BondedBalanceMap::get();
		x.insert(who, bonded);
		BondedBalanceMap::set(&x)
	}
}

impl sp_staking::StakingInterface for StakingMock {
	type Balance = Balance;
	type AccountId = AccountId;

	fn minimum_bond() -> Self::Balance {
		10
	}

	fn current_era() -> EraIndex {
		CurrentEra::get()
	}

	fn bonding_duration() -> EraIndex {
		BondingDuration::get()
	}

	fn active_stake(who: &Self::AccountId) -> Option<Self::Balance> {
		BondedBalanceMap::get().get(who).map(|v| *v)
	}

	fn total_stake(who: &Self::AccountId) -> Option<Self::Balance> {
		match (
			UnbondingBalanceMap::get().get(who).map(|v| *v),
			BondedBalanceMap::get().get(who).map(|v| *v),
		) {
			(None, None) => None,
			(Some(v), None) | (None, Some(v)) => Some(v),
			(Some(a), Some(b)) => Some(a + b),
		}
	}

	fn bond_extra(who: Self::AccountId, extra: Self::Balance) -> DispatchResult {
		let mut x = BondedBalanceMap::get();
		x.get_mut(&who).map(|v| *v += extra);
		BondedBalanceMap::set(&x);
		Ok(())
	}

	fn unbond(who: Self::AccountId, amount: Self::Balance) -> DispatchResult {
		let mut x = BondedBalanceMap::get();
		*x.get_mut(&who).unwrap() = x.get_mut(&who).unwrap().saturating_sub(amount);
		BondedBalanceMap::set(&x);
		let mut y = UnbondingBalanceMap::get();
		*y.entry(who).or_insert(0u64) += amount;
		UnbondingBalanceMap::set(&y);
		Ok(())
	}

	fn chill(_: Self::AccountId) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn withdraw_unbonded(who: Self::AccountId, _: u32) -> Result<bool, DispatchError> {
		// Simulates removing unlocking chunks and only having the bonded balance locked
		let mut x = UnbondingBalanceMap::get();
		x.remove(&who);
		UnbondingBalanceMap::set(&x);

		Ok(UnbondingBalanceMap::get().is_empty() && BondedBalanceMap::get().is_empty())
	}

	fn bond(
		stash: Self::AccountId,
		_: Self::AccountId,
		value: Self::Balance,
		_: Self::AccountId,
	) -> DispatchResult {
		StakingMock::set_bonded_balance(stash, value);
		Ok(())
	}

	fn nominate(_: Self::AccountId, nominations: Vec<Self::AccountId>) -> DispatchResult {
		Nominations::set(&Some(nominations));
		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn nominations(_: Self::AccountId) -> Option<Vec<Self::AccountId>> {
		Nominations::get()
	}
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		LiquidStaking: pallet_liquid_staking::{Pallet, Call, Storage, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub static PreimageByteDeposit: u64 = 0;
	pub static InstantAllowed: bool = false;
}

ord_parameter_types! {
	pub const One: u128 = 1;
	pub const Two: u128 = 2;
	pub const Three: u128 = 3;
	pub const Four: u128 = 4;
	pub const Five: u128 = 5;
	pub const Six: u128 = 6;
}
pub struct OneToFive;
impl SortedMembers<u128> for OneToFive {
	fn sorted_members() -> Vec<u128> {
		vec![1, 2, 3, 4, 5]
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn add(_m: &u128) {}
}

impl pallet_democracy::Config for Test {
	type Proposal = Call;
	type Event = Event;
	type Currency = pallet_balances::Pallet<Self>;
	type EnactmentPeriod = ConstU64<2>;
	type LaunchPeriod = ConstU64<2>;
	type VotingPeriod = ConstU64<2>;
	type VoteLockingPeriod = ConstU64<3>;
	type FastTrackVotingPeriod = ConstU64<2>;
	type MinimumDeposit = ConstU64<1>;
	type ExternalOrigin = EnsureSignedBy<Two, u128>;
	type ExternalMajorityOrigin = EnsureSignedBy<Three, u128>;
	type ExternalDefaultOrigin = EnsureSignedBy<One, u128>;
	type FastTrackOrigin = EnsureSignedBy<Five, u128>;
	type CancellationOrigin = EnsureSignedBy<Four, u128>;
	type BlacklistOrigin = EnsureRoot<u128>;
	type CancelProposalOrigin = EnsureRoot<u128>;
	type VetoOrigin = EnsureSignedBy<OneToFive, u128>;
	type CooloffPeriod = ConstU64<2>;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Slash = ();
	type InstantOrigin = EnsureSignedBy<Six, u128>;
	type InstantAllowed = InstantAllowed;
	type Scheduler = Scheduler;
	type MaxVotes = ConstU32<100>;
	type OperationalPreimageOrigin = EnsureSignedBy<Six, u128>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
	type MaxProposals = ConstU32<100>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = 1000;
}
impl pallet_scheduler::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<u128>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PreimageProvider = ();
	type NoPreimagePostponement = ();
}

impl frame_system::Config for Test {
	type SS58Prefix = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = Call;
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::testing::Header;
	type Event = Event;
	type BlockHashCount = ();
	type DbWeight = ();
	type BlockLength = ();
	type BlockWeights = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = u64;
	type AssetId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<u128>;
	type AssetDeposit = ConstU64<1>;
	type AssetAccountDeposit = ConstU64<10>;
	type MetadataDepositBase = ConstU64<1>;
	type MetadataDepositPerByte = ConstU64<1>;
	type ApprovalDeposit = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type WeightInfo = ();
	type Extra = ();
}

parameter_types! {
	pub const LiquidStakingPalletId: PalletId = PalletId(*b"lstaking");

}

impl pallet_liquid_staking::Config for Test {
	type Event = Event;
	type StakingInterface = StakingMock;
	type AssetId = u32;
	type Assets = Assets;
	type ReservedCurrency = Balances;
	type CurrencyBalance = <Self as pallet_balances::Config>::Balance;
	type PalletId = LiquidStakingPalletId;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 100000000000000),
			(2, 10000000000),
			(3, 10000000000),
			(4, 10000000000),
			(LiquidStaking::account_id(), 1000000000000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let keystore = KeyStore::new();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt(Arc::new(keystore)));
	ext.execute_with(|| System::set_block_number(1));
	ext
}
