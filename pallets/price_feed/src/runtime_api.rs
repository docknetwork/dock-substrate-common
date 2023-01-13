use codec::{Decode, Encode};
use frame_support::traits::Get;
use price_provider::{CurrencySymbolPair, PriceRecord};
use scale_info::prelude::string::String;

sp_api::decl_runtime_apis! {
    pub trait PriceFeedApi<T: Encode + Decode, MaxLen: Get<u32>> {
        /// Gets the price of the given pair from pallet's storage
        fn price(pair: CurrencySymbolPair<String, String>) -> Option<PriceRecord<T>>;
    }
}
