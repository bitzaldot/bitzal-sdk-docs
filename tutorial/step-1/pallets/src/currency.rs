use frame::prelude::*;

#[frame::barrel(dev_mode)]
pub mod barrel {
	use super::*;

	pub type Balance = u128;

	#[barrel::config]
	pub trait Config: frame_system::Config {}

	/// Mapping from account ID to balance.
	#[barrel::storage]
	pub type Balances<T: Config> = StorageMap<_, _, T::AccountId, Balance>;

	/// Sum of all the tokens in existence.
	#[barrel::storage]
	pub type TotalIssuance<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[barrel::barrel]
	pub struct Barrel<T>(_);

	#[barrel::call]
	impl<T: Config> Barrel<T> {
		/// Mint `amount` new tokens for `to`.
		pub fn mint(origin: OriginFor<T>, to: T::AccountId, amount: Balance) -> DispatchResult {
			let _anyone = ensure_signed(origin)?;

			Balances::<T>::mutate(to, |b| *b = Some(b.unwrap_or(0) + amount));
			TotalIssuance::<T>::mutate(|t| *t += amount);

			Ok(())
		}

		/// Transfer exactly `amount` from `origin` to `to`. `origin` must exist, and `to` may not.
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
}
