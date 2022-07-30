use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

/// Represents either a System currency or a set of fungible assets.
#[derive(Encode, Decode, Clone, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum BalanceOrAsset<Balance, AssetId, AssetBalance> {
	Balance { amount: Balance },
	Asset { id: AssetId, amount: AssetBalance },
}

impl<B, A, AB> From<B> for BalanceOrAsset<B, A, AB> {
	fn from(amount: B) -> Self {
		Self::Balance { amount }
	}
}

impl<B, A, AB> BalanceOrAsset<B, A, AB>
where
	A: core::cmp::PartialEq,
	B: core::cmp::PartialOrd,
	AB: core::cmp::PartialOrd,
{
	pub fn is_greater_or_equal(&self, other: &Self) -> bool {
		use BalanceOrAsset::*;
		match (self, other) {
			(Balance { amount: a }, Balance { amount: b }) => a >= b,
			(Asset { amount: a, .. }, Asset { amount: b, .. }) => a >= b,
			_ => false,
		}
	}
	pub fn is_same_currency(&self, other: &Self) -> bool {
		use BalanceOrAsset::*;
		match (self, other) {
			(Balance { .. }, Balance { .. }) => true,
			(Asset { id, .. }, Asset { id: id2, .. }) => id == id2,
			_ => false,
		}
	}
}

impl<B, A, AB> BalanceOrAsset<B, A, AB> {
	pub fn into_amount<T>(self) -> T
	where
		T: From<B>,
		T: From<AB>,
	{
		match self {
			Self::Balance { amount } => amount.into(),
			Self::Asset { amount, id: _ } => amount.into(),
		}
	}
}
