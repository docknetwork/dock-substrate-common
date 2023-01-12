//! Price provider and related stuff.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod currency_pair;
pub mod price_record;

use currency_pair::{EncodableString, StaticCurrencyPair};
use frame_support::traits::Get;

pub use crate::{currency_pair::CurrencyPair, price_record::PriceRecord};

/// Trait to provide price of currency pairs.
/// The raw price amount should be divided by 10^decimals and rounded to get price per 1 unit.
pub trait PriceProvider<T: frame_system::Config, MaxMemberBytesLen: Get<u32>> {
    /// Get the latest price of the given currency pair.
    /// Returns the price record containing raw price amount, decimals, and the block number.
    fn pair_price<From, To, P>(currency_pair: P) -> Option<PriceRecord<T::BlockNumber>>
    where
        From: EncodableString,
        To: EncodableString,
        P: Into<CurrencyPair<From, To, MaxMemberBytesLen>>;
}

/// Trait to provide price of the bound currency pair.
/// The raw price amount should be divided by 10^decimals and rounded to get price per 1 unit.
pub trait BoundPriceProvider<T: frame_system::Config, MaxMemberBytesLen: Get<u32>> {
    /// `from` currency.
    type From: Get<&'static str>;
    /// `to` currency.
    type To: Get<&'static str>;

    /// Get the latest price of the bound currency pair.
    /// Returns the price record containing raw price amount, decimals, and the block number.
    fn price() -> Option<PriceRecord<T::BlockNumber>>;

    /// Returns underlying bound pair to provide a price for.
    fn pair() -> Option<StaticCurrencyPair<Self::From, Self::To, MaxMemberBytesLen>> {
        StaticCurrencyPair::new()
    }
}
