//! Price provider and related stuff.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod currency_pair;
pub mod price_record;
pub use crate::{currency_pair::CurrencyPair, price_record::PriceRecord};

/// Trait to provide price of currency pairs.
/// The raw price amount should be divided by 10^decimals and rounded to get price per 1 unit.
pub trait PriceProvider<T: frame_system::Config> {
    /// Get the latest price of the given currency pair.
    /// Returns the price record containing raw price amount, decimals, and the block number.
    fn pair_price<S: AsRef<str>>(
        currency_pair: CurrencyPair<S>,
    ) -> Option<PriceRecord<T::BlockNumber>>;
}

/// Trait to provide price of the bound currency pair.
/// The raw price amount should be divided by 10^decimals and rounded to get price per 1 unit.
pub trait BoundPriceProvider<T: frame_system::Config> {
    /// Bound pair to provide price for.
    const PAIR: CurrencyPair<&'static str>;

    /// Get the latest price of the bound currency pair.
    /// Returns the price record containing raw price amount, decimals, and the block number.
    fn price() -> Option<PriceRecord<T::BlockNumber>>;
}
