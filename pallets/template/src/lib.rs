#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use common::pallets::dpos::DPoS;
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::{AccountIdConversion, Saturating},
		traits::{Currency, ExistenceRequirement, Hooks},
		weights::Weight,
		Blake2_128Concat, PalletId,
	};
	use frame_system::pallet_prelude::{BlockNumberFor, OriginFor, *};

	/// Account ID as configured for this pallet by the runtime
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	/// Balance as configured for this pallet by the runtime
	pub type BalanceOf<T> = <<T as Config>::NativeCurrency as Currency<AccountIdOf<T>>>::Balance;
	/// Maximum number of validators for the consensus pallet
	pub type ConsensusMaximumAuthorities<T> = <T as pallet_aura::Config>::MaxAuthorities;
	pub type ConsensusAuthorityId<T> = <T as pallet_aura::Config>::AuthorityId;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_aura::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type Epoch: Get<<Self as frame_system::Config>::BlockNumber>;

		/// Minimum stake that a nominator must put down to place a nomination
		#[pallet::constant]
		type MinimumNominatorStake: Get<BalanceOf<Self>>;

		/// Mimimum stake that a validator must put down to register as a validator
		#[pallet::constant]
		type MinimumValidatorStake: Get<BalanceOf<Self>>;

		/// Some currency token
		type NativeCurrency: Currency<AccountIdOf<Self>>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(block_number: T::BlockNumber) -> Weight {
			// if last block (block_number - 1) was the end of an epoch
			if block_number.saturating_sub(1_u32.into()) % T::Epoch::get() == 0_u32.into() {
				let mut validator_stakes: Vec<(_, _)> = ValidatorStakes::<T>::iter().collect();
				validator_stakes.sort_by(|a, b| b.1.cmp(&a.1));

				let validator_ids: Vec<_> = validator_stakes
					.iter()
					.filter_map(|validator_stake| Validators::<T>::get(validator_stake.clone().0))
					.collect();

				let next_set: BoundedVec<ConsensusAuthorityId<T>, ConsensusMaximumAuthorities<T>> =
					BoundedVec::truncate_from(validator_ids);

				pallet_aura::pallet::Pallet::<T>::change_authorities(next_set);
			}
			0_u64
		}
	}

	/// Collection of validator session keys queried by the validator account ID
	#[pallet::storage]
	#[pallet::getter(fn validators)]
	pub type Validators<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, ConsensusAuthorityId<T>, OptionQuery>;

	/// Nominated stakes for a validator
	#[pallet::storage]
	#[pallet::getter(fn validators_totals)]
	pub type ValidatorStakes<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>, ValueQuery>;

	/// Collection of nominations queried by nominator account ID
	#[pallet::storage]
	#[pallet::getter(fn nominations)]
	pub type Nominations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AccountIdOf<T>,
		(AccountIdOf<T>, BalanceOf<T>),
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new nomination has been created
		NewNomination {
			nominator_id: AccountIdOf<T>,
			nominee_id: AccountIdOf<T>,
			stake: BalanceOf<T>,
		},
		/// A new validator has been registered
		ValidatorRegistered {
			account_id: AccountIdOf<T>,
			session_key: ConsensusAuthorityId<T>,
			stake: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Nominator/Validator does not have a sufficient balance to nominate a validator
		InsufficentBalance,
		/// Nominator tried to nominate an irrelevent account
		InvalidNomination,
		/// Validator is not an authority
		NotAnAuthority,
	}

	impl<T: Config> Pallet<T> {
		/// Returns the account ID of the pallet, used for holding stake
		pub fn pallet_account_id() -> AccountIdOf<T> {
			T::PalletId::get().into_account_truncating()
		}

		/// Will attempt to find the validator at the root of a delegation chain.
		///
		/// Returns `None` if unable to find a validator from the chain.
		pub fn find_nominated_validator(
			nominee: AccountIdOf<T>,
			stake: BalanceOf<T>,
		) -> Option<(AccountIdOf<T>, BalanceOf<T>)> {
			if Validators::<T>::contains_key(&nominee) {
				return Some((nominee, stake))
			}
			match Nominations::<T>::get(nominee) {
				Some((delegated_nominee, their_stake)) => Self::find_nominated_validator(
					delegated_nominee,
					stake.saturating_add(their_stake),
				),
				None => None,
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register yourself as a validator.
		#[pallet::weight(10_000)]
		pub fn register_validator(
			origin: OriginFor<T>,
			session_key: ConsensusAuthorityId<T>,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			let account_id = ensure_signed(origin)?;

			<Self as DPoS>::register_validator(account_id, session_key, stake)?;

			Ok(())
		}

		/// Nominate a validator while providing some stake.
		///
		/// Altenatively, provide the accound ID of another nominator and have you stake chained to
		/// thier nominee.
		#[pallet::weight(10_000)]
		pub fn nominate(
			origin: OriginFor<T>,
			nominee_id: AccountIdOf<T>,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			let account_id = ensure_signed(origin)?;

			<Self as DPoS>::nominate(account_id, nominee_id, stake)?;

			Ok(())
		}
	}

	impl<T: Config> DPoS for Pallet<T> {
		type AccountId = AccountIdOf<T>;
		type Balance = BalanceOf<T>;
		type SessionKey = ConsensusAuthorityId<T>;

		/// Register yourself as a validator.
		fn register_validator(
			account_id: Self::AccountId,
			session_key: Self::SessionKey,
			stake: Self::Balance,
		) -> DispatchResult {
			ensure!(stake >= T::MinimumValidatorStake::get(), Error::<T>::InsufficentBalance);

			T::NativeCurrency::transfer(
				&account_id,
				&Self::pallet_account_id(),
				stake,
				ExistenceRequirement::AllowDeath,
			)?;

			Validators::<T>::insert(&account_id, &session_key);

			Self::deposit_event(Event::<T>::ValidatorRegistered { account_id, session_key, stake });

			Ok(())
		}

		/// Nominate a validator while providing some stake.
		///
		/// Altenatively, provide the accound ID of another nominator and have you stake chained to
		/// thier nominee.
		fn nominate(
			nominator_id: Self::AccountId,
			nominee_id: Self::AccountId,
			stake: Self::Balance,
		) -> DispatchResult {
			ensure!(stake >= T::MinimumNominatorStake::get(), Error::<T>::InsufficentBalance);
			ensure!(nominator_id != nominee_id, Error::<T>::InvalidNomination);
			ensure!(
				Validators::<T>::contains_key(&nominee_id) ||
					Nominations::<T>::contains_key(&nominee_id),
				Error::<T>::InvalidNomination
			);

			T::NativeCurrency::transfer(
				&nominator_id,
				&Self::pallet_account_id(),
				stake,
				ExistenceRequirement::AllowDeath,
			)?;

			let (nominee_id, total_stake) = Self::find_nominated_validator(nominee_id, stake)
				.ok_or(Error::<T>::InvalidNomination)?;

			Nominations::<T>::insert(&nominator_id, (&nominee_id, stake));
			ValidatorStakes::<T>::mutate(&nominee_id, |validator_stake| {
				*validator_stake = total_stake;
			});

			Self::deposit_event(Event::<T>::NewNomination { nominator_id, nominee_id, stake });

			Ok(())
		}
	}
}
