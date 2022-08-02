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
use frame_support::{assert_err, assert_noop, assert_ok, assert_storage_noop, bounded_btree_map};

#[test]
fn basic_minting_should_work() {
	new_test_ext().execute_with(|| {
		let owner = LiquidStaking::account_id();
		let account_1 = Origin::signed(1);
		let amount_1 = 4000u64;
		println!("{:?}", Balances::free_balance(owner));

		LiquidStaking::stake(account_1, amount_1).unwrap();
		LiquidStaking::stake(Origin::signed(2), 2000u64).unwrap();
		LiquidStaking::stake(Origin::signed(3), 2000u64).unwrap();

		let account_status = AccountStatus::<Test>::get(1u128, Status::Staking).unwrap();
		assert_eq!(account_status, amount_1);

		println!("active_stake prev nominate {:?}", StakingMock::active_stake(&owner).unwrap());
		//	LiquidStaking::nominate(Origin::signed(1), vec![1u128]).unwrap();
		println!("active_stake after nominate {:?}", StakingMock::active_stake(&owner).unwrap());

		LiquidStaking::unbound(Origin::signed(1), 100u64).unwrap();
		println!("active_stake after unstake {:?}", StakingMock::active_stake(&owner).unwrap());
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
		//assert_eq!(Balances::free_balance(owner),owner_balance + amount);
		//assert_eq!(Assets::balance(asset_id, &1u128), amount - amount_to_unbound);

		println!("active_stake after unstake {:?}", StakingMock::active_stake(&owner).unwrap());
	});
}
