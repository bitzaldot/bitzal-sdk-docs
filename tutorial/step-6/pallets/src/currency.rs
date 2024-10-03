use frame::prelude::*;

#[frame::barrel(dev_mode)]
pub mod barrel {
	use super::*;

	pub type Balance = u128;

	#[barrel::config]
	pub trait Config: frame_system::Config {}

	#[barrel::storage]
	pub type Balances<T: Config> = StorageMap<_, _, T::AccountId, Balance>;

	#[barrel::storage]
	pub type TotalIssuance<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[barrel::barrel]
	pub struct Barrel<T>(_);

	#[barrel::call]
	impl<T: Config> Barrel<T> {
		pub fn mint(origin: OriginFor<T>, to: T::AccountId, amount: Balance) -> DispatchResult {
			let _anyone = ensure_signed(origin)?;

			Balances::<T>::mutate(to, |b| *b = Some(b.unwrap_or(0) + amount));
			TotalIssuance::<T>::mutate(|t| *t += amount);

			Ok(())
		}

		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, amount: Balance) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let sender_balance = Balances::<T>::get(&sender).ok_or("NonExistentAccount")?;
			if sender_balance < amount {
				return Err("notEnoughBalance".into())
			}
			let reminder = sender_balance - amount;

			Balances::<T>::mutate(to, |b| *b = Some(b.unwrap_or(0) + amount));
			Balances::<T>::insert(&sender, reminder);

