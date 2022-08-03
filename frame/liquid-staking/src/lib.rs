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
use sp_staking::{StakingInterface, EraIndex };
use sp_std::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use sp_arithmetic::per_things::Perbill;

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
			+ Zero
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


	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct UnbondingInformation<CurrencyBalance, EraIndex> {
		balance: CurrencyBalance,
		unbonding_era: EraIndex,
	}	

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct UserStakeData<CurrencyBalance> {
		balance: CurrencyBalance,
		percentage: Perbill,
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
	#[pallet::getter(fn staking)]
	pub(super) type Staking<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		T::CurrencyBalance,
	>;

	#[pallet::storage]
	#[pallet::getter(fn total_stake)]
	pub(super) type TotalStaked<T: Config> = StorageValue<_, T::CurrencyBalance>;

	#[pallet::storage]
	#[pallet::getter(fn voting)]
	pub(super) type Voting<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		T::CurrencyBalance,
	>;

	#[pallet::storage]
	#[pallet::getter(fn unbonding)]
	pub(super) type Unbonding<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		UnbondingInformation<T::CurrencyBalance,EraIndex>
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// User deposited balance
		BalanceStaked {account: T::AccountId, amount: T::CurrencyBalance},
		// Validators nominated
		ValidatorsNominated {validators: Vec<T::AccountId>},
		// Funds unbonded
		StakeUnbonded  {account: T::AccountId, amount: T::CurrencyBalance, unbonding_era: EraIndex },
		// User voted
		UserVoted { account: T::AccountId, ref_index: ReferendumIndex, amount: T::CurrencyBalance }
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Error during Unbound
		ErrorUnbonding,
		/// Error during bonding funds
		ErrorBonding,
		/// Error bonfing extra  funds
		ErrorBondingExtra,
		/// Error during withdrawing of bonds 
		WithdrawError,
		/// User trying to claim more assets than staked
		NotEnoughAssets,
		/// Not enough available assets to unbond
		NotEnoughAssetsAvailable,
		/// Error burning asset
		AssetNotBurnt,
		/// Error withdrawing native balance
		BalanceWithdrawError,
		/// Nomination failed
		NominationFailed,
		/// Error minting liquid asset
		AssetMintingFailed,
		/// Error depositing native asset
		DepositFailed,
		/// Error transfering liquid asset
		AssetTransferError,
		/// Govenance voting failed
		GovernanceVotingError,
		/// Funds where not unbonded
		NotUnbondedYet
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}


	#[pallet::call]

	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn stake(origin: OriginFor<T>, amount: T::CurrencyBalance) -> DispatchResult {
			let source = ensure_signed(origin)?;
			let liquid_asset_id = T::AssetId::from(T::LiquidAssetId::get());
			let owner = Self::account_id();

			T::ReservedCurrency::transfer(&source, &owner, amount, KeepAlive)
				.map_err(|_| Error::<T>::DepositFailed)?;

			let total_staked = <TotalStaked<T>>::get();
			let updated_stake = match total_staked {
				Some(total_staked) => total_staked.saturating_add(amount),
				None => amount
			};

			<TotalStaked<T>>::put(updated_stake);

			<Staking<T>>::insert(&source, amount.clone());

			T::Assets::mint_into(liquid_asset_id, &source, amount)
				.map_err(|_| Error::<T>::AssetMintingFailed)?;
			let staked = T::StakingInterface::total_stake(&owner);

			match staked {
				None => T::StakingInterface::bond(
					owner.clone(),
					owner.clone(),
					amount.clone(),
					owner.clone(),
				)
				.map_err(|_| Error::<T>::ErrorBonding)?,
				Some(_) => T::StakingInterface::bond_extra(owner.clone(), amount.clone())
					.map_err(|_| Error::<T>::ErrorBondingExtra)?,
			};
			Self::deposit_event(Event::BalanceStaked {account: source, amount});
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn nominate(_origin: OriginFor<T>, validators: Vec<T::AccountId>) -> DispatchResult {
			let owner = Self::account_id();
			T::StakingInterface::nominate(owner.clone(), validators.clone())
				.map_err(|_| Error::<T>::NominationFailed)?;
			
				Self::deposit_event(Event::ValidatorsNominated {validators});

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn unbound(origin: OriginFor<T>, amount: T::CurrencyBalance) -> DispatchResult {
			let liquid_asset_id = T::AssetId::from(T::LiquidAssetId::get());
			let who = ensure_signed(origin)?;
			let owner = Self::account_id();

			let total_asset_balance = T::Assets::balance(liquid_asset_id, &who);

			ensure!(amount <= total_asset_balance, Error::<T>::NotEnoughAssets);

			let has_voting = <Voting<T>>::get(&who);
			let available = match has_voting {
				Some(voting) => total_asset_balance - voting,
				None => amount,
			};

			ensure!(amount <= available, Error::<T>::NotEnoughAssetsAvailable);
			let remaining = total_asset_balance - available;
			if remaining > Zero::zero() {

				<Staking<T>>::insert(&who, &remaining);
			}

			let current_era = T::StakingInterface::current_era();
			let bonding_duration = T::StakingInterface::current_era();
	 
			let unbonding_information = UnbondingInformation {
				balance: amount,
				unbonding_era: current_era + bonding_duration
			} ;

			T::StakingInterface::unbond(owner, amount).map_err(|_| Error::<T>::ErrorUnbonding)?;
			<Unbonding<T>>::insert(&who, &unbonding_information);
			Self::deposit_event(Event::StakeUnbonded {account: who, amount, unbonding_era: unbonding_information.unbonding_era});
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn withdraw_unbonded(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let liquid_asset_id = T::AssetId::from(T::LiquidAssetId::get());
			let owner = Self::account_id();

			let withdrawn = Unbonding::<T>::get(&who)
			.ok_or(Error::<T>::WithdrawError)?;

			let current_era = T::StakingInterface::current_era();
				if withdrawn.unbonding_era >= current_era {
					T::StakingInterface::withdraw_unbonded(owner.clone(), 10)
					.map_err(|_| Error::<T>::ErrorUnbonding)?;
				T::Assets::burn_from(liquid_asset_id, &owner.clone(), withdrawn.balance)
					.map_err(|_| Error::<T>::AssetNotBurnt)?;
				T::ReservedCurrency::transfer(&owner, &who, withdrawn.balance, KeepAlive)
					.map_err(|_| Error::<T>::BalanceWithdrawError)?;

				// Clean up
				let staked_amount = <Staking<T>>::get(&who);
				match staked_amount {
					Some(staked_amount) => {
						if staked_amount.is_zero(){
							<Staking<T>>::take(&who);
						};
					},
					None => ()
				};
			} else {
				return Err(<Error<T>>::NotUnbondedYet.into());
			}



			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn vote_with_liquid(
			origin: OriginFor<T>,
			#[pallet::compact] ref_index: ReferendumIndex,
			vote: AccountVote<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let liquid_asset_id = T::AssetId::from(T::LiquidAssetId::get());
			let owner = Self::account_id();
			let amount = vote.balance();

			T::Assets::transfer(liquid_asset_id, &who, &owner, amount, true).map_err(|_| Error::<T>::AssetTransferError)?;

			<Voting<T>>::insert(&who, &amount);
			pallet_democracy::Pallet::<T>::vote(origin, ref_index, vote).map_err(|_| Error::<T>::GovernanceVotingError)?;

			Self::deposit_event(Event::UserVoted {account: who, ref_index, amount});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
		
	}
}
