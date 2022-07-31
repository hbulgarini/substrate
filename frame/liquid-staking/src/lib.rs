#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod types;

pub use types::*;

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
			fungibles::{metadata::Mutate as MutateMetadata, Create, Inspect, Mutate, Transfer},
			Currency,
			ExistenceRequirement::KeepAlive,
			ReservableCurrency,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	pub(super) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: ReservableCurrency<Self::AccountId, Balance = Self::CurrencyBalance>;
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

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https&://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn get_channel)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Reserved<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>>;

	/* 	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub liquid_asset: (T::AssetId, T::AccountId, bool, T::CurrencyBalance),
		pub metadata: (T::AssetId, Vec<u8>, Vec<u8>, u8),
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			let owner = T::PalletId::get().into_account_truncating();
			Self {
				liquid_asset: (
					T::AssetId::from(1u32.into()),
					owner,
					true,
					T::CurrencyBalance::from(1u64),
				),
				metadata: (
					T::AssetId::from(1u32.into()),
					"LTOK".into(),
					"Liquid Token".into(),
					10.into(),
				),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// let owner = T::account_id();
			let (id, owner, is_sufficient, min_balance) = &self.liquid_asset;
			//	assert!(!pallet_assets::Asset::T::contains_key(id), "Asset id already in use");
			assert!(!min_balance.is_zero(), "Min balance should not be zero");

			T::Assets::create(*id, owner.clone(), *is_sufficient, *min_balance);

			let (_, name, symbol, decimals) = &self.metadata;
			assert!(pallet_assets::pallet::Asset::contains_key(id), "Asset does not exist");
			//pallet_assets::pallet::Asset::contains_key(key: KeyArg)

			let bounded_name: BoundedVec<u8, T::StringLimit> =
				name.clone().try_into().expect("asset name is too long");
			let bounded_symbol: BoundedVec<u8, T::StringLimit> =
				symbol.clone().try_into().expect("asset symbol is too long");

			let metadata = AssetMetadata {
				deposit: Zero::zero(),
				name: bounded_name,
				symbol: bounded_symbol,
				decimals: *decimals,
				is_frozen: false,
			};

			Metadata::<T>::insert(id, metadata);
		}

		/* 			for (id, account_id, amount) in &self.accounts {
			let result = <Pallet<T>>::increase_balance(
				*id,
				account_id,
				*amount,
				|details| -> DispatchResult {
					debug_assert!(
						T::Balance::max_value() - details.supply >= *amount,
						"checked in prep; qed"
					);
					details.supply = details.supply.saturating_add(*amount);
					Ok(())
				},
			);
			assert!(result.is_ok());
		} */
	} */

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
		pub fn fund(
			_origin: OriginFor<T>,
			founder: T::AccountId,
			funding_amount: T::CurrencyBalance,
		) -> DispatchResult {
			Self::fund_pallet_account(founder, funding_amount);
			let owner = Self::account_id();

			log::info!(
				target: "TEST",
				"Owner {:#?}",
				owner,
			);
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn init(_origin: OriginFor<T>) -> DispatchResult {
			let owner = Self::account_id();
			let asset_id = T::AssetId::from(1u32.into());
			T::Assets::create(asset_id, owner.clone(), true, T::CurrencyBalance::from(1u64))
				.unwrap();
			T::Assets::set(asset_id, &owner, "LDOT".into(), "LDOT".into(), 10).unwrap();

			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn stake(
			origin: OriginFor<T>,
			value: BalanceOrAssetOf<T>,
			validator: Vec<T::AccountId>,
		) -> DispatchResult {
			let source = ensure_signed(origin)?;

			//T::Currency::reserve(&source, value.clone().into_amount())?;

			let valuec = value.clone();
			T::Assets::mint_into(T::AssetId::from(1u32.into()), &source, value.into_amount())
				.unwrap();

			let staked_account = source.clone();
			T::StakingInterface::bond(
				staked_account.clone(),
				staked_account.clone(),
				valuec.into_amount(),
				staked_account.clone(),
			)
			.unwrap();

			T::StakingInterface::nominate(source, validator).unwrap();

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn unstake(origin: OriginFor<T>, value: BalanceOrAssetOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let reserved = Reserved::<T>::get(&who).unwrap();
			let left = reserved - value.into_amount();
			T::Currency::unreserve(&who, left);
			Reserved::<T>::insert(&who, left);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn fund_pallet_account(account_with_funds: T::AccountId, amount: T::CurrencyBalance) {
			let account = Self::account_id();
			T::Currency::transfer(&account_with_funds, &account, amount, KeepAlive).unwrap();
		}
	}
}