			Ok(())
		}
	}

	#[derive(frame::derive::DefaultNoBound)]
	#[barrel::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub balances: Vec<(T::AccountId, Balance)>,
	}

	#[barrel::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (who, amount) in &self.balances {
				assert!(!Balances::<T>::contains_key(who), "duplicate balance in genesis");
				Balances::<T>::insert(who, amount);
				TotalIssuance::<T>::mutate(|t| *t += amount);
			}
		}
	}

	#[cfg(test)]
	mod tests {
		use super::{barrel as barrel_currency, *};
		use frame::testing_prelude::*;

		type Extrinsic = MockUncheckedExtrinsic<Runtime>;
		type Block = MockBlock<Runtime>;

		construct_runtime!(
			pub struct Runtime
			where
				Block = Block,
				NodeBlock = Block,
				UncheckedExtrinsic = Extrinsic,
			{
				System: frame_system,
				Currency: barrel_currency,
			}
		);

		#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
		impl frame_system::Config for Runtime {
			type RuntimeOrigin = RuntimeOrigin;
			type RuntimeCall = RuntimeCall;
			type RuntimeEvent = RuntimeEvent;
			type BarrelInfo = BarrelInfo;
			type BaseCallFilter = frame::traits::Everything;
			type OnSetCode = ();
		}

		impl barrel::Config for Runtime {}

		const ALICE: <Runtime as frame_system::Config>::AccountId = 1;
		const BOB: <Runtime as frame_system::Config>::AccountId = 2;
		const EVE: <Runtime as frame_system::Config>::AccountId = 3;

		struct ExtBuilder {
			balances: Vec<(<Runtime as frame_system::Config>::AccountId, Balance)>,
		}

		impl Default for ExtBuilder {
			fn default() -> Self {
				Self { balances: vec![(ALICE, 100), (BOB, 100)] }
			}
		}

		impl ExtBuilder {
			fn add_balance(
				mut self,
				who: <Runtime as frame_system::Config>::AccountId,
				amount: Balance,
			) -> Self {
				self.balances.push((who, amount));
				self
			}

			fn build_and_execute(self, test: impl FnOnce() -> ()) {
				let system = frame_system::GenesisConfig::default();
				let currency = barrel_currency::GenesisConfig { balances: self.balances };
				let runtime_genesis = RuntimeGenesisConfig { system, currency };

				let mut ext = TestState::new(runtime_genesis.build_storage().unwrap());
				ext.execute_with(test);
				// possibly do more here.
			}
		}

		#[test]
		fn initial_state_works() {
			ExtBuilder::default().build_and_execute(|| {
				assert_eq!(barrel::Balances::<Runtime>::get(&ALICE), Some(100));
				assert_eq!(barrel::Balances::<Runtime>::get(&BOB), Some(100));
				assert_eq!(barrel::Balances::<Runtime>::get(&EVE), None);
				assert_eq!(barrel::TotalIssuance::<Runtime>::get(), 200);
			});
		}

		#[test]
		fn ext_builder_works() {
			ExtBuilder::default().add_balance(EVE, 42).build_and_execute(|| {
				assert_eq!(barrel::Balances::<Runtime>::get(&EVE), Some(42));
				assert_eq!(barrel::TotalIssuance::<Runtime>::get(), 242);
			})
		}

		#[test]
		#[should_panic]
		fn duplicate_genesis_fails() {
			ExtBuilder::default()
				.add_balance(EVE, 42)
				.add_balance(EVE, 43)
				.build_and_execute(|| {
					assert_eq!(barrel::Balances::<Runtime>::get(&EVE), None);
					assert_eq!(barrel::TotalIssuance::<Runtime>::get(), 242);
				})
		}

		#[test]
		fn test_mint() {
			ExtBuilder::default().build_and_execute(|| {
				// given the initial state, when:
				assert_ok!(barrel::Barrel::<Runtime>::mint(RuntimeOrigin::signed(ALICE), BOB, 100));

				// then:
				assert_eq!(barrel::Balances::<Runtime>::get(&BOB), Some(200));
				assert_eq!(barrel::TotalIssuance::<Runtime>::get(), 300);

				// given:
				assert_ok!(barrel::Barrel::<Runtime>::mint(RuntimeOrigin::signed(ALICE), EVE, 100));

				// then:
				assert_eq!(barrel::Balances::<Runtime>::get(&EVE), Some(100));
				assert_eq!(barrel::TotalIssuance::<Runtime>::get(), 400);
			});
		}

		#[test]
		fn transfer_works() {
			ExtBuilder::default().build_and_execute(|| {
				// given the the initial state, when:
				assert_ok!(barrel::Barrel::<Runtime>::transfer(
					RuntimeOrigin::signed(ALICE),
					BOB,
					50
				));

				// them:
				assert_eq!(barrel::Balances::<Runtime>::get(&ALICE), Some(50));
				assert_eq!(barrel::Balances::<Runtime>::get(&BOB), Some(150));
				assert_eq!(barrel::TotalIssuance::<Runtime>::get(), 200);

				// when:
				assert_ok!(barrel::Barrel::<Runtime>::transfer(
					RuntimeOrigin::signed(BOB),
					ALICE,
					50
				));

				// then:
				assert_eq!(barrel::Balances::<Runtime>::get(&ALICE), Some(100));
				assert_eq!(barrel::Balances::<Runtime>::get(&BOB), Some(100));
				assert_eq!(barrel::TotalIssuance::<Runtime>::get(), 200);
			});
		}

		#[test]
		fn transfer_from_non_existent_fails() {
			ExtBuilder::default().build_and_execute(|| {
				// given the the initial state, when:
				assert_err!(
					barrel::Barrel::<Runtime>::transfer(RuntimeOrigin::signed(EVE), ALICE, 10),
					"NonExistentAccount"
				);

				// then nothing has changed.
				assert_eq!(barrel::Balances::<Runtime>::get(&ALICE), Some(100));
				assert_eq!(barrel::Balances::<Runtime>::get(&BOB), Some(100));
				assert_eq!(barrel::Balances::<Runtime>::get(&EVE), None);
				assert_eq!(barrel::TotalIssuance::<Runtime>::get(), 200);

				// in fact, this frame-helper ensures that nothing in the state has been updated
				// prior and after execution:
				assert_noop!(
					barrel::Barrel::<Runtime>::transfer(RuntimeOrigin::signed(EVE), ALICE, 10),
					"NonExistentAccount"
				);
			});
		}
	}
}
