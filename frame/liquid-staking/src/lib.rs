#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use pallet_democracy::{AccountVote, ReferendumIndex};
use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, Zero};
use sp_staking::StakingInterface;
use sp_std::vec::Vec;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			fungibles::{
				metadata::Mutate as MutateMetadata, multicurrency::BalanceOrAsset, Create, Inspect,
				Mutate, Transfer,
			},
			Currency,
			ExistenceRequirement::KeepAlive,
			LockableCurrency, ReservableCurrency,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use pallet_democracy::{Conviction, Vote};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_democracy::Config<Currency = Self::ReservedCurrency>
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type ReservedCurrency: ReservableCurrency<Self::AccountId, Balance = Self::CurrencyBalance>
			+ LockableCurrency<Self::AccountId, Balance = Self::CurrencyBalance>;

		type CurrencyBalance: AtLeast32BitUnsigned
			+ codec::FullCodec
			+ Copy
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ Default
			+ From<u64>
			+ TypeInfo
			+ MaxEncodedLen;

		type AssetId: Member
			+ Parameter
			+ Default
			+ Copy
			+ codec::HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ From<u32>
			+ TypeInfo;

		type Assets: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::CurrencyBalance>
			+ Create<Self::AccountId>
			+ Transfer<Self::AccountId>
			+ Mutate<Self::AccountId>
			+ MutateMetadata<Self::AccountId>;

		/// The interface for nominating.
		type StakingInterface: StakingInterface<
			Balance = BalanceOf<Self>,
			AccountId = Self::AccountId,
		>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	pub type AssetIdOf<T> =
		<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
	pub type AssetBalanceOf<T> =
		<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
	pub type BalanceOrAssetOf<T> = BalanceOrAsset<BalanceOf<T>, AssetIdOf<T>, AssetBalanceOf<T>>;

	pub(super) type BalanceOf<T> = <<T as Config>::ReservedCurrency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub enum VoteType<BalanceOrAssetOf> {
		Standard { vote: Vote, balance: BalanceOrAssetOf },
		Split { balance: BalanceOrAssetOf },
	}
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https&://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn get_staked)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Staked<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::CurrencyBalance>;

	// The pallet's runtime storage items.
	// https&://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn get_voted)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Voted<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::CurrencyBalance>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Deposited,
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]

	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn init(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let account = Self::account_id();
			let amount = BalanceOf::<T>::from(1_000_000_000_000_000u64);
			T::ReservedCurrency::transfer(&who, &account, amount, KeepAlive).unwrap();
			let asset_id = T::AssetId::from(1u32.into());
			T::Assets::create(asset_id, account.clone(), true, T::CurrencyBalance::from(1u64))
				.unwrap();
			T::Assets::set(asset_id, &account, "LDOT".into(), "LDOT".into(), 10).unwrap();
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn init_old(_origin: OriginFor<T>) -> DispatchResult {
			let owner = Self::account_id();
			let asset_id = T::AssetId::from(1u32.into());
			T::Assets::create(asset_id, owner.clone(), true, T::CurrencyBalance::from(1u64))
				.unwrap();
			T::Assets::set(asset_id, &owner, "LDOT".into(), "LDOT".into(), 10).unwrap();

			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn stake(origin: OriginFor<T>, value: T::CurrencyBalance) -> DispatchResult {
			let source = ensure_signed(origin)?;

			let owner = Self::account_id();

			T::ReservedCurrency::transfer(&source, &owner, value, KeepAlive).unwrap();

			<Staked<T>>::insert(&source, &value);

			T::Assets::mint_into(T::AssetId::from(1u32.into()), &source, value).unwrap();

			T::StakingInterface::bond(owner.clone(), owner.clone(), value.clone(), owner.clone())
				.unwrap_or(T::StakingInterface::bond_extra(owner.clone(), value.clone()).unwrap());

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn nominate(_origin: OriginFor<T>, validator: Vec<T::AccountId>) -> DispatchResult {
			let owner = Self::account_id();
			T::StakingInterface::nominate(owner.clone(), validator).unwrap();

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let asset_id = T::AssetId::from(1u32.into());
			T::Assets::balance(asset_id, &who);

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn vote_with_liquid(
			origin: OriginFor<T>,
			#[pallet::compact] ref_index: ReferendumIndex,
			vote: AccountVote<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let asset_id = T::AssetId::from(1u32.into());
			let owner = Self::account_id();
			let amount = vote.balance();
			T::Assets::transfer(asset_id, &who, &owner, amount, true).unwrap();
			<Voted<T>>::insert(&who, &amount);

			pallet_democracy::Pallet::<T>::try_vote(&owner, ref_index, vote).unwrap();

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn fund_pallet_account(account_with_funds: T::AccountId, amount: T::CurrencyBalance) {
			let account = Self::account_id();
			let amount = BalanceOf::<T>::from(1_000_000_000u64);
			T::ReservedCurrency::transfer(&account_with_funds, &account, amount, KeepAlive)
				.unwrap();
		}
	}
}
