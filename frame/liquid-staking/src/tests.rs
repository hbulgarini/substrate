// This file is part of Substrate.

// Copyright (C) 2020-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use crate::mock::*;

#[test]
fn basic_minting_should_work() {
	new_test_ext().execute_with(|| {
		let owner = LiquidStaking::account_id();
		let account_1 = Origin::signed(1);
		let amount_1 = 4000u64;

		let account_2 = Origin::signed(2);
		let amount_2 = 6000u64;

		LiquidStaking::stake(account_1, amount_1).unwrap();
		LiquidStaking::stake(account_2, amount_2).unwrap();


		let account_staked = Staking::<Test>::get(1u128).unwrap();
		assert_eq!(account_staked, amount_1);

		let total_staked = TotalStaked::<Test>::get().unwrap();
		assert_eq!(total_staked, amount_1 + amount_2);
	});
}

#[test]
fn unbound_should_work() {
	new_test_ext().execute_with(|| {
		let owner = LiquidStaking::account_id();
		let who = Origin::signed(1);
		let asset_id = AssetIdOf::<Test>::from(LiquidAssetId::get());
		let amount = 4000u64;
		let amount_to_unbound = 1500u64;
		let owner_balance = Balances::free_balance(owner);

		LiquidStaking::stake(who, amount).unwrap();

		assert_eq!(Balances::free_balance(owner), owner_balance + amount);
		assert_eq!(Assets::balance(asset_id, &1u128), amount);
		assert_eq!(StakingMock::active_stake(&owner).unwrap(), amount);

		LiquidStaking::unbound(Origin::signed(1), amount_to_unbound).unwrap();

		assert_eq!(StakingMock::active_stake(&owner).unwrap(), amount - amount_to_unbound);
	});
}
