//! Provides access to the mapping from currency pair to its price relation updated by some oracle.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    traits::{Get, IsType},
    weights::Weight,
};
use frame_system::{self as system, ensure_root};
use scale_info::{prelude::string::String, TypeInfo};
use sp_std::prelude::*;

pub mod runtime_api;
pub use price_provider::{
    BoundedCurrencySymbolPair, BoundedStringConversionError, CurrencySymbolPair, PriceProvider,
    PriceRecord, StaticPriceProvider,
};
use system::ensure_signed;

mod migrations;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Storage version.
#[derive(Encode, Decode, Clone, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
pub enum Releases {
    /// `dock_price_feed` allows querying only a single pair (`DOCK`/`USD`) price.
    V1SinglePair,
    /// `dock_price_feed` allows to query of any pair price
    V2MultiPair,
}

impl Default for Releases {
    fn default() -> Self {
        Releases::V1SinglePair
    }
}

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use super::*;
    use frame_support::pallet_prelude::{OptionQuery, ValueQuery, *};
    use frame_system::pallet_prelude::*;
    use price_provider::currency_pair::LikeString;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[pallet::constant]
        type MaxSymbolBytesLen: Get<u32>;

        /// The overarching event type.
        type Event: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::Event>
            + Into<<Self as system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        OperatorAdded(
            BoundedCurrencySymbolPair<String, String, T::MaxSymbolBytesLen>,
            <T as system::Config>::AccountId,
        ),
        OperatorRemoved(
            BoundedCurrencySymbolPair<String, String, T::MaxSymbolBytesLen>,
            <T as system::Config>::AccountId,
        ),
        PriceSet(
            BoundedCurrencySymbolPair<String, String, T::MaxSymbolBytesLen>,
            PriceRecord<<T as system::Config>::BlockNumber>,
            <T as system::Config>::AccountId,
        ),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The caller isn't an operator for this currency pair.
        NotAnOperator,
        /// Given operator is already added for this currency pair.
        OperatorIsAlreadyAdded,
        /// Provided operator doesn't exist for this currency pair.
        OperatorDoesNotExist,
    }

    /// Stores operators for the currency pairs.
    #[pallet::storage]
    #[pallet::getter(fn operators)]
    pub type Operators<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        BoundedCurrencySymbolPair<String, String, T::MaxSymbolBytesLen>,
        Twox64Concat,
        <T as frame_system::Config>::AccountId,
        (),
        OptionQuery,
    >;

    /// Stores prices of the currency pairs.
    /// Each price record contains raw amount, decimals, and a block number on which it was added to the storage.
    #[pallet::storage]
    #[pallet::getter(fn price)]
    pub type Prices<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedCurrencySymbolPair<String, String, T::MaxSymbolBytesLen>,
        PriceRecord<T::BlockNumber>,
        OptionQuery,
    >;

    /// Current storage version.
    #[pallet::storage]
    #[pallet::getter(fn version)]
    pub type StorageVersion<T> = StorageValue<_, Releases, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        _phantom: sp_std::marker::PhantomData<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                _phantom: Default::default(),
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Sets price for the given currency pair. Only callable by the currency price operator.
        #[pallet::weight(<T as frame_system::Config>::DbWeight::get().reads_writes(1, 1))]
        pub fn set_price(
            origin: OriginFor<T>,
            currency_pair: CurrencySymbolPair<String, String>,
            price: u64,
            decimals: u8,
        ) -> DispatchResult {
            let account = ensure_signed(origin)?;

            let stored_pair = currency_pair.try_into()?;
            if <Operators<T>>::get(&stored_pair, &account).is_some() {
                let price_record =
                    PriceRecord::new(price, decimals, <system::Pallet<T>>::block_number());
                <Prices<T>>::insert(&stored_pair, price_record);

                Self::deposit_event(Event::<T>::PriceSet(stored_pair, price_record, account));

                return Ok(());
            }

            Err(Error::<T>::NotAnOperator.into())
        }

        /// Adds an operator for the given currency pair. Only callable by Root.
        #[pallet::weight(<T as frame_system::Config>::DbWeight::get().reads_writes(1, 1))]
        pub fn add_operator(
            origin: OriginFor<T>,
            currency_pair: CurrencySymbolPair<String, String>,
            operator: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let stored_pair = currency_pair.try_into()?;
            <Operators<T>>::try_mutate(&stored_pair, &operator, |allowed| {
                if allowed.is_none() {
                    *allowed = Some(());

                    Ok(())
                } else {
                    Err(Error::<T>::OperatorIsAlreadyAdded)
                }
            })?;
            Self::deposit_event(Event::<T>::OperatorAdded(stored_pair, operator));

            Ok(())
        }

        /// Removes an operator for the given currency pair. Only callable by Root.
        #[pallet::weight(<T as frame_system::Config>::DbWeight::get().reads_writes(1, 1))]
        pub fn remove_operator(
            origin: OriginFor<T>,
            currency_pair: CurrencySymbolPair<String, String>,
            operator: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let stored_pair = currency_pair.try_into()?;
            <Operators<T>>::try_mutate(&stored_pair, &operator, |allowed| {
                if allowed.is_some() {
                    allowed.take();

                    Ok(())
                } else {
                    Err(Error::<T>::OperatorDoesNotExist)
                }
            })?;
            Self::deposit_event(Event::<T>::OperatorRemoved(stored_pair, operator));

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            T::DbWeight::get().reads(1)
                + if StorageVersion::<T>::get() == Releases::V1SinglePair {
                    migrations::v1::migrate_to_v2::<T>()
                } else {
                    Weight::zero()
                }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            StorageVersion::<T>::put(Releases::V2MultiPair);
        }
    }

    impl<T: Config> PriceProvider<T> for Pallet<T> {
        type Error = BoundedStringConversionError;

        /// Returns the price of the given currency pair from storage.
        /// This operation performs a single storage read.
        fn pair_price<From, To>(
            currency_pair: CurrencySymbolPair<From, To>,
        ) -> Result<Option<PriceRecord<T::BlockNumber>>, Self::Error>
        where
            From: LikeString + 'static,
            To: LikeString + 'static,
        {
            currency_pair
                .try_into()
                .map(Self::price::<BoundedCurrencySymbolPair<_, _, T::MaxSymbolBytesLen>>)
        }
    }
}
