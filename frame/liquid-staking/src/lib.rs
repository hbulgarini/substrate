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
use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, Saturating, Zero};
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
			+ Saturating
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

		#[pallet::constant]
		type LiquidAssetId: Get<u32>;
	}

	pub enum TransferDirection {
		From,
		To,
	}

	#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
	pub enum Status {
		Staking,
		Voting,
		Unbonding,
	}

	pub type AssetIdOf<T> =
		<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
	pub type AssetBalanceOf<T> =
		<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
	pub type BalanceOrAssetOf<T> = BalanceOrAsset<BalanceOf<T>, AssetIdOf<T>, AssetBalanceOf<T>>;

	pub(super) type BalanceOf<T> = <<T as Config>::ReservedCurrency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn status)]
	pub(super) type AccountStatus<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		Status,
		T::CurrencyBalance,
	>;

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

	// The pallet's runtime storage items.
	// https&://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Bonded<T: Config> = StorageValue<_, bool>;

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
		/// Unbound
		ErrorUnbonding,
		ErrorBonding,
		ErrorBondingExtra,
		WithdrawError,
		NotEnoughAssets,
		NotEnoughAssetsAvailable,
		AssetNotBurnt,
		BalanceWithdrawError,
		NominationFailed,
		AssetMintingFailed,
		DepositFailed,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]

	impl<T: Config> Pallet<T> {
		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn stake(origin: OriginFor<T>, value: T::CurrencyBalance) -> DispatchResult {
			let source = ensure_signed(origin)?;
			let liquid_asset_id = T::AssetId::from(T::LiquidAssetId::get());
			let owner = Self::account_id();

			T::ReservedCurrency::transfer(&source, &owner, value, KeepAlive)
				.map_err(|_| Error::<T>::DepositFailed)?;

			<Staked<T>>::insert(&source, &value);
			<AccountStatus<T>>::insert(&source, Status::Staking, &value);

			T::Assets::mint_into(liquid_asset_id, &source, value)
				.map_err(|_| Error::<T>::AssetMintingFailed)?;
			let staked = T::StakingInterface::total_stake(&owner);

			match staked {
				None => T::StakingInterface::bond(
					owner.clone(),
					owner.clone(),
					value.clone(),
					owner.clone(),
				)
				.map_err(|_| Error::<T>::ErrorBonding)?,
				Some(_) => T::StakingInterface::bond_extra(owner.clone(), value.clone())
					.map_err(|_| Error::<T>::ErrorBondingExtra)?,
			};

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn nominate(_origin: OriginFor<T>, validator: Vec<T::AccountId>) -> DispatchResult {
			let owner = Self::account_id();
			T::StakingInterface::nominate(owner.clone(), validator)
				.map_err(|_| Error::<T>::NominationFailed)?;

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn unbound(origin: OriginFor<T>, amount: T::CurrencyBalance) -> DispatchResult {
			let liquid_asset_id = T::AssetId::from(T::LiquidAssetId::get());
			let who = ensure_signed(origin)?;
			let owner = Self::account_id();

			let total_asset_balance = T::Assets::balance(liquid_asset_id, &who);

			ensure!(amount <= total_asset_balance, Error::<T>::NotEnoughAssets);

			let has_voting = AccountStatus::<T>::get(&who, Status::Voting);
			let available = match has_voting {
				Some(voting) => total_asset_balance - voting,
				None => amount,
			};

			ensure!(amount <= available, Error::<T>::NotEnoughAssetsAvailable);
			let remaining = total_asset_balance - available;
			if remaining > Zero::zero() {
				<AccountStatus<T>>::insert(&who, Status::Staking, &remaining);
			}

			T::StakingInterface::unbond(owner, amount).map_err(|_| Error::<T>::ErrorUnbonding)?;
			<AccountStatus<T>>::insert(&who, Status::Unbonding, &amount);

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn withdraw_unbonded(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let liquid_asset_id = T::AssetId::from(T::LiquidAssetId::get());
			let owner = Self::account_id();

			let withdrawn = AccountStatus::<T>::get(&who, Status::Unbonding)
				.ok_or(Error::<T>::WithdrawError)?;

			T::StakingInterface::withdraw_unbonded(owner.clone(), 10)
				.map_err(|_| Error::<T>::ErrorUnbonding)?;
			T::Assets::burn_from(liquid_asset_id, &owner.clone(), withdrawn)
				.map_err(|_| Error::<T>::AssetNotBurnt)?;
			T::ReservedCurrency::transfer(&owner, &who, withdrawn, KeepAlive)
				.map_err(|_| Error::<T>::BalanceWithdrawError)?;
			Self::transfer_owner(&who, TransferDirection::From, &withdrawn);

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn vote_with_liquid(
			origin: OriginFor<T>,
			#[pallet::compact] ref_index: ReferendumIndex,
			vote: AccountVote<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let liquid_asset_id = T::AssetId::from(T::LiquidAssetId::get());
			let owner = Self::account_id();
			let amount = vote.balance();
			T::Assets::transfer(liquid_asset_id, &who, &owner, amount, true).unwrap();
			<Voted<T>>::insert(&who, &amount);

			// TODO remove pub and use vote
			pallet_democracy::Pallet::<T>::try_vote(&owner, ref_index, vote).unwrap();

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn transfer_owner(
			account: &T::AccountId,
			direction: TransferDirection,
			amount: &T::CurrencyBalance,
		) {
			let liquid_asset_id = T::AssetId::from(T::LiquidAssetId::get());
			let owner = Self::account_id();
			match direction {
				TransferDirection::From =>
					T::Assets::transfer(liquid_asset_id, &account, &owner, *amount, true),
				TransferDirection::To =>
					T::Assets::transfer(liquid_asset_id, &owner, &account, *amount, true),
			};
		}
	}
}
