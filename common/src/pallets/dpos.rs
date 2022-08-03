//! Pallet DPoS traits
use frame_support::pallet_prelude::*;

/// Type and function definitions for a delegated proof of stake system to manage validators
pub trait DPoS {
	/// Account ID type used for both nominators and validators
	type AccountId;
	/// Balance type used to manage staking
	type Balance;
	/// Session key type used by consensus pallet (i.e aura)
	type SessionKey;

	/// Register yourself as a validator.
	fn register_validator(
		account_id: Self::AccountId,
		session_key: Self::SessionKey,
		stake: Self::Balance,
	) -> DispatchResult;

	/// Nominate a validator while providing some stake.
	///
	/// Altenatively, provide the accound ID of another nominator and have you stake chained to
	/// thier nominee.
	fn nominate(
		nominator_id: Self::AccountId,
		nominee_ids: Self::AccountId,
		stake: Self::Balance,
	) -> DispatchResult;
}
