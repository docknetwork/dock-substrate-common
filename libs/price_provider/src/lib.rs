//! Price provider and related stuff.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Get;

pub mod currency_pair;
pub mod price_record;

pub use currency_pair::{
    CurrencyPair, EncodableAsString, StaticCurrencyPair, StoredCurrencyPair,
    StoredCurrencyPairError,
};
pub use price_record::PriceRecord;

/// Trait to provide price of currency pairs.
/// The raw price amount should be divided by 10^decimals and rounded to get price per 1 unit.
pub trait PriceProvider<T: frame_system::Config> {
    type Error;

    /// Get the latest price of the given currency pair.
    /// Returns the price record containing raw price amount, decimals, and the block number.
    fn pair_price<From, To>(
        currency_pair: CurrencyPair<From, To>,
    ) -> Result<Option<PriceRecord<T::BlockNumber>>, Self::Error>
    where
        From: EncodableAsString,
        To: EncodableAsString;
}

/// Trait to provide price of the bound currency pair.
/// The raw price amount should be divided by 10^decimals and rounded to get price per 1 unit.
pub trait StaticPriceProvider<T, P>
where
    T: frame_system::Config,
    P: Get<CurrencyPair<&'static str, &'static str>>,
{
    /// Bound pair.
    type Error;

    /// Get the latest price of the bound currency pair.
    /// Returns the price record containing raw price amount, decimals, and the block number.
    fn price() -> Result<Option<PriceRecord<T::BlockNumber>>, Self::Error>;

    /// Returns underlying bound pair to provide a price for.
    fn pair() -> CurrencyPair<&'static str, &'static str> {
        P::get()
    }
}

impl<T, P, PP> StaticPriceProvider<T, P> for PP
where
    T: frame_system::Config,
    P: Get<CurrencyPair<&'static str, &'static str>>,
    PP: PriceProvider<T>,
{
    type Error = PP::Error;

    fn price() -> Result<Option<PriceRecord<<T as frame_system::Config>::BlockNumber>>, Self::Error>
    {
        Self::pair_price(<Self as StaticPriceProvider<T, P>>::pair())
    }
}
