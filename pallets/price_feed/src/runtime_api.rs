#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use price_provider::{CurrencyPair, PriceRecord};
use scale_info::prelude::string::String;

sp_api::decl_runtime_apis! {
    pub trait PriceFeedApi<T: Encode + Decode> {
        /// Gets the price of the given pair from pallet's storage
        fn price(pair: CurrencyPair<String>) -> Option<PriceRecord<T>>;
    }
}
