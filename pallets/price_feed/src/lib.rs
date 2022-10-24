//! Provides access to the mapping from currency pair to its price relation updated by some oracle.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch,
    traits::{Get, IsType},
    weights::Weight,
};
use frame_system::{self as system, ensure_root};
use scale_info::prelude::string::String;
use sp_std::prelude::*;

pub mod runtime_api;
pub use price_provider::{BoundPriceProvider, CurrencyPair, PriceProvider, PriceRecord};
use system::ensure_signed;

mod migrations;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[derive(codec::Encode, codec::Decode, Clone, scale_info::TypeInfo, PartialEq, Eq)]
pub enum Releases {
    V1SinglePair,
    V2MultiPair,
}

impl Default for Releases {
    fn default() -> Self {
        Releases::V1SinglePair
    }
}

pub trait Config: system::Config + scale_info::TypeInfo {
    /// The overarching event type.
    type Event: From<Event<Self>>
        + IsType<<Self as frame_system::Config>::Event>
        + Into<<Self as system::Config>::Event>;
}

decl_storage! {
    trait Store for Module<T: Config> as PriceFeedModule {
        /// Stores operators for the currency pairs.
        pub Operators get(fn operators):
            double_map hasher(identity) CurrencyPair<String>, hasher(blake2_128_concat) <T as frame_system::Config>::AccountId => Option<()>;

        /// Stores prices of the currency pairs.
        /// Each price record contains raw amount, decimals, and a block number on which it was added to the storage.
        pub Prices get(fn price): map hasher(identity) CurrencyPair<String> => Option<PriceRecord<T::BlockNumber>>;

        /// Current storage version.
        StorageVersion build(|_| Releases::V2MultiPair): Releases;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Config>::AccountId,
        PriceRecord = PriceRecord<<T as system::Config>::BlockNumber>,
    {
        OperatorAdded(CurrencyPair<String>, AccountId),
        OperatorRemoved(CurrencyPair<String>, AccountId),
        PriceSet(CurrencyPair<String>, PriceRecord, AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        NotAnOperator,
        OperatorIsAlreadyAdded,
        OperatorDoesntExist
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {
        fn deposit_event() = default;

        type Error = Error<T>;

        /// Sets price for the given currency pair. Only callable by the currency price operator.
        #[weight = <T as frame_system::Config>::DbWeight::get().reads_writes(1, 1)]
        pub fn set_price(origin, currency_pair: CurrencyPair<String>, price: u64, decimals: u8) -> dispatch::DispatchResult {
            let account = ensure_signed(origin)?;

            if <Operators<T>>::get(&currency_pair, &account).is_some() {
                let price_record = PriceRecord::new(price, decimals, <system::Pallet<T>>::block_number());
                <Prices<T>>::insert(&currency_pair, &price_record);

                Self::deposit_event(Event::<T>::PriceSet(currency_pair, price_record, account));

                return Ok(())
            }

            Err(Error::<T>::NotAnOperator.into())
        }

        /// Adds an operator for the given currency pair. Only callable by Root.
        #[weight = <T as frame_system::Config>::DbWeight::get().reads_writes(1, 1)]
        pub fn add_operator(origin, currency_pair: CurrencyPair<String>, operator: T::AccountId) -> dispatch::DispatchResult{
            ensure_root(origin)?;

            <Operators<T>>::try_mutate(&currency_pair, &operator, |allowed| {
                if allowed.is_none() {
                    *allowed = Some(());

                    Ok(())
                } else {
                    Err(Error::<T>::OperatorIsAlreadyAdded)
                }
            })?;
            Self::deposit_event(Event::<T>::OperatorAdded(currency_pair, operator));

            Ok(())
        }

        /// Removes an operator for the given currency pair. Only callable by Root.
        #[weight = <T as frame_system::Config>::DbWeight::get().reads_writes(1, 1)]
        pub fn remove_operator(origin, currency_pair: CurrencyPair<String>, operator: T::AccountId) -> dispatch::DispatchResult{
            ensure_root(origin)?;

            <Operators<T>>::try_mutate(&currency_pair, &operator, |allowed| {
                if allowed.is_some() {
                    allowed.take();

                    Ok(())
                } else {
                    Err(Error::<T>::OperatorDoesntExist)
                }
            })?;
            Self::deposit_event(Event::<T>::OperatorRemoved(currency_pair, operator));

            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            T::DbWeight::get().reads(1) + if StorageVersion::get() == Releases::V1SinglePair {
                migrations::v1::migrate_to_v2::<T>()
            } else {
                Weight::zero()
            }
        }
    }
}

impl<T: Config> PriceProvider<T> for Module<T> {
    /// Returns the price of the given currency pair from storage.
    /// This operation performs a single storage read.
    fn pair_price<S: AsRef<str>>(
        currency_pair: CurrencyPair<S>,
    ) -> Option<PriceRecord<T::BlockNumber>> {
        Self::price(currency_pair)
    }
}
