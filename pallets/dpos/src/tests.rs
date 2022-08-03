use crate::mock::*;
use frame_support::{assert_noop, assert_ok, traits::tokens::currency::Currency};
use sp_consensus_aura::sr25519;
use sp_core::{keccak_256, Pair};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};

const ALICE_NOMINATOR: AccountId32 = AccountId32::new([0_u8; 32]);
const BOB_NOMINATOR: AccountId32 = AccountId32::new([1_u8; 32]);

fn generate_validators(count: u32) -> Vec<(AccountId32, sr25519::AuthorityId)> {
	let seed: u128 = 12345678901234567890123456789012;
	(0..count)
		.map(|i| {
			let account_id =
				[[0_u8; 16], (&(i as u128 + 1)).to_le_bytes()].concat().try_into().unwrap();
			(
				AccountId32::new(account_id),
				sr25519::AuthorityPair::from_seed(&keccak_256(
					&[(&(seed + i as u128)).to_le_bytes(), (&(seed + i as u128)).to_le_bytes()]
						.concat(),
				))
				.public(),
			)
		})
		.collect()
}

fn with_validators<R>(
	count: u32,
	stake: Balance,
	execute: impl FnOnce(Vec<(AccountId32, sr25519::AuthorityId)>) -> R,
) -> R {
	let validators = generate_validators(count);

	new_test_ext().execute_with(|| {
		validators.iter().for_each(|(account_id, session_key)| {
			Balances::make_free_balance_be(account_id, stake);
			assert_ok!(DPoS::register_validator(
				Origin::signed(account_id.clone()),
				session_key.clone(),
				stake
			));
		});
		execute(validators)
	})
}

#[cfg(test)]
mod register_validator {
	use super::*;

	fn should_fail_on_insufficient_balance() {
		let (account_id, session_key) = &generate_validators(1)[0];
		Balances::make_free_balance_be(&account_id, 1_000);

		assert_noop!(
			DPoS::register_validator(
				Origin::signed(account_id.clone()),
				session_key.clone(),
				1_000
			),
			crate::Error::<Test>::InsufficentBalance
		);
	}
}

#[cfg(test)]
mod nominate {
	use super::*;

	#[test]
	fn should_store_a_validator() {
		let alice_origin = Origin::signed(ALICE_NOMINATOR);

		with_validators(10, 10_000, |validators| {
			Balances::make_free_balance_be(&ALICE_NOMINATOR, 10_000);

			assert_ok!(DPoS::nominate(alice_origin, validators[0].clone().0, 10_000));
			assert_eq!(DPoS::nominations(ALICE_NOMINATOR).unwrap().0, validators[0].clone().0);
		})
	}

	#[test]
	fn should_store_a_validator_when_chained() {
		let alice_origin = Origin::signed(ALICE_NOMINATOR);
		let bob_origin = Origin::signed(BOB_NOMINATOR);

		with_validators(10, 10_000, |validators| {
			Balances::make_free_balance_be(&ALICE_NOMINATOR, 10_000);
			Balances::make_free_balance_be(&BOB_NOMINATOR, 10_000);

			assert_ok!(DPoS::nominate(alice_origin, validators[0].clone().0, 10_000));
			assert_ok!(DPoS::nominate(bob_origin, ALICE_NOMINATOR, 10_000));
			assert_eq!(DPoS::nominations(BOB_NOMINATOR).unwrap().0, validators[0].clone().0);
		})
	}

	#[test]
	fn should_store_a_validators_delegated_stake() {
		let alice_origin = Origin::signed(ALICE_NOMINATOR);

		with_validators(10, 10_000, |validators| {
			Balances::make_free_balance_be(&ALICE_NOMINATOR, 10_000);

			assert_ok!(DPoS::nominate(alice_origin, validators[0].clone().0, 10_000));
			assert_eq!(DPoS::validators_totals(validators[0].clone().0), 10_000);
		})
	}

	#[test]
	fn should_store_a_validators_delegated_stake_when_chained() {
		let alice_origin = Origin::signed(ALICE_NOMINATOR);
		let bob_origin = Origin::signed(BOB_NOMINATOR);

		with_validators(10, 10_000, |validators| {
			Balances::make_free_balance_be(&ALICE_NOMINATOR, 10_000);
			Balances::make_free_balance_be(&BOB_NOMINATOR, 10_000);

			assert_ok!(DPoS::nominate(alice_origin, validators[0].clone().0, 10_000));
			assert_ok!(DPoS::nominate(bob_origin, ALICE_NOMINATOR, 10_000));
			assert_eq!(DPoS::validators_totals(validators[0].clone().0), 20_000);
		})
	}

	#[test]
	fn fail_on_insufficent_balance() {
		let alice_origin = Origin::signed(ALICE_NOMINATOR);

		with_validators(10, 10_000, |validators| {
			Balances::make_free_balance_be(&ALICE_NOMINATOR, 1_000);

			assert_noop!(
				DPoS::nominate(alice_origin, validators[0].clone().0, 1_000),
				crate::Error::<Test>::InsufficentBalance
			);
		})
	}

	#[test]
	fn fail_on_self_nomination() {
		let alice_origin = Origin::signed(ALICE_NOMINATOR);

		with_validators(10, 10_000, |_| {
			Balances::make_free_balance_be(&ALICE_NOMINATOR, 10_000);

			assert_noop!(
				DPoS::nominate(alice_origin, ALICE_NOMINATOR, 10_000),
				crate::Error::<Test>::InvalidNomination
			);
		})
	}

	#[test]
	fn fail_on_irrelevant_nomination() {
		let alice_origin = Origin::signed(ALICE_NOMINATOR);

		with_validators(10, 10_000, |_| {
			Balances::make_free_balance_be(&ALICE_NOMINATOR, 10_000);

			assert_noop!(
				DPoS::nominate(alice_origin, BOB_NOMINATOR, 10_000),
				crate::Error::<Test>::InvalidNomination
			);
		})
	}
}
